use rusqlite::{params, params_from_iter, Connection};
use serde::Serialize;
use std::collections::HashMap;

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
    pub is_friend: bool,
    pub is_points_enabled: bool,
    pub total_points: i64,
    pub last_seen_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentTeammateCandidateDto {
    pub platform: String,
    pub pubg_account_id: Option<String>,
    pub pubg_player_name: String,
    pub last_teammate_at: String,
    pub is_friend: bool,
}

#[derive(Debug)]
struct RecentTeammateCandidateRow {
    platform: String,
    pubg_account_id: Option<String>,
    pubg_player_name: String,
    last_teammate_at: String,
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
              is_friend, is_points_enabled, total_points, last_seen_at, created_at, updated_at
             FROM teammates
             WHERE account_id = ?1 AND is_friend = 1
             ORDER BY CASE WHEN last_seen_at IS NULL THEN 1 ELSE 0 END, last_seen_at DESC, pubg_player_name ASC",
        )?;

        let rows = statement.query_map([self.account_id], |row| Self::map_row(row))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<TeammateDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, account_id, platform, pubg_account_id, pubg_player_name, display_nickname,
              is_friend, is_points_enabled, total_points, last_seen_at, created_at, updated_at
             FROM teammates WHERE account_id = ?1 AND id = ?2 AND is_friend = 1",
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
              is_friend, is_points_enabled, total_points, last_seen_at, created_at, updated_at
             FROM teammates
             WHERE account_id = ?1 AND platform = ?2 AND pubg_account_id = ?3 AND is_friend = 1
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
              is_friend, is_points_enabled, total_points, last_seen_at, created_at, updated_at
             FROM teammates
             WHERE account_id = ?1 AND platform = ?2 AND lower(pubg_player_name) = lower(?3) AND is_friend = 1
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
        if let Some(existing) = self.find_existing(
            &input.platform,
            &input.pubg_player_name,
            input.pubg_account_id.as_deref(),
        )? {
            self.connection.execute(
                "UPDATE teammates
                 SET is_friend = 1,
                     is_points_enabled = ?1,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE account_id = ?2 AND id = ?3",
                params![
                    if input.is_points_enabled { 1 } else { 0 },
                    self.account_id,
                    existing.id
                ],
            )?;

            return self
                .get_by_id(existing.id)?
                .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows));
        }

        self.connection.execute(
            "INSERT INTO teammates
             (account_id, platform, pubg_account_id, pubg_player_name, display_nickname, is_friend, is_points_enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
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
        self.backfill_last_seen_for_friend(
            id,
            &input.pubg_player_name,
            input.pubg_account_id.as_deref(),
        )?;
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

    pub fn delete(&self, id: i64) -> Result<(), AppError> {
        self.connection.execute(
            "DELETE FROM teammates WHERE account_id = ?1 AND id = ?2",
            params![self.account_id, id],
        )?;

        Ok(())
    }

    pub fn get_recent_candidates(
        &self,
        recent_match_limit: usize,
    ) -> Result<Vec<RecentTeammateCandidateDto>, AppError> {
        let mut statement = self.connection.prepare(
            r#"
            WITH recent_matches AS (
              SELECT match_id, platform, played_at
              FROM matches
              WHERE account_id = ?1
              ORDER BY played_at DESC, match_id DESC
              LIMIT ?2
            )
            SELECT
              rm.platform,
              mp.pubg_account_id,
              mp.pubg_player_name,
              rm.played_at
            FROM recent_matches rm
            INNER JOIN match_players self_mp
              ON self_mp.account_id = ?1
             AND self_mp.match_id = rm.match_id
             AND self_mp.is_self = 1
             AND self_mp.team_id IS NOT NULL
            INNER JOIN match_players mp
              ON mp.account_id = ?1
             AND mp.match_id = rm.match_id
            WHERE mp.is_self = 0
              AND mp.team_id = self_mp.team_id
            ORDER BY rm.played_at DESC, rm.match_id DESC, mp.id ASC
            "#,
        )?;

        let rows =
            statement.query_map(params![self.account_id, recent_match_limit as i64], |row| {
                Ok(RecentTeammateCandidateRow {
                    platform: row.get(0)?,
                    pubg_account_id: row.get(1)?,
                    pubg_player_name: row.get(2)?,
                    last_teammate_at: row.get(3)?,
                })
            })?;

        let mut candidates_by_key: HashMap<String, RecentTeammateCandidateDto> = HashMap::new();

        for row in rows {
            let row = row?;
            let existing_friend = match row.pubg_account_id.as_deref() {
                Some(account_id) if !account_id.trim().is_empty() => {
                    match self.get_by_account_id(&row.platform, account_id)? {
                        Some(friend) => Some(friend),
                        None => self.get_by_player_name(&row.platform, &row.pubg_player_name)?,
                    }
                }
                _ => self.get_by_player_name(&row.platform, &row.pubg_player_name)?,
            };

            if existing_friend.is_some() {
                continue;
            }

            let dedupe_key = match row.pubg_account_id.as_deref() {
                Some(account_id) if !account_id.trim().is_empty() => {
                    format!("account:{account_id}")
                }
                _ => format!("name:{}", row.pubg_player_name.to_ascii_lowercase()),
            };

            let candidate = RecentTeammateCandidateDto {
                platform: row.platform,
                pubg_account_id: row.pubg_account_id,
                pubg_player_name: row.pubg_player_name,
                last_teammate_at: row.last_teammate_at,
                is_friend: false,
            };

            let should_replace = candidates_by_key
                .get(&dedupe_key)
                .is_none_or(|current| current.last_teammate_at < candidate.last_teammate_at);

            if should_replace {
                candidates_by_key.insert(dedupe_key, candidate);
            }
        }

        let mut candidates = candidates_by_key.into_values().collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right
                .last_teammate_at
                .cmp(&left.last_teammate_at)
                .then_with(|| left.pubg_player_name.cmp(&right.pubg_player_name))
        });

        Ok(candidates)
    }

    pub fn update_last_seen(&self, id: i64, last_seen_at: &str) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE teammates
             SET last_seen_at = ?1, updated_at = CURRENT_TIMESTAMP
             WHERE account_id = ?2 AND id = ?3
               AND (last_seen_at IS NULL OR last_seen_at < ?1)",
            params![last_seen_at, self.account_id, id],
        )?;

        Ok(())
    }

    pub fn set_last_seen(&self, id: i64, last_seen_at: &str) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE teammates
             SET last_seen_at = ?1, updated_at = CURRENT_TIMESTAMP
             WHERE account_id = ?2 AND id = ?3",
            params![last_seen_at, self.account_id, id],
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

    fn find_existing(
        &self,
        platform: &str,
        pubg_player_name: &str,
        pubg_account_id: Option<&str>,
    ) -> Result<Option<TeammateDto>, AppError> {
        if let Some(account_id) = pubg_account_id {
            if let Some(teammate) = self.get_by_account_id(platform, account_id)? {
                return Ok(Some(teammate));
            }
        }

        self.get_by_player_name(platform, pubg_player_name)
    }

    fn backfill_last_seen_for_friend(
        &self,
        id: i64,
        pubg_player_name: &str,
        pubg_account_id: Option<&str>,
    ) -> Result<(), AppError> {
        let last_seen_at = self.connection.query_row(
            r#"
            SELECT MAX(m.played_at)
            FROM matches m
            INNER JOIN match_players self_mp
              ON self_mp.account_id = m.account_id
             AND self_mp.match_id = m.match_id
             AND self_mp.is_self = 1
             AND self_mp.team_id IS NOT NULL
            INNER JOIN match_players mp
              ON mp.account_id = m.account_id
             AND mp.match_id = m.match_id
            WHERE m.account_id = ?1
              AND mp.is_self = 0
              AND mp.team_id = self_mp.team_id
              AND (
                (?2 IS NOT NULL AND trim(?2) <> '' AND mp.pubg_account_id = ?2)
                OR lower(mp.pubg_player_name) = lower(?3)
              )
            "#,
            params![self.account_id, pubg_account_id, pubg_player_name],
            |row| row.get::<_, Option<String>>(0),
        )?;

        if let Some(last_seen_at) = last_seen_at {
            self.set_last_seen(id, &last_seen_at)?;
        }

        Ok(())
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TeammateDto> {
        Ok(TeammateDto {
            id: row.get(0)?,
            account_id: row.get(1)?,
            platform: row.get(2)?,
            pubg_account_id: row.get(3)?,
            pubg_player_name: row.get(4)?,
            display_nickname: row.get(5)?,
            is_friend: row.get::<_, i64>(6)? == 1,
            is_points_enabled: row.get::<_, i64>(7)? == 1,
            total_points: row.get(8)?,
            last_seen_at: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    }
}
