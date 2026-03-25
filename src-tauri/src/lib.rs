pub mod app_state;
pub mod commands;
pub mod db;
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
            commands::app::app_get_status,
            commands::app::app_get_game_process_status,
            commands::matches::matches_get_all,
            commands::matches::matches_get_by_id,
            commands::matches::matches_get_players,
            commands::points::points_get_all,
            commands::points::points_get_by_match,
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
            commands::teammates::teammates_get_all,
            commands::teammates::teammates_get_by_id,
            commands::teammates::teammates_get_history,
            commands::teammates::teammates_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
