use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use rusqlite::Connection;
use serde::Serialize;

use crate::{
    engine::calculator::{calculate_points, CalculatedPoints, RuleConfig},
    error::AppError,
    parser::telemetry::{aggregate_player_stats, parse_telemetry},
    pubg::client::PubgClient,
    repository::{
        matches::{
            CreateMatchInput, CreateMatchPlayerInput, MatchDto, MatchPlayerDto, MatchesRepository,
        },
        points::{CreatePointRecordInput, PointRecordsRepository},
        rules::PointRulesRepository,
        settings::SettingsRepository,
        teammates::{TeammateDto, TeammatesRepository},
    },
};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncRuntimeStatus {
    pub is_syncing: bool,
    pub current_match_id: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub success: bool,
    pub match_data: Option<MatchDto>,
    pub players: Option<Vec<MatchPlayerDto>>,
    pub points: Option<Vec<CalculatedPoints>>,
    pub error: Option<String>,
}

impl SyncResult {
    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            success: false,
            match_data: None,
            players: None,
            points: None,
            error: Some(message.into()),
        }
    }
}

pub fn read_status(sync_runtime: &Arc<Mutex<SyncRuntimeStatus>>) -> SyncRuntimeStatus {
    sync_runtime
        .lock()
        .map(|status| status.clone())
        .unwrap_or_default()
}

pub fn sync_recent_match(
    connection: &Connection,
    sync_runtime: &Arc<Mutex<SyncRuntimeStatus>>,
) -> Result<SyncResult, AppError> {
    let settings = SettingsRepository::new(connection);
    let api_key = settings.get_string("pubg_api_key", "")?;
    if api_key.trim().is_empty() {
        let result = SyncResult::failed("Please configure PUBG API key before starting sync.");
        set_runtime_error(sync_runtime, result.error.clone());
        return Ok(result);
    }

    let self_player_name = settings.get_string("self_player_name", "")?;
    if self_player_name.trim().is_empty() {
        let result = SyncResult::failed("Please configure your PUBG player name before syncing.");
        set_runtime_error(sync_runtime, result.error.clone());
        return Ok(result);
    }

    let platform = normalize_platform(settings.get_string("self_platform", "steam")?);
    let pubg_client = PubgClient::new(api_key);

    let Some(player) = pubg_client.get_player_by_name(&self_player_name, &platform)? else {
        let result = SyncResult::failed("Player not found in PUBG API.");
        set_runtime_error(sync_runtime, result.error.clone());
        return Ok(result);
    };

    let recent_matches = pubg_client.get_recent_matches(&player.id, &platform, 1)?;
    let Some(match_id) = recent_matches.first() else {
        let result = SyncResult::failed("No recent matches found.");
        set_runtime_error(sync_runtime, result.error.clone());
        return Ok(result);
    };

    sync_match(connection, sync_runtime, match_id, Some(platform.as_str()))
}

pub fn sync_recent_match_with_retry(
    connection: &Connection,
    sync_runtime: &Arc<Mutex<SyncRuntimeStatus>>,
    retry_limit: u64,
    retry_delay: Duration,
) {
    let mut attempts = 0u64;
    loop {
        attempts += 1;
        match sync_recent_match(connection, sync_runtime) {
            Ok(result) if result.success => break,
            Ok(_) | Err(_) if attempts > retry_limit.saturating_add(1) => break,
            Ok(_) | Err(_) => thread::sleep(retry_delay),
        }
    }
}

pub fn sync_match(
    connection: &Connection,
    sync_runtime: &Arc<Mutex<SyncRuntimeStatus>>,
    match_id: &str,
    platform: Option<&str>,
) -> Result<SyncResult, AppError> {
    let busy = begin_sync(sync_runtime, match_id);
    if let Some(result) = busy {
        return Ok(result);
    }

    let sync_result = sync_match_inner(connection, match_id, platform);

    match sync_result {
        Ok(result) => {
            if result.success {
                end_sync(sync_runtime, None);
            } else {
                mark_match_failed(connection, match_id);
                end_sync(sync_runtime, result.error.clone());
            }
            Ok(result)
        }
        Err(error) => {
            mark_match_failed(connection, match_id);
            end_sync(sync_runtime, Some(error.to_string()));
            Ok(SyncResult::failed(error.to_string()))
        }
    }
}

