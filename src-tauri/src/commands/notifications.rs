use serde::Deserialize;
use tauri::State;

use crate::{
    app_state::AppState,
    error::{AppError, ErrorPayload},
    repository::notification_tasks::NotificationFailedTaskDto,
    services::{
        logs::{self, LogLevel},
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

    let _ = logs::write_log_record(
        &connection,
        LogLevel::Info,
        "notifications",
        "notification runtime install requested",
    );

    match napcat_runtime::install_runtime_and_get_status(&connection) {
        Ok(status) => {
            let _ = logs::write_log_record(
                &connection,
                LogLevel::Info,
                "notifications",
                "notification runtime install completed",
            );
            Ok(status)
        }
        Err(error) => {
            let _ = logs::write_log_record(
                &connection,
                LogLevel::Error,
                "notifications",
                &format!("notification runtime install failed: {error}"),
            );
            Err(error.into())
        }
    }
}

#[tauri::command]
pub fn notifications_start_runtime(
    state: State<'_, AppState>,
) -> Result<NotificationPageStatusDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    let _ = logs::write_log_record(
        &connection,
        LogLevel::Info,
        "notifications",
        "notification runtime start requested",
    );

    match napcat_runtime::start_runtime_and_get_status(&connection) {
        Ok(status) => {
            let _ = logs::write_log_record(
                &connection,
                LogLevel::Info,
                "notifications",
                "notification runtime start completed",
            );
            Ok(status)
        }
        Err(error) => {
            let _ = logs::write_log_record(
                &connection,
                LogLevel::Error,
                "notifications",
                &format!("notification runtime start failed: {error}"),
            );
            Err(error.into())
        }
    }
}

#[tauri::command]
pub fn notifications_stop_runtime(state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    let _ = logs::write_log_record(
        &connection,
        LogLevel::Info,
        "notifications",
        "notification runtime stop requested",
    );

    match napcat_runtime::stop_runtime_for_account(&connection) {
        Ok(()) => {
            let _ = logs::write_log_record(
                &connection,
                LogLevel::Info,
                "notifications",
                "notification runtime stop completed",
            );
            Ok(())
        }
        Err(error) => {
            let _ = logs::write_log_record(
                &connection,
                LogLevel::Error,
                "notifications",
                &format!("notification runtime stop failed: {error}"),
            );
            Err(error.into())
        }
    }
}

#[tauri::command]
pub fn notifications_restart_runtime(
    state: State<'_, AppState>,
) -> Result<NotificationPageStatusDto, ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    let _ = logs::write_log_record(
        &connection,
        LogLevel::Info,
        "notifications",
        "notification runtime restart requested",
    );

    match napcat_runtime::restart_runtime_and_get_status(&connection) {
        Ok(status) => {
            let _ = logs::write_log_record(
                &connection,
                LogLevel::Info,
                "notifications",
                "notification runtime restart completed",
            );
            Ok(status)
        }
        Err(error) => {
            let _ = logs::write_log_record(
                &connection,
                LogLevel::Error,
                "notifications",
                &format!("notification runtime restart failed: {error}"),
            );
            Err(error.into())
        }
    }
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

    let _ = logs::write_log_record(
        &connection,
        LogLevel::Info,
        "notifications",
        &format!(
            "resend selected notifications requested (count={})",
            input.task_ids.len()
        ),
    );

    match notifications::resend_selected_notifications(&connection, &input.task_ids) {
        Ok(result) => {
            let _ = logs::write_log_record(
                &connection,
                LogLevel::Info,
                "notifications",
                &format!(
                    "resend selected notifications completed (sent={}, failed={})",
                    result.sent_ids.len(),
                    result.failed_ids.len()
                ),
            );
            Ok(result)
        }
        Err(error) => {
            let _ = logs::write_log_record(
                &connection,
                LogLevel::Error,
                "notifications",
                &format!("resend selected notifications failed: {error}"),
            );
            Err(error.into())
        }
    }
}

#[tauri::command]
pub fn notifications_delete_failed_task(
    state: State<'_, AppState>,
    input: DeleteFailedNotificationInput,
) -> Result<(), ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    match notifications::delete_failed_notification(&connection, input.task_id) {
        Ok(()) => {
            let _ = logs::write_log_record(
                &connection,
                LogLevel::Info,
                "notifications",
                &format!("deleted failed notification task {}", input.task_id),
            );
            Ok(())
        }
        Err(error) => {
            let _ = logs::write_log_record(
                &connection,
                LogLevel::Error,
                "notifications",
                &format!(
                    "delete failed notification task {} failed: {error}",
                    input.task_id
                ),
            );
            Err(error.into())
        }
    }
}

#[tauri::command]
pub fn notifications_send_test(state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let connection = state.db.lock().map_err(|_| ErrorPayload {
        message: "database mutex is poisoned".to_string(),
    })?;

    let _ = logs::write_log_record(
        &connection,
        LogLevel::Info,
        "notifications",
        "notification test send requested",
    );

    match notifications::send_test_notification(&connection) {
        Ok(()) => {
            let _ = logs::write_log_record(
                &connection,
                LogLevel::Info,
                "notifications",
                "notification test send completed",
            );
            Ok(())
        }
        Err(error) => {
            let _ = logs::write_log_record(
                &connection,
                LogLevel::Error,
                "notifications",
                &format!("notification test send failed: {error}"),
            );
            Err(error.into())
        }
    }
}
