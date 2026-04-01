pub mod app_state;
pub mod commands;
pub mod db;
pub mod dto;
pub mod engine;
pub mod error;
pub mod parser;
pub mod platform;
pub mod pubg;
pub mod repository;
pub mod runtime;
pub mod services;

use tauri::Manager;

pub fn run() {
    let state = app_state::AppState::new().expect("failed to initialize application state");

    tauri::Builder::default()
        .manage(state)
        .setup(|app| {
            let app_state = app.state::<app_state::AppState>();
            runtime::scheduler::start_background_scheduler_with_sync(
                app_state.game_process_runtime.clone(),
                Some(app_state.db.clone()),
                Some(app_state.sync_runtime_status.clone()),
            );

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::accounts::accounts_create,
            commands::accounts::accounts_get_active,
            commands::accounts::accounts_get_all,
            commands::accounts::accounts_logout,
            commands::accounts::accounts_switch,
            commands::accounts::accounts_update_active,
            commands::app::app_get_status,
            commands::app::app_get_game_process_status,
            commands::logs::logs_get_recent,
            commands::logs::logs_get_status,
            commands::logs::logs_open_directory,
            commands::logs::logs_update_settings,
            commands::matches::matches_get_all,
            commands::matches::matches_get_by_id,
            commands::matches::matches_get_detail,
            commands::matches::matches_get_players,
            commands::notifications::notifications_get_status,
            commands::notifications::notifications_install_runtime,
            commands::notifications::notifications_start_runtime,
            commands::notifications::notifications_stop_runtime,
            commands::notifications::notifications_restart_runtime,
            commands::notifications::notifications_open_webui_info,
            commands::notifications::notifications_get_failed_tasks,
            commands::notifications::notifications_send_selected,
            commands::notifications::notifications_delete_failed_task,
            commands::notifications::notifications_send_test,
            commands::notifications::notifications_save_group_id,
            commands::notifications::notifications_get_template_config,
            commands::notifications::notifications_save_template_config,
            commands::points::points_get_all,
            commands::points::points_get_by_match,
            commands::points::points_get_history_groups,
            commands::points::points_recalculate_unsettled,
            commands::points::points_get_unsettled_summary,
            commands::points::points_settle_through_match,
            commands::points::points_update_match_note,
            commands::rules::rules_activate,
            commands::rules::rules_create,
            commands::rules::rules_delete,
            commands::rules::rules_get_active,
            commands::rules::rules_get_all,
            commands::rules::rules_update,
            commands::settings::settings_get,
            commands::settings::settings_get_all,
            commands::settings::settings_set,
            commands::sync::sync_get_status,
            commands::sync::sync_start,
            commands::sync::sync_start_match,
            commands::teammates::teammates_create,
            commands::teammates::teammates_delete,
            commands::teammates::teammates_get_all,
            commands::teammates::teammates_get_by_id,
            commands::teammates::teammates_get_history,
            commands::teammates::teammates_get_recent_candidates,
            commands::teammates::teammates_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
