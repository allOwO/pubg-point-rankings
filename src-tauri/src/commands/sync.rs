use serde::Serialize;
use tauri::State;

use crate::{
    app_state::AppState,
    error::{AppError, ErrorPayload},
    repository::settings::SettingsRepository,
    services::sync,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatusDto {
    pub is_syncing: bool,
    pub last_sync_at: Option<String>,
    pub current_match_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStartResultDto {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStartMatchResultDto {
    pub success: bool,
    pub r#match: Option<crate::repository::matches::MatchDto>,
    pub players: Option<Vec<crate::repository::matches::MatchPlayerDto>>,
    pub points: Option<Vec<crate::engine::calculator::CalculatedPoints>>,
    pub error: Option<String>,
}

#[tauri::command]
pub fn sync_get_status(state: State<'_, AppState>) -> Result<SyncStatusDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    let runtime_status = sync::read_status(&state.sync_runtime_status);
    let last_sync_at = SettingsRepository::new(&connection)
        .get_string("last_sync_at", "")
        .map_err(|error: AppError| -> ErrorPayload { error.into() })?;

    Ok(SyncStatusDto {
        is_syncing: runtime_status.is_syncing,
        last_sync_at: (!last_sync_at.is_empty()).then_some(last_sync_at),
        current_match_id: runtime_status.current_match_id,
        error: runtime_status.last_error,
    })
}

#[tauri::command]
pub fn sync_start(state: State<'_, AppState>) -> Result<SyncStartResultDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    let result = sync::sync_recent_match(&connection, &state.sync_runtime_status)
        .map_err(|error: AppError| -> ErrorPayload { error.into() })?;

    Ok(SyncStartResultDto {
        success: result.success,
        error: result.error,
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn sync_start_match(
    state: State<'_, AppState>,
    match_id: String,
    platform: Option<String>,
) -> Result<SyncStartMatchResultDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    let result = sync::sync_match(
        &connection,
        &state.sync_runtime_status,
        &match_id,
        platform.as_deref(),
    )
    .map_err(|error: AppError| -> ErrorPayload { error.into() })?;

    Ok(SyncStartMatchResultDto {
        success: result.success,
        r#match: result.match_data,
        players: result.players,
        points: result.points,
        error: result.error,
    })
}
