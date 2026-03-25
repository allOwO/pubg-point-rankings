use rusqlite::{params, Connection};
use serde::Serialize;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchDto {
    pub id: i64,
    pub account_id: i64,
    pub match_id: String,
    pub platform: String,
    pub map_name: Option<String>,
    pub game_mode: Option<String>,
    pub played_at: String,
    pub match_start_at: Option<String>,
    pub match_end_at: Option<String>,
    pub telemetry_url: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchPlayerDto {
    pub id: i64,
    pub account_id: i64,
    pub match_id: String,
    pub teammate_id: Option<i64>,
    pub pubg_account_id: Option<String>,
    pub pubg_player_name: String,
    pub display_nickname_snapshot: Option<String>,
    pub team_id: Option<i64>,
    pub damage: f64,
    pub kills: i64,
    pub revives: i64,
    pub placement: Option<i64>,
    pub is_self: bool,
    pub is_points_enabled_snapshot: bool,
    pub points: i64,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct CreateMatchInput {
    pub match_id: String,
    pub platform: String,
    pub map_name: Option<String>,
    pub game_mode: Option<String>,
    pub played_at: String,
    pub match_start_at: Option<String>,
    pub match_end_at: Option<String>,
    pub telemetry_url: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct CreateMatchPlayerInput {
    pub match_id: String,
    pub teammate_id: Option<i64>,
    pub pubg_account_id: Option<String>,
    pub pubg_player_name: String,
    pub display_nickname_snapshot: Option<String>,
    pub team_id: Option<i64>,
    pub damage: f64,
    pub kills: i64,
    pub revives: i64,
    pub placement: Option<i64>,
    pub is_self: bool,
    pub is_points_enabled_snapshot: bool,
    pub points: i64,
}

pub struct MatchesRepository<'a> {
    connection: &'a Connection,
    account_id: i64,
}

impl<'a> MatchesRepository<'a> {
    pub fn new(connection: &'a Connection, account_id: i64) -> Self {
        Self {
            connection,
            account_id,
        }
    }

    pub fn get_all(&self, limit: i64, offset: i64) -> Result<Vec<MatchDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, match_id, platform, map_name, game_mode, played_at, match_start_at, match_end_at, telemetry_url, status, created_at, updated_at
             FROM matches WHERE account_id = ?1 ORDER BY played_at DESC LIMIT ?2 OFFSET ?3",
        )?;

        let rows = statement.query_map(params![self.account_id, limit, offset], |row| {
            Self::map_match_row(row)
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_by_id(&self, match_id: &str) -> Result<Option<MatchDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, account_id, match_id, platform, map_name, game_mode, played_at, match_start_at, match_end_at, telemetry_url, status, created_at, updated_at
             FROM matches WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
            Self::map_match_row,
        );

        match result {
            Ok(match_data) => Ok(Some(match_data)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn exists(&self, match_id: &str) -> Result<bool, AppError> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM matches WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    pub fn create(&self, input: CreateMatchInput) -> Result<MatchDto, AppError> {
        self.connection.execute(
            "INSERT INTO matches
             (account_id, match_id, platform, map_name, game_mode, played_at, match_start_at, match_end_at, telemetry_url, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![
                self.account_id,
                input.match_id,
                input.platform,
                input.map_name,
                input.game_mode,
                input.played_at,
                input.match_start_at,
                input.match_end_at,
                input.telemetry_url,
                input.status,
            ],
        )?;

        let inserted_match_id: String = self.connection.query_row(
            "SELECT match_id FROM matches WHERE id = ?1",
            [self.connection.last_insert_rowid()],
            |row| row.get(0),
        )?;

        self.get_by_id(&inserted_match_id)?
            .ok_or_else(|| AppError::Database(rusqlite::Error::ExecuteReturnedResults))
    }

    pub fn update_status(&self, match_id: &str, status: &str) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE matches SET status = ?1, updated_at = CURRENT_TIMESTAMP WHERE account_id = ?2 AND match_id = ?3",
            params![status, self.account_id, match_id],
        )?;
        Ok(())
    }

    pub fn create_player(&self, input: CreateMatchPlayerInput) -> Result<MatchPlayerDto, AppError> {
        self.connection.execute(
            "INSERT INTO match_players
             (account_id, match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot,
              team_id, damage, kills, revives, placement, is_self, is_points_enabled_snapshot, points, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, CURRENT_TIMESTAMP)",
            params![
                self.account_id,
                input.match_id,
                input.teammate_id,
                input.pubg_account_id,
                input.pubg_player_name,
                input.display_nickname_snapshot,
                input.team_id,
                input.damage,
                input.kills,
                input.revives,
                input.placement,
                if input.is_self { 1 } else { 0 },
                if input.is_points_enabled_snapshot { 1 } else { 0 },
                input.points,
            ],
        )?;

        let player_id = self.connection.last_insert_rowid();
        let result = self.connection.query_row(
            "SELECT id, account_id, match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot,
              team_id, damage, kills, revives, placement, is_self, is_points_enabled_snapshot, points, created_at
             FROM match_players
             WHERE account_id = ?1 AND id = ?2",
            params![self.account_id, player_id],
            Self::map_player_row,
        );

        match result {
            Ok(player) => Ok(player),
            Err(error) => Err(error.into()),
        }
    }

    pub fn delete_players_for_match(&self, match_id: &str) -> Result<(), AppError> {
        self.connection.execute(
            "DELETE FROM match_players WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
        )?;
        Ok(())
    }

    pub fn get_players(&self, match_id: &str) -> Result<Vec<MatchPlayerDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot,
              team_id, damage, kills, revives, placement, is_self, is_points_enabled_snapshot, points, created_at
             FROM match_players WHERE account_id = ?1 AND match_id = ?2 ORDER BY damage DESC",
        )?;

        let rows = statement.query_map(params![self.account_id, match_id], |row| {
            Self::map_player_row(row)
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    fn map_match_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MatchDto> {
        Ok(MatchDto {
            id: row.get(0)?,
            account_id: row.get(1)?,
            match_id: row.get(2)?,
            platform: row.get(3)?,
            map_name: row.get(4)?,
            game_mode: row.get(5)?,
            played_at: row.get(6)?,
            match_start_at: row.get(7)?,
            match_end_at: row.get(8)?,
            telemetry_url: row.get(9)?,
            status: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    }

    fn map_player_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MatchPlayerDto> {
        Ok(MatchPlayerDto {
            id: row.get(0)?,
            account_id: row.get(1)?,
            match_id: row.get(2)?,
            teammate_id: row.get(3)?,
            pubg_account_id: row.get(4)?,
            pubg_player_name: row.get(5)?,
            display_nickname_snapshot: row.get(6)?,
            team_id: row.get(7)?,
            damage: row.get(8)?,
            kills: row.get(9)?,
            revives: row.get(10)?,
            placement: row.get(11)?,
            is_self: row.get::<_, i64>(12)? == 1,
            is_points_enabled_snapshot: row.get::<_, i64>(13)? == 1,
            points: row.get(14)?,
            created_at: row.get(15)?,
        })
    }
}
