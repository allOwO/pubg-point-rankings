use tauri::State;

use crate::{
    app_state::AppState,
    error::{AppError, ErrorPayload},
    repository::settings::{AppSettingDto, SettingsRepository},
};

#[tauri::command]
pub fn settings_get_all(state: State<'_, AppState>) -> Result<Vec<AppSettingDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    SettingsRepository::new(&connection)
        .get_all()
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn settings_get(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<AppSettingDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    SettingsRepository::new(&connection)
        .get(&key)
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn settings_set(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    SettingsRepository::new(&connection)
        .set(&key, &value)
        .map(|_| ())
        .map_err(|error: AppError| error.into())
}