fn sync_match_inner(
    connection: &Connection,
    match_id: &str,
    platform: Option<&str>,
) -> Result<SyncResult, AppError> {
    let settings = SettingsRepository::new(connection);
    let api_key = settings.get_string("pubg_api_key", "")?;
    if api_key.trim().is_empty() {
        return Ok(SyncResult::failed(
            "Please configure PUBG API key before starting sync.",
        ));
    }

    let self_player_name = settings.get_string("self_player_name", "")?;
    let target_platform = normalize_platform(
        platform
            .map(ToOwned::to_owned)
            .unwrap_or(settings.get_string("self_platform", "steam")?),
    );

    let pubg_client = PubgClient::new(api_key);
    let matches_repo = MatchesRepository::new(connection);
    let points_repo = PointRecordsRepository::new(connection);

    let mut match_data = matches_repo.get_by_id(match_id)?;
    if match_data.is_none() {
        let Some(pubg_match) = pubg_client.get_match(match_id, &target_platform)? else {
            return Ok(SyncResult::failed("Match not found in PUBG API."));
        };

        let telemetry_url = pubg_client.get_telemetry_url(&pubg_match);
        match_data = Some(matches_repo.create(CreateMatchInput {
            match_id: pubg_match.id,
            platform: target_platform.clone(),
            map_name: pubg_match.attributes.map_name,
            game_mode: pubg_match.attributes.game_mode,
            played_at: pubg_match.attributes.created_at,
            match_start_at: None,
            match_end_at: None,
            telemetry_url,
            status: "detected".to_string(),
        })?);
    }

    let already_ready = match_data
        .as_ref()
        .is_some_and(|existing_match| existing_match.status == "ready");

    if already_ready && points_repo.exists_for_match(match_id)? {
        let existing_players = matches_repo.get_players(match_id)?;
        return Ok(SyncResult {
            success: true,
            match_data,
            players: Some(existing_players),
            points: None,
            error: None,
        });
    }

    matches_repo.update_status(match_id, "syncing")?;

    let telemetry_url = match_data
        .as_ref()
        .and_then(|value| value.telemetry_url.clone())
        .ok_or_else(|| AppError::Message("No telemetry URL available for match.".to_string()))?;

    let telemetry_json = pubg_client.get_telemetry(&telemetry_url)?;
    let telemetry_events = parse_telemetry(&telemetry_json)?;
    let player_stats = aggregate_player_stats(&telemetry_events);

    let active_rule = PointRulesRepository::new(connection)
        .get_active()?
        .ok_or_else(|| AppError::Message("No active point rule configured.".to_string()))?;

    let rule = RuleConfig {
        id: active_rule.id,
        name: active_rule.name,
        damage_points_per_damage: active_rule.damage_points_per_damage,
        kill_points: active_rule.kill_points,
        revive_points: active_rule.revive_points,
        rounding_mode: active_rule.rounding_mode,
    };

    let teammates_repo = TeammatesRepository::new(connection);
    let mut teammates_by_account_id: HashMap<String, TeammateDto> = HashMap::new();
    let mut teammates_by_name: HashMap<String, TeammateDto> = HashMap::new();
    let mut enabled_player_ids: HashSet<String> = HashSet::new();

    for stats in &player_stats {
        let teammate = teammates_repo.find_or_create(
            &stats.pubg_player_name,
            &target_platform,
            Some(stats.pubg_account_id.as_str()),
        )?;
        teammates_repo.update_last_seen(teammate.id)?;

        teammates_by_account_id.insert(stats.pubg_account_id.clone(), teammate.clone());
        teammates_by_name.insert(
            stats.pubg_player_name.to_ascii_lowercase(),
            teammate.clone(),
        );

        let is_self = !self_player_name.trim().is_empty()
            && stats
                .pubg_player_name
                .eq_ignore_ascii_case(self_player_name.trim());
        if teammate.is_points_enabled || is_self {
            enabled_player_ids.insert(stats.pubg_account_id.clone());
        }
    }

    let calculated_points = calculate_points(&player_stats, &rule, &enabled_player_ids);

    let self_account_id = player_stats
        .iter()
        .find(|entry| {
            !self_player_name.trim().is_empty()
                && entry
                    .pubg_player_name
                    .eq_ignore_ascii_case(self_player_name.trim())
        })
        .map(|entry| entry.pubg_account_id.clone());

    let tx = connection.unchecked_transaction()?;
    let tx_matches = MatchesRepository::new(&tx);
    let tx_points = PointRecordsRepository::new(&tx);
    let tx_teammates = TeammatesRepository::new(&tx);
    let tx_settings = SettingsRepository::new(&tx);

    tx_points.delete_for_match(match_id)?;
    tx_matches.delete_players_for_match(match_id)?;

    let mut saved_players = Vec::with_capacity(calculated_points.len());

    for calc in &calculated_points {
        let teammate = teammates_by_account_id
            .get(&calc.pubg_account_id)
            .or_else(|| teammates_by_name.get(&calc.pubg_player_name.to_ascii_lowercase()));

        let saved_player = tx_matches.create_player(CreateMatchPlayerInput {
            match_id: match_id.to_string(),
            teammate_id: teammate.map(|value| value.id),
            pubg_account_id: Some(calc.pubg_account_id.clone()),
            pubg_player_name: calc.pubg_player_name.clone(),
            display_nickname_snapshot: teammate
                .and_then(|value| value.display_nickname.clone())
                .or_else(|| Some(calc.pubg_player_name.clone())),
            team_id: calc.team_id,
            damage: calc.damage,
            kills: calc.kills,
            revives: calc.revives,
            placement: calc.placement,
            is_self: self_account_id
                .as_ref()
                .is_some_and(|account_id| account_id == &calc.pubg_account_id),
            is_points_enabled_snapshot: calc.is_points_enabled,
            points: calc.total_points,
        })?;

        tx_points.create(CreatePointRecordInput {
            match_id: match_id.to_string(),
            match_player_id: saved_player.id,
            teammate_id: teammate.map(|value| value.id),
            rule_id: rule.id,
            rule_name_snapshot: rule.name.clone(),
            damage_points_per_damage_snapshot: rule.damage_points_per_damage,
            kill_points_snapshot: rule.kill_points,
            revive_points_snapshot: rule.revive_points,
            rounding_mode_snapshot: rule.rounding_mode.clone(),
            points: calc.total_points,
            note: None,
        })?;

        if let Some(teammate_data) = teammate {
            let total_points = tx_points.get_total_for_teammate(teammate_data.id)?;
            tx_teammates.update_total_points(teammate_data.id, total_points)?;
        }

        saved_players.push(saved_player);
    }

    tx_matches.update_status(match_id, "ready")?;
    tx_settings.set("last_sync_at", &chrono_like_iso_utc())?;
    tx.commit()?;

    let final_match = MatchesRepository::new(connection).get_by_id(match_id)?;

    Ok(SyncResult {
        success: true,
        match_data: final_match,
        players: Some(saved_players),
        points: Some(calculated_points),
        error: None,
    })
}

