use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::Command,
};

use chrono::Local;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{error::AppError, repository::settings::SettingsRepository};

const LOGGING_ENABLED_KEY: &str = "logging_enabled";
const LOGGING_DIRECTORY_KEY: &str = "logging_directory";
const DEFAULT_LOGGING_ENABLED: bool = true;
const MAX_LOG_FILES_TO_READ: usize = 3;
const MAX_LOG_FILE_SIZE_BYTES: u64 = 10 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntryDto {
    pub timestamp: String,
    pub level: String,
    pub source: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogStatusDto {
    pub enabled: bool,
    pub directory: String,
    pub log_file_path: Option<String>,
}

pub fn default_log_directory_from_executable(executable_path: &Path) -> Result<PathBuf, AppError> {
    let Some(parent) = executable_path.parent() else {
        return Err(AppError::Message(
            "failed to resolve executable parent for log directory".to_string(),
        ));
    };

    Ok(parent.join("logs"))
}

pub fn resolve_default_log_directory() -> Result<PathBuf, AppError> {
    let executable_path = std::env::current_exe().map_err(|error| {
        AppError::Message(format!(
            "failed to resolve current executable for log directory: {error}"
        ))
    })?;
    default_log_directory_from_executable(&executable_path)
}

pub fn get_log_status(connection: &Connection) -> Result<LogStatusDto, AppError> {
    let settings = SettingsRepository::new(connection);
    let enabled = settings.get_bool(LOGGING_ENABLED_KEY, DEFAULT_LOGGING_ENABLED)?;
    let directory = resolve_log_directory(&settings)?;
    let log_file_path = current_log_file_path(&directory);

    Ok(LogStatusDto {
        enabled,
        directory: directory.to_string_lossy().to_string(),
        log_file_path: log_file_path
            .exists()
            .then(|| log_file_path.to_string_lossy().to_string()),
    })
}

pub fn read_recent_log_entries(
    connection: &Connection,
    limit: usize,
) -> Result<Vec<LogEntryDto>, AppError> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    let settings = SettingsRepository::new(connection);
    let directory = resolve_log_directory(&settings)?;
    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut files = fs::read_dir(&directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("log"))
        .collect::<Vec<_>>();
    files.sort_by(|left, right| right.cmp(left));

    let mut entries = Vec::new();
    for file_path in files.into_iter().take(MAX_LOG_FILES_TO_READ) {
        entries.extend(read_log_entries_from_file(&file_path)?);
        if entries.len() >= limit {
            break;
        }
    }

    entries.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));
    entries.truncate(limit);
    Ok(entries)
}

