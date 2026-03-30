use serde::Deserialize;
use tauri::State;

use crate::{
    app_state::AppState,
    error::{AppError, ErrorPayload},
    repository::notification_tasks::NotificationFailedTaskDto,
    services::{
        napcat_runtime::{self, NapCatWebUiInfoDto, NotificationPageStatusDto},
        notifications::{self, SendSelectedResultDto},
    },
};

#[derive(Debug, Deserialize)]
pub struct SendSelectedNotificationsInput {
    #[serde(rename = "taskIds")]
    task_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteFailedNotificationInput {
    #[serde(rename = "taskId")]
    task_id: i64,
}

#[tauri::command]
pub fn notifications_get_status(
    state: State<'_, AppState>,
) -> Result<NotificationPageStatusDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    napcat_runtime::get_notification_status(&connection)
        .map_err(|error: AppError| -> ErrorPayload { error.into() })
}

#[tauri::command]
pub fn notifications_install_runtime(
    state: State<'_, AppState>,
) -> Result<NotificationPageStatusDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    napcat_runtime::install_runtime_and_get_status(&connection)
        .map_err(|error: AppError| -> ErrorPayload { error.into() })
}

#[tauri::command]
pub fn notifications_start_runtime(
    state: State<'_, AppState>,
) -> Result<NotificationPageStatusDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    napcat_runtime::start_runtime_and_get_status(&connection)
        .map_err(|error: AppError| -> ErrorPayload { error.into() })
}

#[tauri::command]
pub fn notifications_stop_runtime(state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    napcat_runtime::stop_runtime_for_account(&connection)
        .map_err(|error: AppError| -> ErrorPayload { error.into() })
}

#[tauri::command]
pub fn notifications_restart_runtime(
    state: State<'_, AppState>,
) -> Result<NotificationPageStatusDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    napcat_runtime::restart_runtime_and_get_status(&connection)
        .map_err(|error: AppError| -> ErrorPayload { error.into() })
}

#[tauri::command]
pub fn notifications_open_webui_info(
    state: State<'_, AppState>,
) -> Result<NapCatWebUiInfoDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    napcat_runtime::open_webui_info(&connection)
        .map_err(|error: AppError| -> ErrorPayload { error.into() })
}

#[tauri::command]
pub fn notifications_get_failed_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<NotificationFailedTaskDto>, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    notifications::get_failed_notifications(&connection)
        .map_err(|error: AppError| -> ErrorPayload { error.into() })
}

#[tauri::command]
pub fn notifications_send_selected(
    state: State<'_, AppState>,
    input: SendSelectedNotificationsInput,
) -> Result<SendSelectedResultDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    notifications::resend_selected_notifications(&connection, &input.task_ids)
        .map_err(|error: AppError| -> ErrorPayload { error.into() })
}

#[tauri::command]
pub fn notifications_delete_failed_task(
    state: State<'_, AppState>,
    input: DeleteFailedNotificationInput,
) -> Result<(), ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    notifications::delete_failed_notification(&connection, input.task_id)
        .map_err(|error: AppError| -> ErrorPayload { error.into() })
}

#[tauri::command]
pub fn notifications_send_test(state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    notifications::send_test_notification(&connection)
        .map_err(|error: AppError| -> ErrorPayload { error.into() })
}
