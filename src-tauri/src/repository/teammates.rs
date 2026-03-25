use rusqlite::{params, params_from_iter, Connection};
use serde::Serialize;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeammateDto {
    pub id: i64,
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
}

impl<'a> TeammatesRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn get_all(&self) -> Result<Vec<TeammateDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, platform, pubg_account_id, pubg_player_name, display_nickname, 
             is_points_enabled, total_points, last_seen_at, created_at, updated_at 
             FROM teammates ORDER BY pubg_player_name",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(TeammateDto {
                id: row.get(0)?,
                platform: row.get(1)?,
                pubg_account_id: row.get(2)?,
                pubg_player_name: row.get(3)?,
                display_nickname: row.get(4)?,
                is_points_enabled: row.get::<_, i64>(5)? == 1,
                total_points: row.get(6)?,
                last_seen_at: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?;

        let teammates = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(teammates)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<TeammateDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, platform, pubg_account_id, pubg_player_name, display_nickname, 
             is_points_enabled, total_points, last_seen_at, created_at, updated_at 
             FROM teammates WHERE id = ?1",
            [id],
            |row| {
                Ok(TeammateDto {
                    id: row.get(0)?,
                    platform: row.get(1)?,
                    pubg_account_id: row.get(2)?,
                    pubg_player_name: row.get(3)?,
                    display_nickname: row.get(4)?,
                    is_points_enabled: row.get::<_, i64>(5)? == 1,
                    total_points: row.get(6)?,
                    last_seen_at: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
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
            "SELECT id, platform, pubg_account_id, pubg_player_name, display_nickname,
              is_points_enabled, total_points, last_seen_at, created_at, updated_at
             FROM teammates
             WHERE platform = ?1 AND pubg_account_id = ?2
             LIMIT 1",
            params![platform, pubg_account_id],
            |row| {
                Ok(TeammateDto {
                    id: row.get(0)?,
                    platform: row.get(1)?,
                    pubg_account_id: row.get(2)?,
                    pubg_player_name: row.get(3)?,
                    display_nickname: row.get(4)?,
                    is_points_enabled: row.get::<_, i64>(5)? == 1,
                    total_points: row.get(6)?,
                    last_seen_at: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
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
            "SELECT id, platform, pubg_account_id, pubg_player_name, display_nickname,
              is_points_enabled, total_points, last_seen_at, created_at, updated_at
             FROM teammates
             WHERE platform = ?1 AND lower(pubg_player_name) = lower(?2)
             LIMIT 1",
            params![platform, pubg_player_name],
            |row| {
                Ok(TeammateDto {
                    id: row.get(0)?,
                    platform: row.get(1)?,
                    pubg_account_id: row.get(2)?,
                    pubg_player_name: row.get(3)?,
                    display_nickname: row.get(4)?,
                    is_points_enabled: row.get::<_, i64>(5)? == 1,
                    total_points: row.get(6)?,
                    last_seen_at: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
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
             (platform, pubg_account_id, pubg_player_name, display_nickname, is_points_enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![
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
        params.push(input.id.into());

        let sql = format!("UPDATE teammates SET {} WHERE id = ?", sets.join(", "));
        self.connection.execute(&sql, params_from_iter(params))?;

        self.get_by_id(input.id)?
            .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows))
    }

    pub fn update_last_seen(&self, id: i64) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE teammates
             SET last_seen_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1",
            [id],
        )?;

        Ok(())
    }

    pub fn update_total_points(&self, id: i64, total_points: i64) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE teammates
             SET total_points = ?1, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?2",
            params![total_points, id],
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
                         WHERE id = ?2",
                        params![account_id, teammate.id],
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
}
