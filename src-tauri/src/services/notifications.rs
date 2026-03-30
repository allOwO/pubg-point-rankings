use chrono::{Duration as ChronoDuration, Utc};
use rusqlite::Connection;
use serde::Serialize;
use serde_json::Value;

use crate::{
    error::AppError,
    repository::{
        accounts::AccountsRepository,
        matches::MatchesRepository,
        notification_tasks::{
            CreateNotificationTaskInput, NotificationFailedTaskDto, NotificationTaskDto,
            NotificationTasksRepository,
        },
        point_match_meta::PointMatchMetaRepository,
        settings::SettingsRepository,
    },
    services::napcat_runtime,
};

const NOTIFICATION_ENABLED_KEY: &str = "notification_enabled";
const NOTIFICATION_GROUP_ID_KEY: &str = "notification_group_id";
const NOTIFICATION_ONEBOT_TOKEN_KEY: &str = "notification_onebot_token";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendSelectedResultDto {
    pub sent_ids: Vec<i64>,
    pub failed_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
struct SendGroupMsgRequest<'a> {
    group_id: i64,
    message: &'a str,
    auto_escape: bool,
}

pub fn enqueue_match_notification(connection: &Connection, match_id: &str) -> Result<(), AppError> {
    let account = AccountsRepository::new(connection).require_active()?;
    let settings = SettingsRepository::new(connection);
    if !get_account_bool(&settings, account.id, NOTIFICATION_ENABLED_KEY, false)? {
        return Ok(());
    }

    let matches_repo = MatchesRepository::new(connection, account.id);
    let Some(match_data) = matches_repo.get_by_id(match_id)? else {
        return Err(AppError::Message("Match not found".to_string()));
    };
    let players = matches_repo.get_players(match_id)?;

    let self_player = players.iter().find(|player| player.is_self).cloned();
    let preview_placement = self_player.as_ref().and_then(|player| player.placement);
    let preview_match_time = match_data
        .match_end_at
        .clone()
        .unwrap_or_else(|| match_data.played_at.clone());
    let preview_battle_summary = render_battle_summary(self_player.as_ref());
    let message_body = render_match_message(
        &account.self_player_name,
        &match_data.match_id,
        &match_data.played_at,
        self_player.as_ref(),
        &preview_battle_summary,
    );

    NotificationTasksRepository::new(connection, account.id).upsert_pending(
        CreateNotificationTaskInput {
            match_id: match_id.to_string(),
            message_body,
            preview_match_time,
            preview_placement,
            preview_battle_summary,
        },
    )
}

pub fn process_due_notifications(connection: &Connection) -> Result<(), AppError> {
    let account = AccountsRepository::new(connection).require_active()?;
    let repository = NotificationTasksRepository::new(connection, account.id);
    let due_tasks = repository.get_due_retries(&now_iso())?;

    for task in due_tasks {
        if match_is_settled(connection, account.id, &task.match_id)? {
            repository.mark_cancelled_settled(&task.match_id)?;
            continue;
        }

        repository.mark_sending(task.id)?;
        match send_to_group(connection, account.id, &task.message_body) {
            Ok(()) => {
                finalize_notification_success(connection, account.id, &task.match_id)?;
            }
            Err(error) => {
                schedule_or_fail(connection, &repository, &task, &error.to_string())?;
            }
        }
    }

    Ok(())
}

pub fn get_failed_notifications(
    connection: &Connection,
) -> Result<Vec<NotificationFailedTaskDto>, AppError> {
    let account = AccountsRepository::new(connection).require_active()?;
    NotificationTasksRepository::new(connection, account.id).get_failed_manual()
}

pub fn resend_selected_notifications(
    connection: &Connection,
    task_ids: &[i64],
) -> Result<SendSelectedResultDto, AppError> {
    let account = AccountsRepository::new(connection).require_active()?;
    let repository = NotificationTasksRepository::new(connection, account.id);

    let mut sent_ids = Vec::new();
    let mut failed_ids = Vec::new();

    for task_id in task_ids {
        let Some(task) = repository.get_failed_manual_by_id(*task_id)? else {
            failed_ids.push(*task_id);
            continue;
        };

        if match_is_settled(connection, account.id, &task.match_id)? {
            repository.mark_cancelled_settled(&task.match_id)?;
            failed_ids.push(task.id);
            continue;
        }

        repository.mark_sending(task.id)?;
        match send_to_group(connection, account.id, &task.message_body) {
            Ok(()) => {
                finalize_notification_success(connection, account.id, &task.match_id)?;
                sent_ids.push(task.id);
            }
            Err(error) => {
                repository.mark_failed_manual(task.id, &error.to_string())?;
                failed_ids.push(task.id);
            }
        }
    }

    Ok(SendSelectedResultDto {
        sent_ids,
        failed_ids,
    })
}

pub fn delete_failed_notification(connection: &Connection, task_id: i64) -> Result<(), AppError> {
    let account = AccountsRepository::new(connection).require_active()?;
    NotificationTasksRepository::new(connection, account.id).mark_deleted(task_id)
}

