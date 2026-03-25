use std::time::Duration;

use rusqlite::Connection;

use crate::{error::AppError, repository::settings::SettingsRepository};

pub const KEY_AUTO_RECENT_MATCH_ENABLED: &str = "auto_recent_match_enabled";
pub const KEY_RUNNING_PROCESS_CHECK_INTERVAL_SECONDS: &str =
    "running_process_check_interval_seconds";
pub const KEY_NOT_RUNNING_PROCESS_CHECK_INTERVAL_SECONDS: &str =
    "not_running_process_check_interval_seconds";
pub const KEY_RUNNING_RECENT_MATCH_INTERVAL_SECONDS: &str = "running_recent_match_interval_seconds";
pub const KEY_COOLDOWN_POLLING_INTERVAL_SECONDS: &str = "cooldown_polling_interval_seconds";
pub const KEY_COOLDOWN_WINDOW_MINUTES: &str = "cooldown_window_minutes";
pub const KEY_RECENT_MATCH_RETRY_DELAY_SECONDS: &str = "recent_match_retry_delay_seconds";
pub const KEY_RECENT_MATCH_RETRY_LIMIT: &str = "recent_match_retry_limit";

#[derive(Debug, Clone)]
pub struct PollingConfig {
    pub auto_recent_match_enabled: bool,
    pub running_process_check_interval: Duration,
    pub not_running_process_check_interval: Duration,
    pub running_recent_match_interval: Duration,
    pub cooldown_polling_interval: Duration,
    pub cooldown_window: Duration,
    pub recent_match_retry_delay: Duration,
    pub recent_match_retry_limit: u64,
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            auto_recent_match_enabled: true,
            running_process_check_interval: Duration::from_secs(5),
            not_running_process_check_interval: Duration::from_secs(30),
            running_recent_match_interval: Duration::from_secs(30),
            cooldown_polling_interval: Duration::from_secs(120),
            cooldown_window: Duration::from_secs(40 * 60),
            recent_match_retry_delay: Duration::from_secs(15),
            recent_match_retry_limit: 2,
        }
    }
}

pub fn load_polling_config(connection: &Connection) -> Result<PollingConfig, AppError> {
    let settings = SettingsRepository::new(connection);
    let defaults = PollingConfig::default();

    let running_process_check_seconds = settings.get_u64_in_range(
        KEY_RUNNING_PROCESS_CHECK_INTERVAL_SECONDS,
        defaults.running_process_check_interval.as_secs(),
        1,
        300,
    )?;
    let not_running_process_check_seconds = settings.get_u64_in_range(
        KEY_NOT_RUNNING_PROCESS_CHECK_INTERVAL_SECONDS,
        defaults.not_running_process_check_interval.as_secs(),
        3,
        900,
    )?;
    let running_recent_match_seconds = settings.get_u64_in_range(
        KEY_RUNNING_RECENT_MATCH_INTERVAL_SECONDS,
        defaults.running_recent_match_interval.as_secs(),
        10,
        900,
    )?;
    let cooldown_polling_seconds = settings.get_u64_in_range(
        KEY_COOLDOWN_POLLING_INTERVAL_SECONDS,
        defaults.cooldown_polling_interval.as_secs(),
        10,
        1200,
    )?;
    let cooldown_window_minutes = settings.get_u64_in_range(
        KEY_COOLDOWN_WINDOW_MINUTES,
        defaults.cooldown_window.as_secs() / 60,
        1,
        180,
    )?;
    let recent_match_retry_delay_seconds = settings.get_u64_in_range(
        KEY_RECENT_MATCH_RETRY_DELAY_SECONDS,
        defaults.recent_match_retry_delay.as_secs(),
        1,
        300,
    )?;
    let recent_match_retry_limit = settings.get_u64_in_range(
        KEY_RECENT_MATCH_RETRY_LIMIT,
        defaults.recent_match_retry_limit,
        0,
        10,
    )?;

    Ok(PollingConfig {
        auto_recent_match_enabled: settings.get_bool(
            KEY_AUTO_RECENT_MATCH_ENABLED,
            defaults.auto_recent_match_enabled,
        )?,
        running_process_check_interval: Duration::from_secs(running_process_check_seconds),
        not_running_process_check_interval: Duration::from_secs(not_running_process_check_seconds),
        running_recent_match_interval: Duration::from_secs(running_recent_match_seconds),
        cooldown_polling_interval: Duration::from_secs(cooldown_polling_seconds),
        cooldown_window: Duration::from_secs(cooldown_window_minutes.saturating_mul(60)),
        recent_match_retry_delay: Duration::from_secs(recent_match_retry_delay_seconds),
        recent_match_retry_limit,
    })
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::{
        db::{migrations::bootstrap_database, schema::DEFAULT_DATA_SQL},
        repository::settings::SettingsRepository,
    };

    use super::{load_polling_config, KEY_RUNNING_PROCESS_CHECK_INTERVAL_SECONDS};

    #[test]
    fn invalid_numbers_fall_back_to_defaults() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        connection
            .execute_batch(crate::db::schema::INITIAL_SCHEMA_SQL)
            .expect("create schema");
        connection
            .execute_batch(DEFAULT_DATA_SQL)
            .expect("insert defaults");
        bootstrap_database(&connection).expect("bootstrap");

        SettingsRepository::new(&connection)
            .set(KEY_RUNNING_PROCESS_CHECK_INTERVAL_SECONDS, "not-a-number")
            .expect("set invalid config");

        let config = load_polling_config(&connection).expect("load config");
        assert_eq!(config.running_process_check_interval.as_secs(), 5);
    }
}
