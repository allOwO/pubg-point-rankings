use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use rusqlite::Connection;

use crate::{
    db::{connection::open_database, migrations::bootstrap_database},
    error::AppError,
    runtime::game_state::GameProcessRuntime,
    services::sync::SyncRuntimeStatus,
};

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub db_path: PathBuf,
    pub app_version: String,
    pub game_process_runtime: Arc<Mutex<GameProcessRuntime>>,
    pub sync_runtime_status: Arc<Mutex<SyncRuntimeStatus>>,
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        let (connection, db_path) = open_database()?;
        bootstrap_database(&connection)?;

        Ok(Self {
            db: Arc::new(Mutex::new(connection)),
            db_path,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            game_process_runtime: Arc::new(Mutex::new(GameProcessRuntime::default())),
            sync_runtime_status: Arc::new(Mutex::new(SyncRuntimeStatus::default())),
        })
    }
}