pub fn send_test_notification(connection: &Connection) -> Result<(), AppError> {
    let account = AccountsRepository::new(connection).require_active()?;
    let test_message = format!(
        "[PUBG] 通知测试\n账号: {}\n时间: {}",
        account.self_player_name,
        now_iso()
    );
    send_to_group(connection, account.id, &test_message)
}

pub fn finalize_notification_success(
    connection: &Connection,
    account_id: i64,
    match_id: &str,
) -> Result<(), AppError> {
    let repository = NotificationTasksRepository::new(connection, account_id);
    if let Some(task) = repository.get_pending_by_match(match_id)? {
        repository.mark_sent(task.id)?;
    }

    PointMatchMetaRepository::new(connection, account_id).settle_single_match(match_id)
}

fn schedule_or_fail(
    connection: &Connection,
    repository: &NotificationTasksRepository<'_>,
    task: &NotificationTaskDto,
    error: &str,
) -> Result<(), AppError> {
    let Some(delay_seconds) = retry_delay_seconds(task.retry_count) else {
        return repository.mark_failed_manual(task.id, error);
    };

    let next_retry_at = (Utc::now() + ChronoDuration::seconds(delay_seconds)).to_rfc3339();
    repository.mark_retry_scheduled(task.id, task.retry_count + 1, error, &next_retry_at)?;

    if match_is_settled(connection, task.account_id, &task.match_id)? {
        repository.mark_cancelled_settled(&task.match_id)?;
    }

    Ok(())
}

fn send_to_group(connection: &Connection, account_id: i64, message: &str) -> Result<(), AppError> {
    let settings = SettingsRepository::new(connection);
    let group_id_text = settings.get_account_string(account_id, NOTIFICATION_GROUP_ID_KEY, "")?;
    let group_id = group_id_text
        .trim()
        .parse::<i64>()
        .map_err(|_| AppError::Message("notification group id must be an integer".to_string()))?;

    let webui_info = napcat_runtime::open_webui_info(connection)?;
    let one_bot_url = webui_info
        .one_bot_url
        .ok_or_else(|| AppError::Message("onebot url is not available".to_string()))?;

    let configured_token =
        settings.get_account_string(account_id, NOTIFICATION_ONEBOT_TOKEN_KEY, "")?;
    let token = if let Some(webui_token) = webui_info.token {
        if webui_token.trim().is_empty() {
            configured_token
        } else {
            webui_token
        }
    } else {
        configured_token
    };

    let endpoint = format!("{}/send_group_msg", one_bot_url.trim_end_matches('/'));
    let payload = SendGroupMsgRequest {
        group_id,
        message,
        auto_escape: true,
    };

    let mut request = ureq::post(&endpoint);
    if !token.trim().is_empty() {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }

    let response = request.send_json(serde_json::to_value(payload).map_err(|error| {
        AppError::Message(format!("failed to serialize onebot request: {error}"))
    })?);

    let response = response.map_err(|error| {
        AppError::Message(format!(
            "failed to send onebot request to {endpoint}: {error}"
        ))
    })?;

    let value: Value = response
        .into_json()
        .map_err(|error| AppError::Message(format!("failed to parse onebot response: {error}")))?;

    let is_ok_status = value
        .get("status")
        .and_then(Value::as_str)
        .is_some_and(|status| status.eq_ignore_ascii_case("ok"));
    let is_zero_retcode = value
        .get("retcode")
        .and_then(Value::as_i64)
        .is_some_and(|retcode| retcode == 0);

    if is_ok_status || is_zero_retcode {
        return Ok(());
    }

    let message = value
        .get("wording")
        .and_then(Value::as_str)
        .or_else(|| value.get("message").and_then(Value::as_str))
        .unwrap_or("onebot send_group_msg failed");
    Err(AppError::Message(message.to_string()))
}

fn match_is_settled(
    connection: &Connection,
    account_id: i64,
    match_id: &str,
) -> Result<bool, AppError> {
    let meta = PointMatchMetaRepository::new(connection, account_id).get_by_match(match_id)?;
    Ok(meta.is_some_and(|item| item.settled_at.is_some()))
}

fn retry_delay_seconds(retry_count: i64) -> Option<i64> {
    match retry_count {
        0 => Some(10),
        1 => Some(20),
        2 => Some(40),
        _ => None,
    }
}

fn get_account_bool(
    settings: &SettingsRepository<'_>,
    account_id: i64,
    key: &str,
    default_value: bool,
) -> Result<bool, AppError> {
    let fallback = if default_value { "1" } else { "0" };
    let value = settings.get_account_string(account_id, key, fallback)?;
    let normalized = value.trim().to_ascii_lowercase();

    Ok(match normalized.as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => default_value,
    })
}

fn render_battle_summary(
    self_player: Option<&crate::repository::matches::MatchPlayerDto>,
) -> String {
    let Some(player) = self_player else {
        return "暂无玩家数据".to_string();
    };

    format!(
        "击杀 {} / 助攻 {} / 伤害 {:.1} / 扶起 {}",
        player.kills, player.assists, player.damage, player.revives
    )
}

