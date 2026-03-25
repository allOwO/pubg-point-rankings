use serde::Deserialize;
use tauri::State;

use crate::{
    app_state::AppState,
    error::{AppError, ErrorPayload},
    repository::{
        accounts::AccountsRepository,
        rules::{CreatePointRuleInput, PointRulesRepository, UpdatePointRuleInput},
    },
};

#[derive(Debug, Deserialize)]
pub struct CreateRuleRequest {
    name: String,
    #[serde(rename = "damagePointsPerDamage")]
    damage_points_per_damage: i64,
    #[serde(rename = "killPoints")]
    kill_points: i64,
    #[serde(rename = "revivePoints")]
    revive_points: i64,
    #[serde(rename = "roundingMode")]
    rounding_mode: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRuleRequest {
    id: i64,
    name: Option<String>,
    #[serde(rename = "damagePointsPerDamage")]
    damage_points_per_damage: Option<i64>,
    #[serde(rename = "killPoints")]
    kill_points: Option<i64>,
    #[serde(rename = "revivePoints")]
    revive_points: Option<i64>,
    #[serde(rename = "roundingMode")]
    rounding_mode: Option<String>,
}

#[tauri::command]
pub fn rules_get_all(
    state: State<'_, AppState>,
) -> Result<Vec<crate::repository::rules::PointRuleDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    PointRulesRepository::new(&connection, account.id)
        .get_all()
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn rules_get_active(
    state: State<'_, AppState>,
) -> Result<Option<crate::repository::rules::PointRuleDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    PointRulesRepository::new(&connection, account.id)
        .get_active()
        .map_err(|error: AppError| error.into())
}

#[tauri::command(rename_all = "camelCase")]
pub fn rules_create(
    state: State<'_, AppState>,
    input: CreateRuleRequest,
) -> Result<crate::repository::rules::PointRuleDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    let create_input = CreatePointRuleInput {
        name: input.name,
        damage_points_per_damage: input.damage_points_per_damage,
        kill_points: input.kill_points,
        revive_points: input.revive_points,
        rounding_mode: input.rounding_mode,
    };

    PointRulesRepository::new(&connection, account.id)
        .create(create_input)
        .map_err(|error: AppError| error.into())
}

#[tauri::command(rename_all = "camelCase")]
pub fn rules_update(
    state: State<'_, AppState>,
    input: UpdateRuleRequest,
) -> Result<crate::repository::rules::PointRuleDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    let update_input = UpdatePointRuleInput {
        id: input.id,
        name: input.name,
        damage_points_per_damage: input.damage_points_per_damage,
        kill_points: input.kill_points,
        revive_points: input.revive_points,
        rounding_mode: input.rounding_mode,
    };

    PointRulesRepository::new(&connection, account.id)
        .update(update_input)
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn rules_delete(state: State<'_, AppState>, id: i64) -> Result<(), ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    PointRulesRepository::new(&connection, account.id)
        .delete(id)
        .map_err(|error: AppError| error.into())
}

#[tauri::command]
pub fn rules_activate(
    state: State<'_, AppState>,
    id: i64,
) -> Result<crate::repository::rules::PointRuleDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;
    let account = AccountsRepository::new(&connection)
        .require_active()
        .map_err(ErrorPayload::from)?;

    PointRulesRepository::new(&connection, account.id)
        .activate(id)
        .map_err(|error: AppError| error.into())
}
