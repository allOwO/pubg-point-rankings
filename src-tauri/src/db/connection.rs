use std::path::PathBuf;

use rusqlite::Connection;

use crate::error::AppError;

pub fn resolve_database_path() -> Result<PathBuf, AppError> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "home directory is unavailable",
        )
    })?;

    let path = if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir.join("AppData").join("Local"))
            .join("pubg-point-rankings")
            .join("app.db")
    } else if cfg!(target_os = "macos") {
        home_dir
            .join("Library")
            .join("Application Support")
            .join("pubg-point-rankings")
            .join("app.db")
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir.join(".config"))
            .join("pubg-point-rankings")
            .join("app.db")
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    Ok(path)
}

pub fn open_database() -> Result<(Connection, PathBuf), AppError> {
    let path = resolve_database_path()?;
    let connection = Connection::open(&path)?;

    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "foreign_keys", "ON")?;

    Ok((connection, path))
}
