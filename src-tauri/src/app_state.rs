use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use rusqlite::Connection;

use crate::{
    db::{connection::open_database, migrations::bootstrap_database},
    error::AppError,
    repository::{
        accounts::AccountsRepository, points::PointRecordsRepository, settings::SettingsRepository,
    },
    runtime::game_state::GameProcessRuntime,
    services::logs::{self, LogLevel},
    services::sync::{ManualSyncTaskStatus, SyncRuntimeStatus},
};

const POINTS_IDENTITY_KEY_REPAIR_FLAG: &str = "points_identity_key_repaired_v6";

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub db_path: PathBuf,
    pub app_version: String,
    pub game_process_runtime: Arc<Mutex<GameProcessRuntime>>,
    pub sync_runtime_status: Arc<Mutex<SyncRuntimeStatus>>,
    pub manual_sync_task_status: Arc<Mutex<ManualSyncTaskStatus>>,
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        let (connection, db_path) = open_database()?;
        bootstrap_database(&connection)?;
        ensure_active_account_point_history_repaired(&connection)?;

        let _ = logs::write_log_record(
            &connection,
            LogLevel::Info,
            "app",
            &format!(
                "application state initialized (version={}, database={})",
                env!("CARGO_PKG_VERSION"),
                db_path.display()
            ),
        );

        Ok(Self {
            db: Arc::new(Mutex::new(connection)),
            db_path,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            game_process_runtime: Arc::new(Mutex::new(GameProcessRuntime::default())),
            sync_runtime_status: Arc::new(Mutex::new(SyncRuntimeStatus::default())),
            manual_sync_task_status: Arc::new(Mutex::new(ManualSyncTaskStatus::default())),
        })
    }
}

pub fn ensure_active_account_point_history_repaired(
    connection: &Connection,
) -> Result<(), AppError> {
    let Some(account) = AccountsRepository::new(connection).get_active()? else {
        return Ok(());
    };

    ensure_account_point_history_repaired(connection, account.id, &account.self_player_name)
}

pub fn ensure_account_point_history_repaired(
    connection: &Connection,
    account_id: i64,
    self_player_name: &str,
) -> Result<(), AppError> {
    let settings = SettingsRepository::new(connection);
    let already_repaired =
        settings.get_account_string(account_id, POINTS_IDENTITY_KEY_REPAIR_FLAG, "0")?;

    if already_repaired == "1" {
        return Ok(());
    }

    PointRecordsRepository::new(connection, account_id)
        .repair_points_with_current_identities(self_player_name)?;
    settings.set_account(account_id, POINTS_IDENTITY_KEY_REPAIR_FLAG, "1")?;

    Ok(())
}