pub fn write_log_record(
    connection: &Connection,
    level: LogLevel,
    source: &str,
    message: &str,
) -> Result<(), AppError> {
    let settings = SettingsRepository::new(connection);
    if !settings.get_bool(LOGGING_ENABLED_KEY, DEFAULT_LOGGING_ENABLED)? {
        return Ok(());
    }

    let directory = resolve_log_directory(&settings)?;
    fs::create_dir_all(&directory)?;

    let file_path = current_log_file_path(&directory);
    if file_path.exists() {
        let metadata = fs::metadata(&file_path)?;
        if metadata.len() >= MAX_LOG_FILE_SIZE_BYTES {
            fs::write(&file_path, [])?;
        }
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .map_err(|error| {
            AppError::Message(format!(
                "failed to open log file {}: {error}",
                file_path.display()
            ))
        })?;

    let record = LogEntryDto {
        timestamp: Local::now().to_rfc3339(),
        level: level.as_str().to_string(),
        source: source.trim().to_string(),
        message: message.trim().to_string(),
    };
    let line = serde_json::to_string(&record)
        .map_err(|error| AppError::Message(format!("failed to serialize log record: {error}")))?;

    writeln!(file, "{line}").map_err(|error| {
        AppError::Message(format!(
            "failed to write log file {}: {error}",
            file_path.display()
        ))
    })?;

    Ok(())
}

pub fn update_log_settings(
    connection: &Connection,
    enabled: bool,
    directory: &str,
) -> Result<LogStatusDto, AppError> {
    let resolved_directory = if directory.trim().is_empty() {
        resolve_default_log_directory()?
    } else {
        PathBuf::from(directory.trim())
    };
    fs::create_dir_all(&resolved_directory)?;

    let settings = SettingsRepository::new(connection);
    settings.set(LOGGING_ENABLED_KEY, if enabled { "1" } else { "0" })?;
    settings.set(
        LOGGING_DIRECTORY_KEY,
        resolved_directory.to_string_lossy().as_ref(),
    )?;

    if enabled {
        let _ = write_log_record(
            connection,
            LogLevel::Info,
            "settings",
            &format!(
                "logging settings updated (enabled={}, directory={})",
                enabled,
                resolved_directory.display()
            ),
        );
    }

    Ok(LogStatusDto {
        enabled,
        directory: resolved_directory.to_string_lossy().to_string(),
        log_file_path: current_log_file_path(&resolved_directory)
            .exists()
            .then(|| {
                current_log_file_path(&resolved_directory)
                    .to_string_lossy()
                    .to_string()
            }),
    })
}

pub fn open_log_directory(connection: &Connection) -> Result<(), AppError> {
    let settings = SettingsRepository::new(connection);
    let directory = resolve_log_directory(&settings)?;
    fs::create_dir_all(&directory)?;

    let mut command = if cfg!(target_os = "macos") {
        Command::new("open")
    } else if cfg!(target_os = "windows") {
        Command::new("explorer")
    } else {
        Command::new("xdg-open")
    };

    let status = command.arg(&directory).status().map_err(|error| {
        AppError::Message(format!(
            "failed to open log directory {}: {error}",
            directory.display()
        ))
    })?;

    if !status.success() {
        return Err(AppError::Message(format!(
            "failed to open log directory {}",
            directory.display()
        )));
    }

    Ok(())
}

fn resolve_log_directory(settings: &SettingsRepository<'_>) -> Result<PathBuf, AppError> {
    let configured = settings.get_string(LOGGING_DIRECTORY_KEY, "")?;
    let trimmed = configured.trim();
    if trimmed.is_empty() {
        return resolve_default_log_directory();
    }

    Ok(PathBuf::from(trimmed))
}

fn current_log_file_path(directory: &Path) -> PathBuf {
    let file_name = format!("app-{}.log", Local::now().format("%Y-%m-%d"));
    directory.join(file_name)
}

fn read_log_entries_from_file(file_path: &Path) -> Result<Vec<LogEntryDto>, AppError> {
    let file = File::open(file_path).map_err(|error| {
        AppError::Message(format!(
            "failed to open log file {}: {error}",
            file_path.display()
        ))
    })?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|error| {
            AppError::Message(format!(
                "failed to read log file {}: {error}",
                file_path.display()
            ))
        })?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<LogEntryDto>(trimmed) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use rusqlite::Connection;

    use super::{
        default_log_directory_from_executable, get_log_status, read_recent_log_entries,
        update_log_settings, write_log_record, LogLevel,
    };
    use crate::{db::migrations::bootstrap_database, repository::settings::SettingsRepository};

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let unique = format!(
            "{}-{}",
            prefix,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn seed_logging_settings(connection: &Connection, directory: &Path, enabled: bool) {
        bootstrap_database(connection).expect("bootstrap db");
        let settings = SettingsRepository::new(connection);
        settings
            .set("logging_enabled", if enabled { "1" } else { "0" })
            .expect("set logging enabled");
        settings
            .set("logging_directory", directory.to_string_lossy().as_ref())
            .expect("set logging directory");
    }

    #[test]
    fn default_log_directory_uses_executable_parent_logs_folder() {
        let executable_path =
            PathBuf::from("/Applications/PUBG Point Rankings.app/Contents/MacOS/pubg");

        let directory = default_log_directory_from_executable(&executable_path)
            .expect("resolve default log directory");

        assert_eq!(
            directory,
            PathBuf::from("/Applications/PUBG Point Rankings.app/Contents/MacOS/logs")
        );
    }

    #[test]
    fn write_log_record_persists_entries_and_reads_them_back_newest_first() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        let log_dir = make_temp_dir("logs-service-write");
        seed_logging_settings(&connection, &log_dir, true);

        write_log_record(&connection, LogLevel::Info, "sync", "sync started")
            .expect("write first record");
        write_log_record(&connection, LogLevel::Error, "notifications", "send failed")
            .expect("write second record");

        let status = get_log_status(&connection).expect("read status");
        assert!(status.enabled);
        assert_eq!(status.directory, log_dir.to_string_lossy());
        assert!(status.log_file_path.is_some());

        let entries = read_recent_log_entries(&connection, 10).expect("read log entries");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "ERROR");
        assert_eq!(entries[0].source, "notifications");
        assert_eq!(entries[0].message, "send failed");
        assert_eq!(entries[1].level, "INFO");
        assert_eq!(entries[1].source, "sync");
        assert_eq!(entries[1].message, "sync started");

        let _ = fs::remove_dir_all(log_dir);
    }

    #[test]
    fn write_log_record_skips_files_when_logging_disabled() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        let log_dir = make_temp_dir("logs-service-disabled");
        seed_logging_settings(&connection, &log_dir, false);

        write_log_record(&connection, LogLevel::Warn, "app", "disabled test")
            .expect("skip write while disabled");

        let entries =
            read_recent_log_entries(&connection, 10).expect("read entries while disabled");
        assert!(entries.is_empty());

        let directory_entries = fs::read_dir(&log_dir).expect("read log directory").count();
        assert_eq!(directory_entries, 0);

        let _ = fs::remove_dir_all(log_dir);
    }

    #[test]
    fn update_log_settings_trims_and_creates_directory() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        bootstrap_database(&connection).expect("bootstrap db");
        let log_dir = make_temp_dir("logs-settings-update").join("nested-logs");

        update_log_settings(
            &connection,
            true,
            &format!("  {}  ", log_dir.to_string_lossy()),
        )
        .expect("update log settings");

        let settings = SettingsRepository::new(&connection);
        let stored_directory = settings
            .get_string("logging_directory", "")
            .expect("read stored directory");
        let stored_enabled = settings
            .get_bool("logging_enabled", false)
            .expect("read stored enabled");

        assert_eq!(stored_directory, log_dir.to_string_lossy());
        assert!(stored_enabled);
        assert!(log_dir.exists());

        let _ = fs::remove_dir_all(log_dir.parent().expect("log dir parent"));
    }

    #[test]
    fn write_log_record_truncates_existing_file_after_ten_megabytes() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        let log_dir = make_temp_dir("logs-service-truncate");
        seed_logging_settings(&connection, &log_dir, true);

        let file_path = super::current_log_file_path(&log_dir);
        fs::write(
            &file_path,
            vec![b'x'; (super::MAX_LOG_FILE_SIZE_BYTES + 1) as usize],
        )
        .expect("seed oversized log file");

        write_log_record(&connection, LogLevel::Info, "sync", "fresh entry")
            .expect("write replacement record");

        let content = fs::read_to_string(&file_path).expect("read log file");
        assert!(content.contains("fresh entry"));
        assert!(!content.contains(&"x".repeat(32)));

        let metadata = fs::metadata(&file_path).expect("stat log file");
        assert!(metadata.len() < super::MAX_LOG_FILE_SIZE_BYTES);

        let _ = fs::remove_dir_all(log_dir);
    }
}
