use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration as ChronoDuration, FixedOffset, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
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
const NOTIFICATION_TEMPLATE_V1_KEY: &str = "notification_template_v1";
const TEMPLATE_LINE_IDS: [&str; 6] = [
    "header", "player1", "player2", "player3", "player4", "battle",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendSelectedResultDto {
    pub sent_ids: Vec<i64>,
    pub failed_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationTemplateLineConfigDto {
    pub id: String,
    pub prefix: String,
    pub suffix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationTemplateConfigDto {
    pub order: Vec<String>,
    pub lines: HashMap<String, NotificationTemplateLineConfigDto>,
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

    let template_config = get_template_config(connection)?;
    let self_player = players.iter().find(|player| player.is_self).cloned();
    let preview_placement = self_player.as_ref().and_then(|player| player.placement);
    let preview_match_time = match_data
        .match_end_at
        .clone()
        .unwrap_or_else(|| match_data.played_at.clone());
    let preview_battle_summary = render_battle_summary(&players);
    let message_body = render_match_message(
        &template_config,
        &preview_match_time,
        &players,
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
        let message_body = rebuild_notification_message(
            connection,
            account.id,
            &task.match_id,
            &task.message_body,
        )?;
        match send_to_group(connection, account.id, &message_body) {
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
            let status = repository.get_status_by_id(task.id)?;
            apply_send_selected_result(task.id, status.as_deref(), &mut sent_ids, &mut failed_ids);
            continue;
        }

        repository.mark_sending(task.id)?;
        let message_body = rebuild_notification_message(
            connection,
            account.id,
            &task.match_id,
            &task.message_body,
        )?;
        match send_to_group(connection, account.id, &message_body) {
            Ok(()) => {
                finalize_notification_success(connection, account.id, &task.match_id)?;
            }
            Err(error) => {
                repository.mark_failed_manual(task.id, &error.to_string())?;
            }
        }

        let status = repository.get_status_by_id(task.id)?;
        apply_send_selected_result(task.id, status.as_deref(), &mut sent_ids, &mut failed_ids);
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

pub fn get_template_config(
    connection: &Connection,
) -> Result<NotificationTemplateConfigDto, AppError> {
    let account = AccountsRepository::new(connection).require_active()?;
    let settings = SettingsRepository::new(connection);
    let raw = settings.get_account_string(account.id, NOTIFICATION_TEMPLATE_V1_KEY, "")?;
    if raw.trim().is_empty() {
        return Ok(default_template_config());
    }

    match serde_json::from_str::<NotificationTemplateConfigDto>(&raw) {
        Ok(parsed) => normalize_template_config(parsed).or_else(|_| Ok(default_template_config())),
        Err(_) => Ok(default_template_config()),
    }
}

pub fn save_template_config(
    connection: &Connection,
    config: &NotificationTemplateConfigDto,
) -> Result<NotificationTemplateConfigDto, AppError> {
    let account = AccountsRepository::new(connection).require_active()?;
    let normalized = normalize_template_config(config.clone())?;
    let json = serde_json::to_string(&normalized).map_err(|error| {
        AppError::Message(format!(
            "failed to serialize notification template config: {error}"
        ))
    })?;
    SettingsRepository::new(connection).set_account(
        account.id,
        NOTIFICATION_TEMPLATE_V1_KEY,
        &json,
    )?;
    Ok(normalized)
}

pub fn default_template_config() -> NotificationTemplateConfigDto {
    let mut lines = HashMap::new();
    for id in TEMPLATE_LINE_IDS {
        lines.insert(
            id.to_string(),
            NotificationTemplateLineConfigDto {
                id: id.to_string(),
                prefix: String::new(),
                suffix: String::new(),
            },
        );
    }

    NotificationTemplateConfigDto {
        order: TEMPLATE_LINE_IDS.iter().map(|id| id.to_string()).collect(),
        lines,
    }
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

fn apply_send_selected_result(
    task_id: i64,
    status: Option<&str>,
    sent_ids: &mut Vec<i64>,
    failed_ids: &mut Vec<i64>,
) {
    match status {
        Some("sent") => sent_ids.push(task_id),
        _ => failed_ids.push(task_id),
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

fn normalize_template_config(
    config: NotificationTemplateConfigDto,
) -> Result<NotificationTemplateConfigDto, AppError> {
    let mut seen = HashSet::new();
    if config.order.len() != TEMPLATE_LINE_IDS.len() {
        return Err(AppError::Message(
            "notification template order must contain 6 lines".to_string(),
        ));
    }
    for id in &config.order {
        if !TEMPLATE_LINE_IDS.contains(&id.as_str()) {
            return Err(AppError::Message(format!("invalid template line id: {id}")));
        }
        if !seen.insert(id.clone()) {
            return Err(AppError::Message(format!(
                "duplicate template line id: {id}"
            )));
        }
    }

    let mut normalized_lines = HashMap::new();
    for id in TEMPLATE_LINE_IDS {
        let Some(line) = config.lines.get(id) else {
            return Err(AppError::Message(format!(
                "missing template line config: {id}"
            )));
        };
        if line.id != id {
            return Err(AppError::Message(format!(
                "template line id mismatch for {id}"
            )));
        }
        normalized_lines.insert(
            id.to_string(),
            NotificationTemplateLineConfigDto {
                id: id.to_string(),
                prefix: line.prefix.clone(),
                suffix: line.suffix.clone(),
            },
        );
    }

    Ok(NotificationTemplateConfigDto {
        order: config.order,
        lines: normalized_lines,
    })
}

fn rebuild_notification_message(
    connection: &Connection,
    account_id: i64,
    match_id: &str,
    fallback_message: &str,
) -> Result<String, AppError> {
    let matches_repo = MatchesRepository::new(connection, account_id);
    let Some(match_data) = matches_repo.get_by_id(match_id)? else {
        return Ok(fallback_message.to_string());
    };
    let players = matches_repo.get_players(match_id)?;
    if players.is_empty() {
        return Ok(fallback_message.to_string());
    }

    let played_at = match_data
        .match_end_at
        .clone()
        .unwrap_or_else(|| match_data.played_at.clone());
    let battle_summary = render_battle_summary(&players);
    let template_config = get_template_config(connection)?;

    Ok(render_match_message(
        &template_config,
        &played_at,
        &players,
        &battle_summary,
    ))
}

fn render_battle_summary(players: &[crate::repository::matches::MatchPlayerDto]) -> String {
    let participants: Vec<&crate::repository::matches::MatchPlayerDto> = players
        .iter()
        .filter(|player| player.is_points_enabled_snapshot)
        .collect();

    if participants.len() < 2 {
        return "无对战结果".to_string();
    }

    let mut highest = participants[0];
    let mut lowest = participants[0];
    for player in participants.iter().skip(1) {
        if player.points > highest.points
            || (player.points == highest.points && player.id < highest.id)
        {
            highest = player;
        }
        if player.points < lowest.points
            || (player.points == lowest.points && player.id < lowest.id)
        {
            lowest = player;
        }
    }

    let gap = highest.points - lowest.points;
    if gap == 0 {
        return "无人胜出".to_string();
    }

    format!(
        "{} → {} {} 分",
        display_player_name(lowest),
        display_player_name(highest),
        gap
    )
}

fn render_match_message(
    template_config: &NotificationTemplateConfigDto,
    played_at: &str,
    players: &[crate::repository::matches::MatchPlayerDto],
    battle_summary: &str,
) -> String {
    let self_player = players.iter().find(|player| player.is_self);
    let placement = self_player
        .and_then(|player| player.placement)
        .map(|value| format!("第{value}名"))
        .unwrap_or_else(|| "名次未知".to_string());
    let player_lines = build_player_lines(players, self_player.and_then(|player| player.team_id));
    let line_values = HashMap::from([
        (
            "header".to_string(),
            format!("{}｜{}", format_match_time(played_at), placement),
        ),
        ("player1".to_string(), player_lines[0].clone()),
        ("player2".to_string(), player_lines[1].clone()),
        ("player3".to_string(), player_lines[2].clone()),
        ("player4".to_string(), player_lines[3].clone()),
        ("battle".to_string(), battle_summary.to_string()),
    ]);

    render_template_message(template_config, &line_values)
}

fn render_template_message(
    template_config: &NotificationTemplateConfigDto,
    line_values: &HashMap<String, String>,
) -> String {
    template_config
        .order
        .iter()
        .filter_map(|id| {
            let line = template_config.lines.get(id)?;
            let content = line_values.get(id)?;
            if content.trim().is_empty() || content.trim() == "-" {
                return None;
            }
            Some(format!("{}{}{}", line.prefix, content, line.suffix))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_player_lines(
    players: &[crate::repository::matches::MatchPlayerDto],
    team_id: Option<i64>,
) -> [String; 4] {
    let mut team_players: Vec<&crate::repository::matches::MatchPlayerDto> = players
        .iter()
        .filter(|player| match team_id {
            Some(target_team_id) => player.team_id == Some(target_team_id),
            None => true,
        })
        .collect();

    team_players.sort_by(|left, right| {
        right
            .is_self
            .cmp(&left.is_self)
            .then_with(|| right.points.cmp(&left.points))
            .then_with(|| right.kills.cmp(&left.kills))
            .then_with(|| right.assists.cmp(&left.assists))
            .then_with(|| right.damage.total_cmp(&left.damage))
    });

    let mut lines = [String::new(), String::new(), String::new(), String::new()];
    for (index, player) in team_players.into_iter().take(4).enumerate() {
        lines[index] = format!(
            "{}：{}杀 / {}伤 / {}救 / {}分",
            display_player_name(player),
            player.kills,
            player.damage.trunc() as i64,
            player.revives,
            player.points
        );
    }
    lines
}

fn display_player_name(player: &crate::repository::matches::MatchPlayerDto) -> String {
    player
        .display_nickname_snapshot
        .clone()
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| player.pubg_player_name.clone())
}

fn format_match_time(value: &str) -> String {
    DateTime::parse_from_rfc3339(value)
        .map(|date| {
            date.with_timezone(&FixedOffset::east_opt(8 * 60 * 60).expect("valid UTC+8 offset"))
                .format("%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_else(|_| value.to_string())
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rusqlite::Connection;

    use crate::db::migrations::bootstrap_database;

    use super::{
        apply_send_selected_result, default_template_config, finalize_notification_success,
        get_template_config, process_due_notifications, rebuild_notification_message,
        render_template_message, retry_delay_seconds, save_template_config,
        NotificationTemplateConfigDto, NotificationTemplateLineConfigDto,
    };

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

    #[test]
    fn template_config_round_trips_per_account() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        bootstrap_database(&connection).expect("bootstrap schema");

        let default_config =
            get_template_config(&connection).expect("load default template config");
        assert_eq!(default_config, default_template_config());

        let saved = save_template_config(
            &connection,
            &NotificationTemplateConfigDto {
                order: vec![
                    "battle".to_string(),
                    "header".to_string(),
                    "player1".to_string(),
                    "player2".to_string(),
                    "player3".to_string(),
                    "player4".to_string(),
                ],
                lines: std::collections::HashMap::from([
                    (
                        "header".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "header".to_string(),
                            prefix: "[".to_string(),
                            suffix: "]".to_string(),
                        },
                    ),
                    (
                        "player1".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "player1".to_string(),
                            prefix: "".to_string(),
                            suffix: "".to_string(),
                        },
                    ),
                    (
                        "player2".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "player2".to_string(),
                            prefix: "".to_string(),
                            suffix: "".to_string(),
                        },
                    ),
                    (
                        "player3".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "player3".to_string(),
                            prefix: "".to_string(),
                            suffix: "".to_string(),
                        },
                    ),
                    (
                        "player4".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "player4".to_string(),
                            prefix: "".to_string(),
                            suffix: "".to_string(),
                        },
                    ),
                    (
                        "battle".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "battle".to_string(),
                            prefix: "RESULT: ".to_string(),
                            suffix: "".to_string(),
                        },
                    ),
                ]),
            },
        )
        .expect("save template config");

        assert_eq!(saved.order[0], "battle");
        assert_eq!(saved.lines["header"].prefix, "[");
        assert_eq!(saved.lines["battle"].prefix, "RESULT: ");
        assert_eq!(
            get_template_config(&connection).expect("reload template config"),
            saved
        );
    }

    #[test]
    fn render_template_message_applies_order_and_affixes() {
        let message = render_template_message(
            &NotificationTemplateConfigDto {
                order: vec![
                    "battle".to_string(),
                    "header".to_string(),
                    "player1".to_string(),
                    "player2".to_string(),
                    "player3".to_string(),
                    "player4".to_string(),
                ],
                lines: std::collections::HashMap::from([
                    (
                        "header".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "header".to_string(),
                            prefix: "[".to_string(),
                            suffix: "]".to_string(),
                        },
                    ),
                    (
                        "player1".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "player1".to_string(),
                            prefix: "".to_string(),
                            suffix: " <-1".to_string(),
                        },
                    ),
                    (
                        "player2".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "player2".to_string(),
                            prefix: "".to_string(),
                            suffix: "".to_string(),
                        },
                    ),
                    (
                        "player3".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "player3".to_string(),
                            prefix: "".to_string(),
                            suffix: "".to_string(),
                        },
                    ),
                    (
                        "player4".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "player4".to_string(),
                            prefix: "".to_string(),
                            suffix: "".to_string(),
                        },
                    ),
                    (
                        "battle".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "battle".to_string(),
                            prefix: "RESULT: ".to_string(),
                            suffix: "".to_string(),
                        },
                    ),
                ]),
            },
            &std::collections::HashMap::from([
                ("header".to_string(), "03-30 06:47｜第4名".to_string()),
                (
                    "player1".to_string(),
                    "allOwO：1杀 / 143伤 / 0救 / 317分".to_string(),
                ),
                ("player2".to_string(), "-".to_string()),
                ("player3".to_string(), "-".to_string()),
                ("player4".to_string(), "-".to_string()),
                ("battle".to_string(), "张三 → 李四 12 分".to_string()),
            ]),
        );

        assert_eq!(
            message,
            "RESULT: 张三 → 李四 12 分\n[03-30 06:47｜第4名]\nallOwO：1杀 / 143伤 / 0救 / 317分 <-1"
        );
    }

    #[test]
    fn rebuild_failed_message_uses_latest_template_and_falls_back_when_match_missing() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        bootstrap_database(&connection).expect("bootstrap schema");

        connection
            .execute(
                "INSERT INTO matches
                 (account_id, match_id, platform, map_name, game_mode, played_at, match_end_at, status, created_at, updated_at)
                 VALUES (1, 'match-template-retry', 'steam', 'Erangel', 'squad', '2026-03-29T14:47:49Z', '2026-03-29T14:47:49Z', 'ready', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                [],
            )
            .expect("insert match");

        connection
            .execute(
                "INSERT INTO match_players
                 (account_id, match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot,
                  team_id, damage, kills, assists, revives, placement, is_self, is_points_enabled_snapshot, points, created_at)
                 VALUES
                 (1, 'match-template-retry', NULL, 'self', 'allOwO', NULL, 7, 143.6, 1, 0, 0, 4, 1, 1, 317, CURRENT_TIMESTAMP),
                 (1, 'match-template-retry', NULL, 'mate1', '队友A', NULL, 7, 88.2, 0, 0, 1, 4, 0, 1, 184, CURRENT_TIMESTAMP),
                 (1, 'match-template-retry', NULL, 'mate2', '队友B', NULL, 7, 201.4, 2, 1, 0, 4, 0, 1, 402, CURRENT_TIMESTAMP),
                 (1, 'match-template-retry', NULL, 'mate3', '队友C', NULL, 7, 0.0, 0, 0, 0, 4, 0, 1, 0, CURRENT_TIMESTAMP)",
                [],
            )
            .expect("insert players");

        let saved_config = save_template_config(
            &connection,
            &NotificationTemplateConfigDto {
                order: vec![
                    "battle".to_string(),
                    "header".to_string(),
                    "player1".to_string(),
                    "player2".to_string(),
                    "player3".to_string(),
                    "player4".to_string(),
                ],
                lines: HashMap::from([
                    (
                        "header".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "header".to_string(),
                            prefix: "[".to_string(),
                            suffix: "]".to_string(),
                        },
                    ),
                    (
                        "player1".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "player1".to_string(),
                            prefix: "".to_string(),
                            suffix: "".to_string(),
                        },
                    ),
                    (
                        "player2".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "player2".to_string(),
                            prefix: "".to_string(),
                            suffix: "".to_string(),
                        },
                    ),
                    (
                        "player3".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "player3".to_string(),
                            prefix: "".to_string(),
                            suffix: "".to_string(),
                        },
                    ),
                    (
                        "player4".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "player4".to_string(),
                            prefix: "".to_string(),
                            suffix: "".to_string(),
                        },
                    ),
                    (
                        "battle".to_string(),
                        NotificationTemplateLineConfigDto {
                            id: "battle".to_string(),
                            prefix: "RESULT: ".to_string(),
                            suffix: "".to_string(),
                        },
                    ),
                ]),
            },
        )
        .expect("save template config");

        let rebuilt =
            rebuild_notification_message(&connection, 1, "match-template-retry", "persisted body")
                .expect("rebuild message");
        assert!(rebuilt.starts_with("RESULT: 队友C → 队友B 402 分\n[03-29 22:47｜第4名]"));
        assert!(rebuilt.contains("allOwO：1杀 / 143伤 / 0救 / 317分"));
        assert!(!rebuilt.contains("\n-\n"));
        assert!(!rebuilt.ends_with("\n-"));
        assert_eq!(saved_config.order[0], "battle");

        let fallback =
            rebuild_notification_message(&connection, 1, "missing-match", "persisted body")
                .expect("fallback message");
        assert_eq!(fallback, "persisted body");
    }

    #[test]
    fn apply_send_selected_result_uses_persisted_status() {
        let mut sent_ids = Vec::new();
        let mut failed_ids = Vec::new();

        apply_send_selected_result(5, Some("sent"), &mut sent_ids, &mut failed_ids);
        apply_send_selected_result(6, Some("failed_manual"), &mut sent_ids, &mut failed_ids);
        apply_send_selected_result(7, Some("cancelled_settled"), &mut sent_ids, &mut failed_ids);
        apply_send_selected_result(8, None, &mut sent_ids, &mut failed_ids);

        assert_eq!(sent_ids, vec![5]);
        assert_eq!(failed_ids, vec![6, 7, 8]);
    }
}
