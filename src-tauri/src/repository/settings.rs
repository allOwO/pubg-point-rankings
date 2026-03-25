use rusqlite::Connection;
use serde::Serialize;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingDto {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

pub struct SettingsRepository<'a> {
    connection: &'a Connection,
}

impl<'a> SettingsRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn get_all(&self) -> Result<Vec<AppSettingDto>, AppError> {
        let mut statement = self
            .connection
            .prepare("SELECT key, value, updated_at FROM app_settings ORDER BY key")?;

        let rows = statement.query_map([], |row| {
            Ok(AppSettingDto {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })?;

        let settings = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(settings)
    }

    pub fn get(&self, key: &str) -> Result<Option<AppSettingDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT key, value, updated_at FROM app_settings WHERE key = ?1",
            [key],
            |row| {
                Ok(AppSettingDto {
                    key: row.get(0)?,
                    value: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            },
        );

        match result {
            Ok(setting) => Ok(Some(setting)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn get_string(&self, key: &str, default_value: &str) -> Result<String, AppError> {
        let result = self.connection.query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(value) => Ok(value),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(default_value.to_string()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn get_bool(&self, key: &str, default_value: bool) -> Result<bool, AppError> {
        let value = self.get_string(key, if default_value { "1" } else { "0" })?;
        let normalized = value.trim().to_ascii_lowercase();

        Ok(match normalized.as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default_value,
        })
    }

    pub fn get_u64_in_range(
        &self,
        key: &str,
        default_value: u64,
        min_value: u64,
        max_value: u64,
    ) -> Result<u64, AppError> {
        let default_text = default_value.to_string();
        let value = self.get_string(key, &default_text)?;
        let parsed = value.trim().parse::<u64>().ok();

        let bounded = parsed.unwrap_or(default_value).clamp(min_value, max_value);

        Ok(bounded)
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), AppError> {
        self.connection.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            [key, value],
        )?;
        Ok(())
    }
}