fn begin_sync(sync_runtime: &Arc<Mutex<SyncRuntimeStatus>>, match_id: &str) -> Option<SyncResult> {
    let Ok(mut status) = sync_runtime.lock() else {
        return Some(SyncResult::failed("sync runtime mutex is poisoned"));
    };

    if status.is_syncing {
        return Some(SyncResult::failed(
            status
                .current_match_id
                .as_ref()
                .map(|current| format!("Another sync is already running for {current}."))
                .unwrap_or_else(|| "Another sync is already running.".to_string()),
        ));
    }

    status.is_syncing = true;
    status.current_match_id = Some(match_id.to_string());
    status.last_error = None;
    None
}

fn end_sync(sync_runtime: &Arc<Mutex<SyncRuntimeStatus>>, error: Option<String>) {
    if let Ok(mut status) = sync_runtime.lock() {
        status.is_syncing = false;
        status.current_match_id = None;
        status.last_error = error;
    }
}

fn set_runtime_error(sync_runtime: &Arc<Mutex<SyncRuntimeStatus>>, error: Option<String>) {
    if let Ok(mut status) = sync_runtime.lock() {
        status.last_error = error;
    }
}

fn mark_match_failed(connection: &Connection, match_id: &str) {
    let _ = MatchesRepository::new(connection).update_status(match_id, "failed");
}

fn normalize_platform(value: String) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "steam" | "xbox" | "psn" | "kakao" => value.trim().to_ascii_lowercase(),
        _ => "steam".to_string(),
    }
}

fn chrono_like_iso_utc() -> String {
    let now = std::time::SystemTime::now();
    let datetime: chrono::DateTime<chrono::Utc> = now.into();
    datetime.to_rfc3339()
}
