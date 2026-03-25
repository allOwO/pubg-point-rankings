use serde::Deserialize;
use tauri::State;

use crate::{
    app_state::AppState,
    error::{AppError, ErrorPayload},
    repository::accounts::{
        AccountDto, AccountsRepository, CreateAccountInput, UpdateAccountInput,
    },
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequest {
    account_name: String,
    self_player_name: String,
    self_platform: String,
    pubg_api_key: String,
    set_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountRequest {
    account_name: String,
    self_player_name: String,
    self_platform: String,
    pubg_api_key: String,
}

#[tauri::command]
pub fn accounts_get_all(state: State<'_, AppState>) -> Result<Vec<AccountDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    AccountsRepository::new(&connection)
        .get_all()
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn accounts_get_active(state: State<'_, AppState>) -> Result<Option<AccountDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    AccountsRepository::new(&connection)
        .get_active()
        .map_err(|error: AppError| error.into())
}

#[tauri::command(rename_all = "camelCase")]
pub fn accounts_create(
    state: State<'_, AppState>,
    input: CreateAccountRequest,
) -> Result<AccountDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    AccountsRepository::new(&connection)
        .create(CreateAccountInput {
            account_name: input.account_name,
            self_player_name: input.self_player_name,
            self_platform: input.self_platform,
            pubg_api_key: input.pubg_api_key,
            set_active: input.set_active.unwrap_or(true),
        })
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn accounts_switch(state: State<'_, AppState>, id: i64) -> Result<AccountDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    AccountsRepository::new(&connection)
        .switch_active(id)
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn accounts_logout(state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    AccountsRepository::new(&connection)
        .logout()
        .map_err(|error: AppError| error.into())
}

#[tauri::command(rename_all = "camelCase")]
pub fn accounts_update_active(
    state: State<'_, AppState>,
    input: UpdateAccountRequest,
) -> Result<AccountDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    AccountsRepository::new(&connection)
        .update_active(UpdateAccountInput {
            account_name: input.account_name,
            self_player_name: input.self_player_name,
            self_platform: input.self_platform,
            pubg_api_key: input.pubg_api_key,
        })
        .map_err(|error: AppError| error.into())
}
