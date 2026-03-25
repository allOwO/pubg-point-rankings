use tauri::State;

use crate::{
    app_state::AppState,
    error::{AppError, ErrorPayload},
    repository::matches::MatchesRepository,
};

#[tauri::command]
pub fn matches_get_all(
    state: State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<crate::repository::matches::MatchDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    MatchesRepository::new(&connection)
        .get_all(limit.unwrap_or(100), offset.unwrap_or(0))
        .map_err(|error: AppError| error.into())
}

#[tauri::command(rename_all = "camelCase")]
pub fn matches_get_by_id(
    state: State<'_, AppState>,
    match_id: String,
) -> Result<Option<crate::repository::matches::MatchDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    MatchesRepository::new(&connection)
        .get_by_id(&match_id)
        .map_err(|error: AppError| error.into())
}

#[tauri::command(rename_all = "camelCase")]
pub fn matches_get_players(
    state: State<'_, AppState>,
    match_id: String,
) -> Result<Vec<crate::repository::matches::MatchPlayerDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    MatchesRepository::new(&connection)
        .get_players(&match_id)
        .map_err(|error: AppError| error.into())
}
