use serde::Deserialize;
use tauri::State;

use crate::{
    app_state::AppState,
    error::{AppError, ErrorPayload},
    repository::teammates::{CreateTeammateInput, TeammatesRepository, UpdateTeammateInput},
};

#[derive(Debug, Deserialize)]
pub struct CreateTeammateRequest {
    platform: String,
    #[serde(rename = "pubgAccountId")]
    pubg_account_id: Option<String>,
    #[serde(rename = "pubgPlayerName")]
    pubg_player_name: String,
    #[serde(rename = "displayNickname")]
    display_nickname: Option<String>,
    #[serde(rename = "isPointsEnabled")]
    is_points_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTeammateRequest {
    id: i64,
    #[serde(rename = "displayNickname")]
    display_nickname: Option<String>,
    #[serde(rename = "isPointsEnabled")]
    is_points_enabled: Option<bool>,
}

#[tauri::command]
pub fn teammates_get_all(
    state: State<'_, AppState>,
) -> Result<Vec<crate::repository::teammates::TeammateDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    TeammatesRepository::new(&connection)
        .get_all()
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn teammates_get_by_id(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<crate::repository::teammates::TeammateDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    TeammatesRepository::new(&connection)
        .get_by_id(id)
        .map_err(|error: AppError| error.into())
}

#[tauri::command(rename_all = "camelCase")]
pub fn teammates_create(
    state: State<'_, AppState>,
    input: CreateTeammateRequest,
) -> Result<crate::repository::teammates::TeammateDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    let create_input = CreateTeammateInput {
        platform: input.platform,
        pubg_account_id: input.pubg_account_id,
        pubg_player_name: input.pubg_player_name,
        display_nickname: input.display_nickname,
        is_points_enabled: input.is_points_enabled.unwrap_or(true),
    };

    TeammatesRepository::new(&connection)
        .create(create_input)
        .map_err(|error: AppError| error.into())
}

#[tauri::command(rename_all = "camelCase")]
pub fn teammates_update(
    state: State<'_, AppState>,
    input: UpdateTeammateRequest,
) -> Result<crate::repository::teammates::TeammateDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    let update_input = UpdateTeammateInput {
        id: input.id,
        display_nickname: input.display_nickname,
        is_points_enabled: input.is_points_enabled,
    };

    TeammatesRepository::new(&connection)
        .update(update_input)
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn teammates_get_history(
    state: State<'_, AppState>,
    id: i64,
) -> Result<TeammateHistoryResponse, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    let teammate_repo = TeammatesRepository::new(&connection);
    let points_repo = crate::repository::points::PointRecordsRepository::new(&connection);

    let teammate = teammate_repo.get_by_id(id).map_err(ErrorPayload::from)?;

    let teammate = teammate.ok_or_else(|| ErrorPayload {
        message: "Teammate not found".to_string(),
    })?;

    let records = points_repo
        .get_by_teammate(id)
        .map_err(ErrorPayload::from)?;

    let total_matches = records.len() as i64;

    Ok(TeammateHistoryResponse {
        teammate,
        records,
        total_matches,
    })
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeammateHistoryResponse {
    teammate: crate::repository::teammates::TeammateDto,
    records: Vec<crate::repository::points::PointRecordDto>,
    total_matches: i64,
}