fn render_match_message(
    self_player_name: &str,
    match_id: &str,
    played_at: &str,
    self_player: Option<&crate::repository::matches::MatchPlayerDto>,
    battle_summary: &str,
) -> String {
    let placement = self_player
        .and_then(|player| player.placement)
        .map(|value| format!("第 {value} 名"))
        .unwrap_or_else(|| "名次未知".to_string());

    format!(
        "[PUBG] 战绩同步完成\n玩家: {self_player_name}\n对局: {match_id}\n时间: {played_at}\n名次: {placement}\n{battle_summary}"
    )
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::db::migrations::bootstrap_database;

    use super::{finalize_notification_success, process_due_notifications, retry_delay_seconds};

    #[test]
    fn successful_send_marks_match_settled() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        bootstrap_database(&connection).expect("bootstrap schema");

        connection
            .execute(
                "INSERT INTO matches
                 (account_id, match_id, platform, map_name, game_mode, played_at, status, created_at, updated_at)
                 VALUES (1, 'match-notify-1', 'steam', 'Erangel', 'squad', '2026-03-30T12:00:00Z', 'ready', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                [],
            )
            .expect("insert match");

        connection
            .execute(
                "INSERT INTO match_players
                 (account_id, match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot,
                  team_id, damage, kills, revives, placement, is_self, is_points_enabled_snapshot, points, created_at)
                 VALUES (1, 'match-notify-1', NULL, 'account.self', 'SelfPlayer', 'SelfPlayer',
                         1, 120.0, 2, 0, 3, 1, 1, 640, CURRENT_TIMESTAMP)",
                [],
            )
            .expect("insert player");
        let match_player_id = connection.last_insert_rowid();

        connection
            .execute(
                "INSERT INTO point_records
                 (account_id, match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot,
                  damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot,
                  rounding_mode_snapshot, points, note, created_at)
                 VALUES (1, 'match-notify-1', ?1, NULL, 1, 'Default Rule',
                         2, 300, 150, 'round', 640, NULL, CURRENT_TIMESTAMP)",
                [match_player_id],
            )
            .expect("insert point record");

        connection
            .execute(
                "INSERT INTO match_notification_tasks
                 (account_id, match_id, status, retry_count, next_retry_at, message_body, preview_match_time,
                  preview_placement, preview_battle_summary, last_error, sent_at, deleted_at, created_at, updated_at)
                 VALUES (1, 'match-notify-1', 'sending', 0, NULL, 'body', '2026-03-30T12:00:00Z',
                         3, 'SelfPlayer 640', NULL, NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                [],
            )
            .expect("insert notification task");

        finalize_notification_success(&connection, 1, "match-notify-1").expect("finalize success");

        let settled_at: Option<String> = connection
            .query_row(
                "SELECT settled_at FROM point_match_meta WHERE account_id = 1 AND match_id = 'match-notify-1'",
                [],
                |row| row.get(0),
            )
            .expect("read settled_at");

        assert!(settled_at.is_some());
    }

    #[test]
    fn retry_schedule_uses_10_20_40_seconds() {
        assert_eq!(retry_delay_seconds(0), Some(10));
        assert_eq!(retry_delay_seconds(1), Some(20));
        assert_eq!(retry_delay_seconds(2), Some(40));
        assert_eq!(retry_delay_seconds(3), None);
    }

    #[test]
    fn due_retry_is_cancelled_when_match_already_settled() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        bootstrap_database(&connection).expect("bootstrap schema");

        connection
            .execute(
                "INSERT INTO matches
                 (account_id, match_id, platform, map_name, game_mode, played_at, status, created_at, updated_at)
                 VALUES (1, 'match-notify-settled', 'steam', 'Erangel', 'squad', '2026-03-30T12:00:00Z', 'ready', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                [],
            )
            .expect("insert match");

        connection
            .execute(
                "INSERT INTO point_match_meta
                 (account_id, match_id, note, settled_at, settlement_batch_id, created_at, updated_at)
                 VALUES (1, 'match-notify-settled', NULL, CURRENT_TIMESTAMP, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                [],
            )
            .expect("insert settled meta");

        connection
            .execute(
                "INSERT INTO match_notification_tasks
                 (account_id, match_id, status, retry_count, next_retry_at, message_body, preview_match_time,
                  preview_placement, preview_battle_summary, last_error, sent_at, deleted_at, created_at, updated_at)
                 VALUES (1, 'match-notify-settled', 'retrying', 1, NULL, 'body', '2026-03-30T12:00:00Z',
                         1, 'summary', 'network', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                [],
            )
            .expect("insert retry task");

        process_due_notifications(&connection).expect("process due notifications");

        let status: String = connection
            .query_row(
                "SELECT status FROM match_notification_tasks WHERE account_id = 1 AND match_id = 'match-notify-settled'",
                [],
                |row| row.get(0),
            )
            .expect("load task status");
        assert_eq!(status, "cancelled_settled");
    }
}
