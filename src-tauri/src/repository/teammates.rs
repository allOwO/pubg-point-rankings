use rusqlite::{params, params_from_iter, Connection};
use serde::Serialize;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeammateDto {
    pub id: i64,
    pub account_id: i64,
    pub platform: String,
    pub pubg_account_id: Option<String>,
    pub pubg_player_name: String,
    pub display_nickname: Option<String>,
    pub is_points_enabled: bool,
    pub total_points: i64,
    pub last_seen_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct CreateTeammateInput {
    pub platform: String,
    pub pubg_account_id: Option<String>,
    pub pubg_player_name: String,
    pub display_nickname: Option<String>,
    pub is_points_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateTeammateInput {
    pub id: i64,
    pub display_nickname: Option<String>,
    pub is_points_enabled: Option<bool>,
}

pub struct TeammatesRepository<'a> {
    connection: &'a Connection,
    account_id: i64,
}

impl<'a> TeammatesRepository<'a> {
    pub fn new(connection: &'a Connection, account_id: i64) -> Self {
        Self {
            connection,
            account_id,
        }
    }

    pub fn get_all(&self) -> Result<Vec<TeammateDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, platform, pubg_account_id, pubg_player_name, display_nickname,
              is_points_enabled, total_points, last_seen_at, created_at, updated_at
             FROM teammates WHERE account_id = ?1 ORDER BY pubg_player_name",
        )?;

        let rows = statement.query_map([self.account_id], |row| Self::map_row(row))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<TeammateDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, account_id, platform, pubg_account_id, pubg_player_name, display_nickname,
              is_points_enabled, total_points, last_seen_at, created_at, updated_at
             FROM teammates WHERE account_id = ?1 AND id = ?2",
            params![self.account_id, id],
            Self::map_row,
        );

        match result {
            Ok(teammate) => Ok(Some(teammate)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn get_by_account_id(
        &self,
        platform: &str,
        pubg_account_id: &str,
    ) -> Result<Option<TeammateDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, account_id, platform, pubg_account_id, pubg_player_name, display_nickname,
              is_points_enabled, total_points, last_seen_at, created_at, updated_at
             FROM teammates
             WHERE account_id = ?1 AND platform = ?2 AND pubg_account_id = ?3
             LIMIT 1",
            params![self.account_id, platform, pubg_account_id],
            Self::map_row,
        );

        match result {
            Ok(teammate) => Ok(Some(teammate)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn get_by_player_name(
        &self,
        platform: &str,
        pubg_player_name: &str,
    ) -> Result<Option<TeammateDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, account_id, platform, pubg_account_id, pubg_player_name, display_nickname,
              is_points_enabled, total_points, last_seen_at, created_at, updated_at
             FROM teammates
             WHERE account_id = ?1 AND platform = ?2 AND lower(pubg_player_name) = lower(?3)
             LIMIT 1",
            params![self.account_id, platform, pubg_player_name],
            Self::map_row,
        );

        match result {
            Ok(teammate) => Ok(Some(teammate)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn create(&self, input: CreateTeammateInput) -> Result<TeammateDto, AppError> {
        self.connection.execute(
            "INSERT INTO teammates
             (account_id, platform, pubg_account_id, pubg_player_name, display_nickname, is_points_enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![
                self.account_id,
                input.platform,
                input.pubg_account_id,
                input.pubg_player_name,
                input.display_nickname,
                if input.is_points_enabled { 1 } else { 0 },
            ],
        )?;

        let id = self.connection.last_insert_rowid();
        self.get_by_id(id)?
            .ok_or_else(|| AppError::Database(rusqlite::Error::ExecuteReturnedResults))
    }

    pub fn update(&self, input: UpdateTeammateInput) -> Result<TeammateDto, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(display_nickname) = input.display_nickname {
            sets.push("display_nickname = ?".to_string());
            params.push(display_nickname.into());
        }
        if let Some(is_points_enabled) = input.is_points_enabled {
            sets.push("is_points_enabled = ?".to_string());
            params.push((if is_points_enabled { 1 } else { 0 }).into());
        }

        if sets.is_empty() {
            return self
                .get_by_id(input.id)?
                .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows));
        }

        sets.push("updated_at = CURRENT_TIMESTAMP".to_string());
        params.push(self.account_id.into());
        params.push(input.id.into());

        let sql = format!(
            "UPDATE teammates SET {} WHERE account_id = ? AND id = ?",
            sets.join(", ")
        );
        self.connection.execute(&sql, params_from_iter(params))?;

        self.get_by_id(input.id)?
            .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows))
    }

    pub fn update_last_seen(&self, id: i64) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE teammates
             SET last_seen_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
             WHERE account_id = ?1 AND id = ?2",
            params![self.account_id, id],
        )?;

        Ok(())
    }

    pub fn update_total_points(&self, id: i64, total_points: i64) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE teammates
             SET total_points = ?1, updated_at = CURRENT_TIMESTAMP
             WHERE account_id = ?2 AND id = ?3",
            params![total_points, self.account_id, id],
        )?;

        Ok(())
    }

    pub fn find_or_create(
        &self,
        pubg_player_name: &str,
        platform: &str,
        pubg_account_id: Option<&str>,
    ) -> Result<TeammateDto, AppError> {
        if let Some(account_id) = pubg_account_id {
            if let Some(teammate) = self.get_by_account_id(platform, account_id)? {
                return Ok(teammate);
            }
        }

        if let Some(teammate) = self.get_by_player_name(platform, pubg_player_name)? {
            if let Some(account_id) = pubg_account_id {
                if teammate.pubg_account_id.is_none() {
                    self.connection.execute(
                        "UPDATE teammates
                         SET pubg_account_id = ?1, updated_at = CURRENT_TIMESTAMP
                         WHERE account_id = ?2 AND id = ?3",
                        params![account_id, self.account_id, teammate.id],
                    )?;
                    return self
                        .get_by_id(teammate.id)?
                        .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows));
                }
            }

            return Ok(teammate);
        }

        self.create(CreateTeammateInput {
            platform: platform.to_string(),
            pubg_account_id: pubg_account_id.map(ToOwned::to_owned),
            pubg_player_name: pubg_player_name.to_string(),
            display_nickname: None,
            is_points_enabled: true,
        })
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TeammateDto> {
        Ok(TeammateDto {
            id: row.get(0)?,
            account_id: row.get(1)?,
            platform: row.get(2)?,
            pubg_account_id: row.get(3)?,
            pubg_player_name: row.get(4)?,
            display_nickname: row.get(5)?,
            is_points_enabled: row.get::<_, i64>(6)? == 1,
            total_points: row.get(7)?,
            last_seen_at: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    }
}
