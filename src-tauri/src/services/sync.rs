use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rusqlite::Connection;
use serde::Serialize;

use crate::{
    engine::calculator::{calculate_points, CalculatedPoints, PlayerStats, RuleConfig},
    error::AppError,
    parser::telemetry::{parse_match_detail, parse_telemetry},
    pubg::client::{PubgClient, PubgMatch, PubgMatchParticipantStats},
    repository::{
        accounts::AccountsRepository,
        matches::{
            CreateMatchDamageEventInput, CreateMatchInput, CreateMatchKillEventInput,
            CreateMatchKnockEventInput, CreateMatchPlayerInput, CreateMatchPlayerWeaponStatInput,
            CreateMatchReviveEventInput, MatchDto, MatchPlayerDto, MatchesRepository,
        },
        points::{CreatePointRecordInput, PointRecordsRepository},
        rules::PointRulesRepository,
        settings::SettingsRepository,
        teammates::{TeammateDto, TeammatesRepository},
    },
    services::{
        logs::{self, LogLevel},
        notifications,
    },
};

const REMOTE_FETCH_CONCURRENCY: usize = 4;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncRuntimeStatus {
    pub is_syncing: bool,
    pub current_match_id: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManualSyncTaskState {
    Idle,
    Syncing,
    Success,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ManualSyncTaskStatus {
    pub state: ManualSyncTaskState,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub error_message: Option<String>,
}

impl Default for ManualSyncTaskStatus {
    fn default() -> Self {
        Self {
            state: ManualSyncTaskState::Idle,
            started_at: None,
            finished_at: None,
            error_message: None,
        }
    }
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

#[derive(Debug, Clone)]
struct MatchSyncBundle {
    match_id: String,
    platform: String,
    pubg_match: PubgMatch,
    participant_stats: Vec<PubgMatchParticipantStats>,
    telemetry_url: String,
    telemetry_json: String,
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

pub fn read_manual_task_status(
    manual_status: &Arc<Mutex<ManualSyncTaskStatus>>,
) -> Result<ManualSyncTaskStatus, AppError> {
    manual_status
        .lock()
        .map(|status| status.clone())
        .map_err(|_| AppError::Message("manual sync task status mutex is poisoned".to_string()))
}

pub fn spawn_manual_recent_matches_batch(
    db: Arc<Mutex<Connection>>,
    sync_runtime: Arc<Mutex<SyncRuntimeStatus>>,
    manual_status: Arc<Mutex<ManualSyncTaskStatus>>,
    limit: usize,
) -> SyncResult {
    let busy = begin_sync(&sync_runtime, "recent-batch");
    if let Some(result) = busy {
        return result;
    }

    if let Err(error) = set_manual_task_syncing(&manual_status) {
        end_sync(&sync_runtime, Some(error.to_string()));
        return SyncResult::failed(error.to_string());
    }

    thread::spawn(move || {
        let result = run_manual_sync_job(&manual_status, || {
            let connection = db
                .lock()
                .map_err(|_| AppError::Message("database mutex is poisoned".to_string()))?;
            sync_recent_matches_batch_inner(&connection, limit)
        });

        if result.success {
            end_sync(&sync_runtime, None);
        } else {
            end_sync(&sync_runtime, result.error.clone());
        }
    });

    SyncResult {
        success: true,
        match_data: None,
        players: None,
        points: None,
        error: None,
    }
}

pub fn sync_recent_match(
    connection: &Connection,
    sync_runtime: &Arc<Mutex<SyncRuntimeStatus>>,
) -> Result<SyncResult, AppError> {
    let active_account = AccountsRepository::new(connection).require_active()?;
    let api_key = require_api_key(&active_account.pubg_api_key)?;
    let self_player_name = require_player_name(&active_account.self_player_name)?;
    let platform = normalize_platform(active_account.self_platform.clone());
    let pubg_client = PubgClient::new(api_key);

    let recent_matches =
        pubg_client.get_recent_matches_for_player_name(&self_player_name, &platform, 1)?;
    let Some(match_id) = recent_matches.first() else {
        let result = SyncResult::failed("No recent matches found.");
        set_runtime_error(sync_runtime, result.error.clone());
        return Ok(result);
    };

    sync_match(connection, sync_runtime, match_id, Some(platform.as_str()))
}

pub fn sync_recent_matches_batch(
    connection: &Connection,
    sync_runtime: &Arc<Mutex<SyncRuntimeStatus>>,
    limit: usize,
) -> Result<SyncResult, AppError> {
    let busy = begin_sync(sync_runtime, "recent-batch");
    if let Some(result) = busy {
        return Ok(result);
    }

    let result = sync_recent_matches_batch_inner(connection, limit);
    match result {
        Ok(result) => {
            if result.success {
                end_sync(sync_runtime, None);
            } else {
                end_sync(sync_runtime, result.error.clone());
            }
            Ok(result)
        }
        Err(error) => {
            end_sync(sync_runtime, Some(error.to_string()));
            Ok(SyncResult::failed(error.to_string()))
        }
    }
}

pub fn sync_recent_match_with_retry(
    connection: &Connection,
    sync_runtime: &Arc<Mutex<SyncRuntimeStatus>>,
    retry_limit: u64,
    retry_delay: Duration,
) {
    let _ = logs::write_log_record(
        connection,
        LogLevel::Info,
        "sync",
        "automatic recent-match sync started",
    );

    let mut attempts = 0u64;
    loop {
        attempts += 1;
        match sync_recent_match(connection, sync_runtime) {
            Ok(result) if result.success => {
                let message = if attempts == 1 {
                    "automatic recent-match sync completed".to_string()
                } else {
                    format!("automatic recent-match sync completed after {attempts} attempts")
                };
                let _ = logs::write_log_record(connection, LogLevel::Info, "sync", &message);
                break;
            }
            Ok(result) => {
                let error_message = result
                    .error
                    .unwrap_or_else(|| "automatic recent-match sync failed".to_string());

                if attempts > retry_limit.saturating_add(1) {
                    let _ = logs::write_log_record(
                        connection,
                        LogLevel::Error,
                        "sync",
                        &format!(
                            "automatic recent-match sync failed after {attempts} attempts: {error_message}"
                        ),
                    );
                    break;
                }

                let _ = logs::write_log_record(
                    connection,
                    LogLevel::Warn,
                    "sync",
                    &format!(
                        "automatic recent-match sync attempt {attempts} failed: {error_message}; retrying in {} seconds",
                        retry_delay.as_secs()
                    ),
                );
                thread::sleep(retry_delay);
            }
            Err(error) => {
                if attempts > retry_limit.saturating_add(1) {
                    let _ = logs::write_log_record(
                        connection,
                        LogLevel::Error,
                        "sync",
                        &format!(
                            "automatic recent-match sync failed after {attempts} attempts: {error}"
                        ),
                    );
                    break;
                }

                let _ = logs::write_log_record(
                    connection,
                    LogLevel::Warn,
                    "sync",
                    &format!(
                        "automatic recent-match sync attempt {attempts} failed: {error}; retrying in {} seconds",
                        retry_delay.as_secs()
                    ),
                );
                thread::sleep(retry_delay);
            }
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

fn sync_recent_matches_batch_inner(
    connection: &Connection,
    limit: usize,
) -> Result<SyncResult, AppError> {
    let active_account = AccountsRepository::new(connection).require_active()?;
    let api_key = require_api_key(&active_account.pubg_api_key)?;
    let self_player_name = require_player_name(&active_account.self_player_name)?;
    let platform = normalize_platform(active_account.self_platform.clone());
    let pubg_client = PubgClient::new(api_key.clone());
    let matches_repo = MatchesRepository::new(connection, active_account.id);
    let points_repo = PointRecordsRepository::new(connection, active_account.id);

    let recent_match_ids =
        pubg_client.get_recent_matches_for_player_name(&self_player_name, &platform, limit)?;
    if recent_match_ids.is_empty() {
        return Ok(SyncResult::failed("No recent matches found."));
    }

    let mut match_ids_to_fetch = Vec::new();
    let mut latest_existing_result = None;
    for match_id in &recent_match_ids {
        if match_needs_refresh(&matches_repo, &points_repo, match_id)? {
            match_ids_to_fetch.push(match_id.clone());
            continue;
        }

        if latest_existing_result.is_none() {
            latest_existing_result = Some(SyncResult {
                success: true,
                match_data: matches_repo.get_by_id(match_id)?,
                players: Some(matches_repo.get_players(match_id)?),
                points: None,
                error: None,
            });
        }
    }

    if match_ids_to_fetch.is_empty() {
        return Ok(latest_existing_result.unwrap_or_else(|| SyncResult {
            success: true,
            match_data: None,
            players: None,
            points: None,
            error: None,
        }));
    }

    let fetched = fetch_match_bundles_concurrently(
        &api_key,
        &platform,
        &match_ids_to_fetch,
        REMOTE_FETCH_CONCURRENCY,
    );

    let mut latest_success = latest_existing_result;
    let mut first_error = None;
    for (match_id, result) in fetched {
        match result {
            Ok(bundle) => match persist_match_bundle(
                connection,
                &active_account.self_player_name,
                &platform,
                bundle,
            ) {
                Ok(sync_result) if sync_result.success => {
                    if latest_success.is_none() {
                        latest_success = Some(sync_result);
                    }
                }
                Ok(sync_result) => {
                    mark_match_failed(connection, &match_id);
                    if first_error.is_none() {
                        first_error = sync_result.error;
                    }
                }
                Err(error) => {
                    mark_match_failed(connection, &match_id);
                    if first_error.is_none() {
                        first_error = Some(error.to_string());
                    }
                }
            },
            Err(error) => {
                mark_match_failed(connection, &match_id);
                if first_error.is_none() {
                    first_error = Some(error.to_string());
                }
            }
        }
    }

    Ok(latest_success.unwrap_or_else(|| {
        SyncResult::failed(
            first_error.unwrap_or_else(|| "Failed to sync recent matches.".to_string()),
        )
    }))
}

fn sync_match_inner(
    connection: &Connection,
    match_id: &str,
    platform: Option<&str>,
) -> Result<SyncResult, AppError> {
    let active_account = AccountsRepository::new(connection).require_active()?;
    let api_key = require_api_key(&active_account.pubg_api_key)?;
    let self_player_name = require_player_name(&active_account.self_player_name)?;
    let target_platform = normalize_platform(
        platform
            .map(ToOwned::to_owned)
            .unwrap_or(active_account.self_platform.clone()),
    );

    let matches_repo = MatchesRepository::new(connection, active_account.id);
    let points_repo = PointRecordsRepository::new(connection, active_account.id);
    if !match_needs_refresh(&matches_repo, &points_repo, match_id)? {
        return Ok(SyncResult {
            success: true,
            match_data: matches_repo.get_by_id(match_id)?,
            players: Some(matches_repo.get_players(match_id)?),
            points: None,
            error: None,
        });
    }

    let bundle = fetch_match_bundle(&api_key, &target_platform, match_id)?;
    persist_match_bundle(connection, &self_player_name, &target_platform, bundle)
}

fn fetch_match_bundles_concurrently(
    api_key: &str,
    platform: &str,
    match_ids: &[String],
    concurrency: usize,
) -> Vec<(String, Result<MatchSyncBundle, AppError>)> {
    let chunk_size = concurrency.max(1);
    let mut results = Vec::new();

    for chunk in match_ids.chunks(chunk_size) {
        let mut handles = Vec::with_capacity(chunk.len());
        for match_id in chunk {
            let api_key = api_key.to_string();
            let platform = platform.to_string();
            let match_id = match_id.clone();
            handles.push(thread::spawn(move || {
                let result = fetch_match_bundle(&api_key, &platform, &match_id);
                (match_id, result)
            }));
        }

        for (fallback_match_id, handle) in chunk.iter().cloned().zip(handles) {
            match handle.join() {
                Ok(result) => results.push(result),
                Err(_) => results.push((
                    fallback_match_id,
                    Err(AppError::Message("match fetch worker panicked".to_string())),
                )),
            }
        }
    }

    results
}

fn fetch_match_bundle(
    api_key: &str,
    platform: &str,
    match_id: &str,
) -> Result<MatchSyncBundle, AppError> {
    let pubg_client = PubgClient::new(api_key.to_string());
    let Some(pubg_match) = pubg_client.get_match(match_id, platform)? else {
        return Err(AppError::Message(
            "Match not found in PUBG API.".to_string(),
        ));
    };
    let telemetry_url = pubg_client
        .get_telemetry_url(&pubg_match)
        .ok_or_else(|| AppError::Message("No telemetry URL available for match.".to_string()))?;
    let telemetry_json = pubg_client.get_telemetry(&telemetry_url)?;

    Ok(MatchSyncBundle {
        match_id: pubg_match.id.clone(),
        platform: platform.to_string(),
        participant_stats: pubg_client.extract_match_participants(&pubg_match),
        pubg_match,
        telemetry_url,
        telemetry_json,
    })
}

fn persist_match_bundle(
    connection: &Connection,
    self_player_name: &str,
    target_platform: &str,
    bundle: MatchSyncBundle,
) -> Result<SyncResult, AppError> {
    let active_account = AccountsRepository::new(connection).require_active()?;
    let matches_repo = MatchesRepository::new(connection, active_account.id);
    let telemetry_events = parse_telemetry(&bundle.telemetry_json)?;
    let telemetry_detail = parse_match_detail(&telemetry_events);
    let player_stats =
        merge_player_stats(&bundle.participant_stats, &telemetry_detail.player_stats);

    let match_end_at = telemetry_detail
        .match_end_at
        .clone()
        .or_else(|| Some(bundle.pubg_match.attributes.created_at.clone()));
    let match_start_at = telemetry_detail.match_start_at.clone().or_else(|| {
        derive_match_start_at(
            match_end_at.as_deref(),
            bundle.pubg_match.attributes.duration,
        )
    });

    let create_input = CreateMatchInput {
        match_id: bundle.match_id.clone(),
        platform: bundle.platform.clone(),
        map_name: bundle.pubg_match.attributes.map_name.clone(),
        game_mode: bundle.pubg_match.attributes.game_mode.clone(),
        played_at: bundle.pubg_match.attributes.created_at.clone(),
        match_start_at,
        match_end_at,
        telemetry_url: Some(bundle.telemetry_url.clone()),
        status: "syncing".to_string(),
    };

    if matches_repo.get_by_id(&bundle.match_id)?.is_some() {
        matches_repo.update_match_fields(create_input.clone())?
    } else {
        Some(matches_repo.create(create_input.clone())?)
    };

    let active_rule = PointRulesRepository::new(connection, active_account.id)
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

    let self_team_id = player_stats
        .iter()
        .find(|entry| {
            entry
                .pubg_player_name
                .eq_ignore_ascii_case(self_player_name)
        })
        .and_then(|entry| entry.team_id);
    let self_account_id = player_stats
        .iter()
        .find(|entry| {
            entry
                .pubg_player_name
                .eq_ignore_ascii_case(self_player_name)
        })
        .map(|entry| entry.pubg_account_id.clone());

    let teammates_repo = TeammatesRepository::new(connection, active_account.id);
    let mut teammates_by_key: HashMap<String, TeammateDto> = HashMap::new();
    let mut enabled_player_keys: HashSet<String> = HashSet::new();
    for stats in &player_stats {
        let teammate = if stats.pubg_account_id.trim().is_empty() {
            teammates_repo.get_by_player_name(target_platform, &stats.pubg_player_name)?
        } else {
            teammates_repo
                .get_by_account_id(target_platform, &stats.pubg_account_id)?
                .or_else(|| {
                    teammates_repo
                        .get_by_player_name(target_platform, &stats.pubg_player_name)
                        .ok()
                        .flatten()
                })
        };

        if let Some(teammate) = teammate.clone() {
            teammates_by_key.insert(
                player_identity_key(&stats.pubg_account_id, &stats.pubg_player_name),
                teammate.clone(),
            );
        }

        let is_same_team = self_team_id.is_some() && stats.team_id == self_team_id;
        let is_enabled = if stats
            .pubg_player_name
            .eq_ignore_ascii_case(self_player_name)
        {
            true
        } else if is_same_team {
            teammate
                .as_ref()
                .map(|value| value.is_points_enabled)
                .unwrap_or(true)
        } else {
            false
        };

        if is_enabled {
            enabled_player_keys.insert(player_identity_key(
                &stats.pubg_account_id,
                &stats.pubg_player_name,
            ));
        }
    }

    let calculated_points = calculate_points(&player_stats, &rule, &enabled_player_keys);
    let player_name_by_account_id = player_stats
        .iter()
        .filter(|entry| !entry.pubg_account_id.trim().is_empty())
        .map(|entry| {
            (
                entry.pubg_account_id.clone(),
                entry.pubg_player_name.clone(),
            )
        })
        .collect::<HashMap<_, _>>();

    let tx = connection.unchecked_transaction()?;
    let tx_matches = MatchesRepository::new(&tx, active_account.id);
    let tx_points = PointRecordsRepository::new(&tx, active_account.id);
    let tx_teammates = TeammatesRepository::new(&tx, active_account.id);
    let tx_settings = SettingsRepository::new(&tx);

    tx_points.delete_for_match(&bundle.match_id)?;
    tx_matches.delete_players_for_match(&bundle.match_id)?;
    tx_matches.delete_detail_events_for_match(&bundle.match_id)?;

    for stats in &player_stats {
        if !stats
            .pubg_player_name
            .eq_ignore_ascii_case(self_player_name)
            && self_team_id.is_some()
            && stats.team_id == self_team_id
        {
            if let Some(teammate) = teammates_by_key.get(&player_identity_key(
                &stats.pubg_account_id,
                &stats.pubg_player_name,
            )) {
                tx_teammates
                    .update_last_seen(teammate.id, &bundle.pubg_match.attributes.created_at)?;
            }
        }
    }

    let mut saved_players = Vec::with_capacity(calculated_points.len());
    for calc in &calculated_points {
        let teammate = teammates_by_key
            .get(&player_identity_key(
                &calc.pubg_account_id,
                &calc.pubg_player_name,
            ))
            .cloned();

        let saved_player = tx_matches.create_player(CreateMatchPlayerInput {
            match_id: bundle.match_id.clone(),
            teammate_id: teammate.as_ref().map(|value| value.id),
            pubg_account_id: Some(calc.pubg_account_id.clone()),
            pubg_player_name: calc.pubg_player_name.clone(),
            display_nickname_snapshot: teammate
                .as_ref()
                .and_then(|value| value.display_nickname.clone())
                .or_else(|| Some(calc.pubg_player_name.clone())),
            team_id: calc.team_id,
            damage: calc.damage,
            kills: calc.kills,
            assists: calc.assists,
            revives: calc.revives,
            placement: calc.placement,
            is_self: self_account_id
                .as_ref()
                .is_some_and(|account_id| account_id == &calc.pubg_account_id),
            is_points_enabled_snapshot: calc.is_points_enabled,
            points: calc.total_points,
        })?;

        tx_points.create(CreatePointRecordInput {
            match_id: bundle.match_id.clone(),
            match_player_id: saved_player.id,
            teammate_id: teammate.as_ref().map(|value| value.id),
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

    for event in &telemetry_detail.damage_events {
        tx_matches.create_damage_event(CreateMatchDamageEventInput {
            match_id: bundle.match_id.clone(),
            attacker_account_id: event.attacker_account_id.clone(),
            attacker_name: event.attacker_name.clone(),
            victim_account_id: event.victim_account_id.clone(),
            victim_name: event.victim_name.clone(),
            damage: event.damage,
            damage_type_category: event.damage_type_category.clone(),
            damage_causer_name: event.damage_causer_name.clone(),
            event_at: event.event_at.clone(),
        })?;
    }

    for event in &telemetry_detail.kill_events {
        tx_matches.create_kill_event(CreateMatchKillEventInput {
            match_id: bundle.match_id.clone(),
            killer_account_id: event.killer_account_id.clone(),
            killer_name: event.killer_name.clone(),
            victim_account_id: event.victim_account_id.clone(),
            victim_name: event.victim_name.clone(),
            assistant_account_id: event.assistant_account_id.clone(),
            assistant_name: event
                .assistant_account_id
                .as_ref()
                .and_then(|account_id| player_name_by_account_id.get(account_id).cloned()),
            damage_type_category: event.damage_type_category.clone(),
            damage_causer_name: event.damage_causer_name.clone(),
            event_at: event.event_at.clone(),
        })?;
    }

    for event in &telemetry_detail.knock_events {
        tx_matches.create_knock_event(CreateMatchKnockEventInput {
            match_id: bundle.match_id.clone(),
            attacker_account_id: event.attacker_account_id.clone(),
            attacker_name: event.attacker_name.clone(),
            victim_account_id: event.victim_account_id.clone(),
            victim_name: event.victim_name.clone(),
            damage_type_category: event.damage_type_category.clone(),
            damage_causer_name: event.damage_causer_name.clone(),
            event_at: event.event_at.clone(),
        })?;
    }

    for event in &telemetry_detail.revive_events {
        tx_matches.create_revive_event(CreateMatchReviveEventInput {
            match_id: bundle.match_id.clone(),
            reviver_account_id: event.reviver_account_id.clone(),
            reviver_name: event.reviver_name.clone(),
            victim_account_id: event.victim_account_id.clone(),
            victim_name: event.victim_name.clone(),
            event_at: event.event_at.clone(),
        })?;
    }

    for stat in &telemetry_detail.weapon_damage_stats {
        tx_matches.create_weapon_stat(CreateMatchPlayerWeaponStatInput {
            match_id: bundle.match_id.clone(),
            pubg_account_id: Some(stat.pubg_account_id.clone()),
            pubg_player_name: stat.pubg_player_name.clone(),
            weapon_name: stat.weapon_name.clone(),
            total_damage: stat.total_damage,
        })?;
    }

    tx_matches.update_status(&bundle.match_id, "ready")?;
    tx_settings.set_account(active_account.id, "last_sync_at", &chrono_like_iso_utc())?;
    tx.commit()?;
    notifications::enqueue_match_notification(connection, &bundle.match_id)?;

    let final_match =
        MatchesRepository::new(connection, active_account.id).get_by_id(&bundle.match_id)?;
    Ok(SyncResult {
        success: true,
        match_data: final_match,
        players: Some(saved_players),
        points: Some(calculated_points),
        error: None,
    })
}

fn match_needs_refresh(
    matches_repo: &MatchesRepository<'_>,
    points_repo: &PointRecordsRepository<'_>,
    match_id: &str,
) -> Result<bool, AppError> {
    let Some(match_data) = matches_repo.get_by_id(match_id)? else {
        return Ok(true);
    };

    if match_data.status != "ready" {
        return Ok(true);
    }

    if match_data.match_start_at.is_none() || match_data.match_end_at.is_none() {
        return Ok(true);
    }

    if !points_repo.exists_for_match(match_id)? {
        return Ok(true);
    }

    if !matches_repo.has_detail_payload(match_id)? {
        return Ok(true);
    }

    Ok(false)
}

fn merge_player_stats(
    participant_stats: &[PubgMatchParticipantStats],
    telemetry_stats: &[PlayerStats],
) -> Vec<PlayerStats> {
    let mut telemetry_by_key = telemetry_stats
        .iter()
        .cloned()
        .map(|entry| {
            (
                player_identity_key(&entry.pubg_account_id, &entry.pubg_player_name),
                entry,
            )
        })
        .collect::<HashMap<_, _>>();

    let mut merged = Vec::new();
    for participant in participant_stats {
        let key = player_identity_key(&participant.pubg_account_id, &participant.pubg_player_name);
        let telemetry = telemetry_by_key.remove(&key);
        merged.push(PlayerStats {
            pubg_account_id: participant.pubg_account_id.clone(),
            pubg_player_name: participant.pubg_player_name.clone(),
            damage: round_damage(participant.damage),
            kills: participant.kills,
            assists: participant.assists,
            revives: participant.revives,
            team_id: participant
                .team_id
                .or_else(|| telemetry.as_ref().and_then(|entry| entry.team_id)),
            placement: participant
                .placement
                .or_else(|| telemetry.as_ref().and_then(|entry| entry.placement)),
        });
    }

    merged.extend(telemetry_by_key.into_values().map(|mut entry| {
        entry.assists = 0;
        entry.damage = round_damage(entry.damage);
        entry
    }));

    merged.sort_by(|left, right| {
        right
            .kills
            .cmp(&left.kills)
            .then_with(|| right.assists.cmp(&left.assists))
            .then_with(|| {
                right
                    .damage
                    .partial_cmp(&left.damage)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| left.pubg_player_name.cmp(&right.pubg_player_name))
    });
    merged
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

fn set_manual_task_syncing(
    manual_status: &Arc<Mutex<ManualSyncTaskStatus>>,
) -> Result<(), AppError> {
    let mut status = manual_status
        .lock()
        .map_err(|_| AppError::Message("manual sync task status mutex is poisoned".to_string()))?;
    status.state = ManualSyncTaskState::Syncing;
    status.started_at = Some(chrono_like_iso_utc());
    status.finished_at = None;
    status.error_message = None;
    Ok(())
}

fn finish_manual_task(
    manual_status: &Arc<Mutex<ManualSyncTaskStatus>>,
    state: ManualSyncTaskState,
    error_message: Option<String>,
) -> Result<(), AppError> {
    let mut status = manual_status
        .lock()
        .map_err(|_| AppError::Message("manual sync task status mutex is poisoned".to_string()))?;
    status.state = state;
    status.finished_at = Some(chrono_like_iso_utc());
    status.error_message = error_message;
    Ok(())
}

fn run_manual_sync_job<F>(
    manual_status: &Arc<Mutex<ManualSyncTaskStatus>>,
    execute: F,
) -> SyncResult
where
    F: FnOnce() -> Result<SyncResult, AppError>,
{
    if let Err(error) = set_manual_task_syncing(manual_status) {
        return SyncResult::failed(error.to_string());
    }

    let result = match execute() {
        Ok(result) => result,
        Err(error) => SyncResult::failed(error.to_string()),
    };

    let finish_result = if result.success {
        finish_manual_task(manual_status, ManualSyncTaskState::Success, None)
    } else {
        finish_manual_task(
            manual_status,
            ManualSyncTaskState::Failed,
            result.error.clone(),
        )
    };

    if let Err(error) = finish_result {
        return SyncResult::failed(error.to_string());
    }

    result
}

fn mark_match_failed(connection: &Connection, match_id: &str) {
    if let Ok(active_account) = AccountsRepository::new(connection).require_active() {
        let _ =
            MatchesRepository::new(connection, active_account.id).update_status(match_id, "failed");
    }
}

fn require_api_key(api_key: &str) -> Result<String, AppError> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        Err(AppError::Message(
            "Please configure PUBG API key before starting sync.".to_string(),
        ))
    } else {
        Ok(trimmed.to_string())
    }
}

fn require_player_name(player_name: &str) -> Result<String, AppError> {
    let trimmed = player_name.trim();
    if trimmed.is_empty() {
        Err(AppError::Message(
            "Please configure your PUBG player name before syncing.".to_string(),
        ))
    } else {
        Ok(trimmed.to_string())
    }
}

fn normalize_platform(value: String) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "steam" | "xbox" | "psn" | "kakao" => value.trim().to_ascii_lowercase(),
        _ => "steam".to_string(),
    }
}

fn player_identity_key(pubg_account_id: &str, pubg_player_name: &str) -> String {
    let trimmed_account_id = pubg_account_id.trim();
    if !trimmed_account_id.is_empty() {
        return format!("account:{trimmed_account_id}");
    }

    format!("name:{}", pubg_player_name.trim().to_ascii_lowercase())
}

fn round_damage(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn derive_match_start_at(
    match_end_at: Option<&str>,
    duration_seconds: Option<i64>,
) -> Option<String> {
    let match_end_at = match_end_at?;
    let duration_seconds = duration_seconds?;
    let end_at = DateTime::parse_from_rfc3339(match_end_at).ok()?;
    let start_at = end_at - ChronoDuration::seconds(duration_seconds);
    Some(start_at.with_timezone(&Utc).to_rfc3339())
}

fn chrono_like_iso_utc() -> String {
    let now = std::time::SystemTime::now();
    let datetime: chrono::DateTime<chrono::Utc> = now.into();
    datetime.to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use crate::{
        db::migrations::bootstrap_database, repository::settings::SettingsRepository,
        services::logs::read_recent_log_entries,
    };

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let unique = format!(
            "{}-{}",
            prefix,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn seed_logging_settings(connection: &Connection, directory: &Path) {
        bootstrap_database(connection).expect("bootstrap db");
        let settings = SettingsRepository::new(connection);
        settings
            .set("logging_enabled", "1")
            .expect("enable logging");
        settings
            .set("logging_directory", directory.to_string_lossy().as_ref())
            .expect("set logging directory");
    }

    #[test]
    fn run_manual_sync_job_marks_success_state() {
        let manual_status = Arc::new(Mutex::new(ManualSyncTaskStatus::default()));

        let result = run_manual_sync_job(&manual_status, || {
            Ok(SyncResult {
                success: true,
                match_data: None,
                players: None,
                points: None,
                error: None,
            })
        });

        assert!(result.success);
        let latest = read_manual_task_status(&manual_status).expect("read manual status");
        assert_eq!(latest.state, ManualSyncTaskState::Success);
        assert!(latest.started_at.is_some());
        assert!(latest.finished_at.is_some());
        assert_eq!(latest.error_message, None);
    }

    #[test]
    fn run_manual_sync_job_marks_failed_state() {
        let manual_status = Arc::new(Mutex::new(ManualSyncTaskStatus::default()));

        let result = run_manual_sync_job(&manual_status, || Ok(SyncResult::failed("boom")));

        assert!(!result.success);
        let latest = read_manual_task_status(&manual_status).expect("read manual status");
        assert_eq!(latest.state, ManualSyncTaskState::Failed);
        assert_eq!(latest.error_message.as_deref(), Some("boom"));
    }

    #[test]
    fn auto_recent_match_sync_writes_failure_logs() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        let log_dir = make_temp_dir("sync-auto-logs");
        let sync_runtime = Arc::new(Mutex::new(SyncRuntimeStatus::default()));
        seed_logging_settings(&connection, &log_dir);

        sync_recent_match_with_retry(&connection, &sync_runtime, 0, Duration::from_secs(0));

        let entries = read_recent_log_entries(&connection, 20).expect("read log entries");
        assert!(
            entries.iter().any(|entry| {
                entry.source == "sync" && entry.message.contains("automatic recent-match sync")
            }),
            "expected automatic sync log entry, got {entries:?}"
        );

        let _ = fs::remove_dir_all(log_dir);
    }

    #[test]
    fn spawn_manual_recent_matches_batch_sets_syncing_before_thread_work() {
        let db = Arc::new(Mutex::new(
            Connection::open_in_memory().expect("open in-memory db"),
        ));
        let db_guard = db.lock().expect("lock db");
        let sync_runtime = Arc::new(Mutex::new(SyncRuntimeStatus::default()));
        let manual_status = Arc::new(Mutex::new(ManualSyncTaskStatus::default()));

        let result =
            spawn_manual_recent_matches_batch(db.clone(), sync_runtime, manual_status.clone(), 12);

        assert!(result.success);
        let latest = read_manual_task_status(&manual_status).expect("read manual status");
        assert_eq!(latest.state, ManualSyncTaskState::Syncing);

        drop(db_guard);
    }

    #[test]
    fn read_manual_task_status_returns_error_when_mutex_is_poisoned() {
        let manual_status = Arc::new(Mutex::new(ManualSyncTaskStatus::default()));
        let manual_status_for_panic = manual_status.clone();
        let _ = std::thread::spawn(move || {
            let _guard = manual_status_for_panic.lock().expect("lock manual status");
            panic!("poison manual status mutex");
        })
        .join();

        let error = read_manual_task_status(&manual_status).expect_err("expects poisoned mutex");
        assert!(error
            .to_string()
            .contains("manual sync task status mutex is poisoned"));
    }
}
