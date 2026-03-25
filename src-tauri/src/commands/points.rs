use tauri::State;

use crate::{
    app_state::AppState,
    error::{AppError, ErrorPayload},
    repository::points::PointRecordsRepository,
};

#[tauri::command]
pub fn points_get_all(
    state: State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<crate::repository::points::PointRecordDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    PointRecordsRepository::new(&connection)
        .get_all(limit.unwrap_or(100), offset.unwrap_or(0))
        .map_err(|error: AppError| error.into())
}

#[tauri::command(rename_all = "camelCase")]
pub fn points_get_by_match(
    state: State<'_, AppState>,
    match_id: String,
) -> Result<Vec<crate::repository::points::PointRecordDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    PointRecordsRepository::new(&connection)
        .get_by_match(&match_id)
        .map_err(|error: AppError| error.into())
}
