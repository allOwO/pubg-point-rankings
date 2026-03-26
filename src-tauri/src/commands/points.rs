use serde::Deserialize;
use tauri::State;

use crate::{
    app_state::AppState,
    error::{AppError, ErrorPayload},
    repository::{
        accounts::AccountsRepository, point_match_meta::PointMatchMetaRepository,
        points::PointRecordsRepository,
    },
};

#[derive(Debug, Deserialize)]
pub struct UpdatePointMatchNoteInput {
    #[serde(rename = "matchId")]
    match_id: String,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SettlePointMatchesInput {
    #[serde(rename = "endMatchId")]
    end_match_id: String,
}

#[tauri::command]
pub fn points_get_all(
    state: State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<crate::repository::points::PointRecordDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    PointRecordsRepository::new(&connection, account.id)
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
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    PointRecordsRepository::new(&connection, account.id)
        .get_by_match(&match_id)
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn points_get_history_groups(
    state: State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<crate::repository::points::PointHistoryListItemDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    PointRecordsRepository::new(&connection, account.id)
        .get_history_groups(limit.unwrap_or(50), offset.unwrap_or(0))
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn points_get_unsettled_summary(
    state: State<'_, AppState>,
) -> Result<crate::repository::points::UnsettledBattleSummaryDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    PointRecordsRepository::new(&connection, account.id)
        .get_unsettled_summary()
        .map_err(|error: AppError| error.into())
}

#[tauri::command(rename_all = "camelCase")]
pub fn points_update_match_note(
    state: State<'_, AppState>,
    input: UpdatePointMatchNoteInput,
) -> Result<(), ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    PointMatchMetaRepository::new(&connection, account.id)
        .upsert_note(&input.match_id, input.note)
        .map_err(|error: AppError| error.into())
}

#[tauri::command(rename_all = "camelCase")]
pub fn points_settle_through_match(
    state: State<'_, AppState>,
    input: SettlePointMatchesInput,
) -> Result<crate::repository::point_match_meta::SettleThroughMatchResultDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    PointMatchMetaRepository::new(&connection, account.id)
        .settle_through_match(&input.end_match_id)
        .map_err(|error: AppError| error.into())
}
