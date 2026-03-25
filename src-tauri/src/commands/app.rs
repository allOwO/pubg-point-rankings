use serde::Serialize;
use tauri::State;

use crate::{
    app_state::AppState,
    error::{AppError, ErrorPayload},
    repository::settings::SettingsRepository,
    runtime::game_state::GameProcessStatusSnapshot,
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
pub struct AppStatusDto {
    pub version: String,
    pub database_path: String,
    pub is_database_ready: bool,
    pub sync_status: SyncStatusDto,
}

#[tauri::command]
pub fn app_get_status(state: State<'_, AppState>) -> Result<AppStatusDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    let settings = SettingsRepository::new(&connection);
    let runtime_status = sync::read_status(&state.sync_runtime_status);
    let last_sync_at = settings
        .get_string("last_sync_at", "")
        .map_err(|error: AppError| -> ErrorPayload { error.into() })?;

    Ok(AppStatusDto {
        version: state.app_version.clone(),
        database_path: state.db_path.display().to_string(),
        is_database_ready: true,
        sync_status: SyncStatusDto {
            is_syncing: runtime_status.is_syncing,
            last_sync_at: (!last_sync_at.is_empty()).then_some(last_sync_at),
            current_match_id: runtime_status.current_match_id,
            error: runtime_status.last_error,
        },
    })
}

#[tauri::command]
pub fn app_get_game_process_status(
    state: State<'_, AppState>,
) -> Result<GameProcessStatusSnapshot, ErrorPayload> {
    let runtime_state = state
        .game_process_runtime
        .lock()
        .map_err(|_| ErrorPayload {
            message: "game process state mutex is poisoned".to_string(),
        })?;

    Ok(runtime_state.snapshot())
}
