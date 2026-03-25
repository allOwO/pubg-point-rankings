use rusqlite::Connection;
use serde::Serialize;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchDto {
    pub id: i64,
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
}

impl<'a> MatchesRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn get_all(&self, limit: i64, offset: i64) -> Result<Vec<MatchDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, match_id, platform, map_name, game_mode, played_at, match_start_at, match_end_at, telemetry_url, status, created_at, updated_at 
             FROM matches ORDER BY played_at DESC LIMIT ?1 OFFSET ?2",
        )?;

        let rows = statement.query_map([limit, offset], |row| {
            Ok(MatchDto {
                id: row.get(0)?,
                match_id: row.get(1)?,
                platform: row.get(2)?,
                map_name: row.get(3)?,
                game_mode: row.get(4)?,
                played_at: row.get(5)?,
                match_start_at: row.get(6)?,
                match_end_at: row.get(7)?,
                telemetry_url: row.get(8)?,
                status: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?;

        let matches = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(matches)
    }

    pub fn get_by_id(&self, match_id: &str) -> Result<Option<MatchDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, match_id, platform, map_name, game_mode, played_at, match_start_at, match_end_at, telemetry_url, status, created_at, updated_at 
             FROM matches WHERE match_id = ?1",
            [match_id],
            |row| {
                Ok(MatchDto {
                    id: row.get(0)?,
                    match_id: row.get(1)?,
                    platform: row.get(2)?,
                    map_name: row.get(3)?,
                    game_mode: row.get(4)?,
                    played_at: row.get(5)?,
                    match_start_at: row.get(6)?,
                    match_end_at: row.get(7)?,
                    telemetry_url: row.get(8)?,
                    status: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            },
        );

        match result {
            Ok(match_data) => Ok(Some(match_data)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn exists(&self, match_id: &str) -> Result<bool, AppError> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM matches WHERE match_id = ?1",
            [match_id],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    pub fn create(&self, input: CreateMatchInput) -> Result<MatchDto, AppError> {
        self.connection.execute(
            "INSERT INTO matches
             (match_id, platform, map_name, game_mode, played_at, match_start_at, match_end_at, telemetry_url, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            (
                input.match_id,
                input.platform,
                input.map_name,
                input.game_mode,
                input.played_at,
                input.match_start_at,
                input.match_end_at,
                input.telemetry_url,
                input.status,
            ),
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
            "UPDATE matches SET status = ?1, updated_at = CURRENT_TIMESTAMP WHERE match_id = ?2",
            (status, match_id),
        )?;

        Ok(())
    }

    pub fn create_player(&self, input: CreateMatchPlayerInput) -> Result<MatchPlayerDto, AppError> {
        self.connection.execute(
            "INSERT INTO match_players
             (match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot,
              team_id, damage, kills, revives, placement, is_self, is_points_enabled_snapshot, points, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, CURRENT_TIMESTAMP)",
            (
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
            ),
        )?;

        let player_id = self.connection.last_insert_rowid();
        let result = self.connection.query_row(
            "SELECT id, match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot,
              team_id, damage, kills, revives, placement, is_self, is_points_enabled_snapshot, points, created_at
             FROM match_players
             WHERE id = ?1",
            [player_id],
            |row| {
                Ok(MatchPlayerDto {
                    id: row.get(0)?,
                    match_id: row.get(1)?,
                    teammate_id: row.get(2)?,
                    pubg_account_id: row.get(3)?,
                    pubg_player_name: row.get(4)?,
                    display_nickname_snapshot: row.get(5)?,
                    team_id: row.get(6)?,
                    damage: row.get(7)?,
                    kills: row.get(8)?,
                    revives: row.get(9)?,
                    placement: row.get(10)?,
                    is_self: row.get::<_, i64>(11)? == 1,
                    is_points_enabled_snapshot: row.get::<_, i64>(12)? == 1,
                    points: row.get(13)?,
                    created_at: row.get(14)?,
                })
            },
        );

        match result {
            Ok(player) => Ok(player),
            Err(error) => Err(error.into()),
        }
    }

    pub fn delete_players_for_match(&self, match_id: &str) -> Result<(), AppError> {
        self.connection
            .execute("DELETE FROM match_players WHERE match_id = ?1", [match_id])?;
        Ok(())
    }

    pub fn get_players(&self, match_id: &str) -> Result<Vec<MatchPlayerDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot, 
             team_id, damage, kills, revives, placement, is_self, is_points_enabled_snapshot, points, created_at 
             FROM match_players WHERE match_id = ?1 ORDER BY damage DESC",
        )?;

        let rows = statement.query_map([match_id], |row| {
            Ok(MatchPlayerDto {
                id: row.get(0)?,
                match_id: row.get(1)?,
                teammate_id: row.get(2)?,
                pubg_account_id: row.get(3)?,
                pubg_player_name: row.get(4)?,
                display_nickname_snapshot: row.get(5)?,
                team_id: row.get(6)?,
                damage: row.get(7)?,
                kills: row.get(8)?,
                revives: row.get(9)?,
                placement: row.get(10)?,
                is_self: row.get::<_, i64>(11)? == 1,
                is_points_enabled_snapshot: row.get::<_, i64>(12)? == 1,
                points: row.get(13)?,
                created_at: row.get(14)?,
            })
        })?;

        let players = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(players)
    }
}
