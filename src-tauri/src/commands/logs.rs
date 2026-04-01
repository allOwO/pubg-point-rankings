use serde::Deserialize;
use tauri::State;

use crate::{
    app_state::AppState,
    error::{AppError, ErrorPayload},
    services::logs::{self, LogEntryDto, LogStatusDto},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLogSettingsInput {
    enabled: bool,
    directory: String,
}

#[tauri::command]
pub fn logs_get_status(state: State<'_, AppState>) -> Result<LogStatusDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    logs::get_log_status(&connection).map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn logs_get_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<LogEntryDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    logs::read_recent_log_entries(&connection, limit.unwrap_or(500))
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn logs_update_settings(
    state: State<'_, AppState>,
    input: UpdateLogSettingsInput,
) -> Result<LogStatusDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    logs::update_log_settings(&connection, input.enabled, &input.directory)
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn logs_open_directory(state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    logs::open_log_directory(&connection).map_err(|error: AppError| error.into())
}
