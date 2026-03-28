use serde::Deserialize;
use tauri::State;

use crate::{
    app_state::AppState,
    error::{AppError, ErrorPayload},
    repository::{
        accounts::AccountsRepository,
        teammates::{
            CreateTeammateInput, RecentTeammateCandidateDto, TeammatesRepository,
            UpdateTeammateInput,
        },
    },
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

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeammateHistoryResponse {
    teammate: crate::repository::teammates::TeammateDto,
    records: Vec<crate::repository::points::PointRecordDto>,
    total_matches: i64,
}

#[tauri::command]
pub fn teammates_get_all(
    state: State<'_, AppState>,
) -> Result<Vec<crate::repository::teammates::TeammateDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    TeammatesRepository::new(&connection, account.id)
        .get_all()
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn teammates_get_recent_candidates(
    state: State<'_, AppState>,
) -> Result<Vec<RecentTeammateCandidateDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    TeammatesRepository::new(&connection, account.id)
        .get_recent_candidates(10)
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
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    TeammatesRepository::new(&connection, account.id)
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
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    let create_input = CreateTeammateInput {
        platform: input.platform,
        pubg_account_id: input.pubg_account_id,
        pubg_player_name: input.pubg_player_name,
        display_nickname: input.display_nickname,
        is_points_enabled: input.is_points_enabled.unwrap_or(true),
    };

    TeammatesRepository::new(&connection, account.id)
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
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    let update_input = UpdateTeammateInput {
        id: input.id,
        display_nickname: input.display_nickname,
        is_points_enabled: input.is_points_enabled,
    };

    TeammatesRepository::new(&connection, account.id)
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
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    let teammate_repo = TeammatesRepository::new(&connection, account.id);
    let points_repo =
        crate::repository::points::PointRecordsRepository::new(&connection, account.id);

    let teammate = teammate_repo.get_by_id(id).map_err(ErrorPayload::from)?;
    let teammate = teammate.ok_or_else(|| ErrorPayload {
        message: "Teammate not found".to_string(),
    })?;

    let records = points_repo
        .get_by_teammate(id)
        .map_err(ErrorPayload::from)?;

    Ok(TeammateHistoryResponse {
        teammate,
        total_matches: records.len() as i64,
        records,
    })
}

#[tauri::command]
pub fn teammates_delete(state: State<'_, AppState>, id: i64) -> Result<(), ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    TeammatesRepository::new(&connection, account.id)
        .delete(id)
        .map_err(|error: AppError| error.into())
}
