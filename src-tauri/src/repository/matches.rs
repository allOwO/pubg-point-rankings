use rusqlite::{params, Connection};
use serde::Serialize;

use crate::{error::AppError, parser::telemetry::display_damage_causer_name};

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
    pub assists: i64,
    pub revives: i64,
    pub placement: Option<i64>,
    pub is_self: bool,
    pub is_points_enabled_snapshot: bool,
    pub points: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchDamageEventDto {
    pub id: i64,
    pub account_id: i64,
    pub match_id: String,
    pub attacker_account_id: Option<String>,
    pub attacker_name: Option<String>,
    pub victim_account_id: Option<String>,
    pub victim_name: Option<String>,
    pub damage: f64,
    pub damage_type_category: Option<String>,
    pub damage_causer_name: Option<String>,
    pub event_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchKillEventDto {
    pub id: i64,
    pub account_id: i64,
    pub match_id: String,
    pub killer_account_id: Option<String>,
    pub killer_name: Option<String>,
    pub victim_account_id: Option<String>,
    pub victim_name: Option<String>,
    pub assistant_account_id: Option<String>,
    pub assistant_name: Option<String>,
    pub damage_type_category: Option<String>,
    pub damage_causer_name: Option<String>,
    pub event_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchKnockEventDto {
    pub id: i64,
    pub account_id: i64,
    pub match_id: String,
    pub attacker_account_id: Option<String>,
    pub attacker_name: Option<String>,
    pub victim_account_id: Option<String>,
    pub victim_name: Option<String>,
    pub damage_type_category: Option<String>,
    pub damage_causer_name: Option<String>,
    pub event_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchReviveEventDto {
    pub id: i64,
    pub account_id: i64,
    pub match_id: String,
    pub reviver_account_id: Option<String>,
    pub reviver_name: Option<String>,
    pub victim_account_id: Option<String>,
    pub victim_name: Option<String>,
    pub event_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchPlayerWeaponStatDto {
    pub id: i64,
    pub account_id: i64,
    pub match_id: String,
    pub pubg_account_id: Option<String>,
    pub pubg_player_name: String,
    pub weapon_name: String,
    pub total_damage: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchDetailDto {
    pub r#match: MatchDto,
    pub players: Vec<MatchPlayerDto>,
    pub damage_events: Vec<MatchDamageEventDto>,
    pub kill_events: Vec<MatchKillEventDto>,
    pub knock_events: Vec<MatchKnockEventDto>,
    pub revive_events: Vec<MatchReviveEventDto>,
    pub weapon_stats: Vec<MatchPlayerWeaponStatDto>,
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
    pub assists: i64,
    pub revives: i64,
    pub placement: Option<i64>,
    pub is_self: bool,
    pub is_points_enabled_snapshot: bool,
    pub points: i64,
}

#[derive(Debug, Clone)]
pub struct CreateMatchDamageEventInput {
    pub match_id: String,
    pub attacker_account_id: Option<String>,
    pub attacker_name: Option<String>,
    pub victim_account_id: Option<String>,
    pub victim_name: Option<String>,
    pub damage: f64,
    pub damage_type_category: Option<String>,
    pub damage_causer_name: Option<String>,
    pub event_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateMatchKillEventInput {
    pub match_id: String,
    pub killer_account_id: Option<String>,
    pub killer_name: Option<String>,
    pub victim_account_id: Option<String>,
    pub victim_name: Option<String>,
    pub assistant_account_id: Option<String>,
    pub assistant_name: Option<String>,
    pub damage_type_category: Option<String>,
    pub damage_causer_name: Option<String>,
    pub event_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateMatchKnockEventInput {
    pub match_id: String,
    pub attacker_account_id: Option<String>,
    pub attacker_name: Option<String>,
    pub victim_account_id: Option<String>,
    pub victim_name: Option<String>,
    pub damage_type_category: Option<String>,
    pub damage_causer_name: Option<String>,
    pub event_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateMatchReviveEventInput {
    pub match_id: String,
    pub reviver_account_id: Option<String>,
    pub reviver_name: Option<String>,
    pub victim_account_id: Option<String>,
    pub victim_name: Option<String>,
    pub event_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateMatchPlayerWeaponStatInput {
    pub match_id: String,
    pub pubg_account_id: Option<String>,
    pub pubg_player_name: String,
    pub weapon_name: String,
    pub total_damage: f64,
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
             FROM matches
             WHERE account_id = ?1
             ORDER BY match_end_at DESC, played_at DESC, match_id DESC
             LIMIT ?2 OFFSET ?3",
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

    pub fn get_detail(&self, match_id: &str) -> Result<Option<MatchDetailDto>, AppError> {
        let Some(match_data) = self.get_by_id(match_id)? else {
            return Ok(None);
        };

        Ok(Some(MatchDetailDto {
            r#match: match_data,
            players: self.get_players(match_id)?,
            damage_events: self.get_damage_events(match_id)?,
            kill_events: self.get_kill_events(match_id)?,
            knock_events: self.get_knock_events(match_id)?,
            revive_events: self.get_revive_events(match_id)?,
            weapon_stats: self.get_weapon_stats(match_id)?,
        }))
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

    pub fn update_match_fields(
        &self,
        input: CreateMatchInput,
    ) -> Result<Option<MatchDto>, AppError> {
        self.connection.execute(
            "UPDATE matches
             SET platform = ?1,
                 map_name = ?2,
                 game_mode = ?3,
                 played_at = ?4,
                 match_start_at = ?5,
                 match_end_at = ?6,
                 telemetry_url = ?7,
                 status = ?8,
                 updated_at = CURRENT_TIMESTAMP
             WHERE account_id = ?9 AND match_id = ?10",
            params![
                input.platform,
                input.map_name,
                input.game_mode,
                input.played_at,
                input.match_start_at,
                input.match_end_at,
                input.telemetry_url,
                input.status,
                self.account_id,
                input.match_id,
            ],
        )?;

        self.get_by_id(&input.match_id)
    }

    pub fn update_status(&self, match_id: &str, status: &str) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE matches SET status = ?1, updated_at = CURRENT_TIMESTAMP WHERE account_id = ?2 AND match_id = ?3",
            params![status, self.account_id, match_id],
        )?;
        Ok(())
    }

    pub fn has_detail_payload(&self, match_id: &str) -> Result<bool, AppError> {
        let damage_count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM match_damage_events WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
            |row| row.get(0),
        )?;
        let kill_count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM match_kill_events WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
            |row| row.get(0),
        )?;
        let knock_count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM match_knock_events WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
            |row| row.get(0),
        )?;
        let revive_count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM match_revive_events WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
            |row| row.get(0),
        )?;
        let weapon_count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM match_player_weapon_stats WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
            |row| row.get(0),
        )?;

        Ok(damage_count > 0
            || kill_count > 0
            || knock_count > 0
            || revive_count > 0
            || weapon_count > 0)
    }

    pub fn update_match_metadata(
        &self,
        match_id: &str,
        played_at: String,
        match_start_at: Option<String>,
        match_end_at: Option<String>,
        telemetry_url: Option<String>,
    ) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE matches
             SET played_at = ?1,
                 match_start_at = ?2,
                 match_end_at = ?3,
                 telemetry_url = ?4,
                 updated_at = CURRENT_TIMESTAMP
             WHERE account_id = ?5 AND match_id = ?6",
            params![
                played_at,
                match_start_at,
                match_end_at,
                telemetry_url,
                self.account_id,
                match_id,
            ],
        )?;
        Ok(())
    }

    pub fn create_player(&self, input: CreateMatchPlayerInput) -> Result<MatchPlayerDto, AppError> {
        self.connection.execute(
            "INSERT INTO match_players
             (account_id, match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot,
              team_id, damage, kills, assists, revives, placement, is_self, is_points_enabled_snapshot, points, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, CURRENT_TIMESTAMP)",
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
                input.assists,
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
              team_id, damage, kills, assists, revives, placement, is_self, is_points_enabled_snapshot, points, created_at
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

    pub fn create_damage_event(
        &self,
        input: CreateMatchDamageEventInput,
    ) -> Result<MatchDamageEventDto, AppError> {
        self.connection.execute(
            "INSERT INTO match_damage_events
             (account_id, match_id, attacker_account_id, attacker_name, victim_account_id, victim_name,
              damage, damage_type_category, damage_causer_name, event_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, CURRENT_TIMESTAMP)",
            params![
                self.account_id,
                input.match_id,
                input.attacker_account_id,
                input.attacker_name,
                input.victim_account_id,
                input.victim_name,
                input.damage,
                input.damage_type_category,
                input.damage_causer_name,
                input.event_at,
            ],
        )?;

        let id = self.connection.last_insert_rowid();
        let result = self.connection.query_row(
            "SELECT id, account_id, match_id, attacker_account_id, attacker_name, victim_account_id, victim_name,
              damage, damage_type_category, damage_causer_name, event_at, created_at
             FROM match_damage_events
             WHERE account_id = ?1 AND id = ?2",
            params![self.account_id, id],
            Self::map_damage_event_row,
        );

        match result {
            Ok(item) => Ok(item),
            Err(error) => Err(error.into()),
        }
    }

    pub fn create_kill_event(
        &self,
        input: CreateMatchKillEventInput,
    ) -> Result<MatchKillEventDto, AppError> {
        self.connection.execute(
            "INSERT INTO match_kill_events
             (account_id, match_id, killer_account_id, killer_name, victim_account_id, victim_name,
              assistant_account_id, assistant_name, damage_type_category, damage_causer_name, event_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, CURRENT_TIMESTAMP)",
            params![
                self.account_id,
                input.match_id,
                input.killer_account_id,
                input.killer_name,
                input.victim_account_id,
                input.victim_name,
                input.assistant_account_id,
                input.assistant_name,
                input.damage_type_category,
                input.damage_causer_name,
                input.event_at,
            ],
        )?;

        let id = self.connection.last_insert_rowid();
        let result = self.connection.query_row(
            "SELECT id, account_id, match_id, killer_account_id, killer_name, victim_account_id, victim_name,
              assistant_account_id, assistant_name, damage_type_category, damage_causer_name, event_at, created_at
             FROM match_kill_events
             WHERE account_id = ?1 AND id = ?2",
            params![self.account_id, id],
            Self::map_kill_event_row,
        );

        match result {
            Ok(item) => Ok(item),
            Err(error) => Err(error.into()),
        }
    }

    pub fn create_knock_event(
        &self,
        input: CreateMatchKnockEventInput,
    ) -> Result<MatchKnockEventDto, AppError> {
        self.connection.execute(
            "INSERT INTO match_knock_events
             (account_id, match_id, attacker_account_id, attacker_name, victim_account_id, victim_name,
              damage_type_category, damage_causer_name, event_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, CURRENT_TIMESTAMP)",
            params![
                self.account_id,
                input.match_id,
                input.attacker_account_id,
                input.attacker_name,
                input.victim_account_id,
                input.victim_name,
                input.damage_type_category,
                input.damage_causer_name,
                input.event_at,
            ],
        )?;

        let id = self.connection.last_insert_rowid();
        let result = self.connection.query_row(
            "SELECT id, account_id, match_id, attacker_account_id, attacker_name, victim_account_id, victim_name,
              damage_type_category, damage_causer_name, event_at, created_at
             FROM match_knock_events
             WHERE account_id = ?1 AND id = ?2",
            params![self.account_id, id],
            Self::map_knock_event_row,
        );

        match result {
            Ok(item) => Ok(item),
            Err(error) => Err(error.into()),
        }
    }

    pub fn create_revive_event(
        &self,
        input: CreateMatchReviveEventInput,
    ) -> Result<MatchReviveEventDto, AppError> {
        self.connection.execute(
            "INSERT INTO match_revive_events
             (account_id, match_id, reviver_account_id, reviver_name, victim_account_id, victim_name, event_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)",
            params![
                self.account_id,
                input.match_id,
                input.reviver_account_id,
                input.reviver_name,
                input.victim_account_id,
                input.victim_name,
                input.event_at,
            ],
        )?;

        let id = self.connection.last_insert_rowid();
        let result = self.connection.query_row(
            "SELECT id, account_id, match_id, reviver_account_id, reviver_name, victim_account_id, victim_name, event_at, created_at
             FROM match_revive_events
             WHERE account_id = ?1 AND id = ?2",
            params![self.account_id, id],
            Self::map_revive_event_row,
        );

        match result {
            Ok(item) => Ok(item),
            Err(error) => Err(error.into()),
        }
    }

    pub fn create_weapon_stat(
        &self,
        input: CreateMatchPlayerWeaponStatInput,
    ) -> Result<MatchPlayerWeaponStatDto, AppError> {
        self.connection.execute(
            "INSERT INTO match_player_weapon_stats
             (account_id, match_id, pubg_account_id, pubg_player_name, weapon_name, total_damage, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)",
            params![
                self.account_id,
                input.match_id,
                input.pubg_account_id,
                input.pubg_player_name,
                input.weapon_name,
                input.total_damage,
            ],
        )?;

        let id = self.connection.last_insert_rowid();
        let result = self.connection.query_row(
            "SELECT id, account_id, match_id, pubg_account_id, pubg_player_name, weapon_name, total_damage, created_at
             FROM match_player_weapon_stats
             WHERE account_id = ?1 AND id = ?2",
            params![self.account_id, id],
            Self::map_weapon_stat_row,
        );

        match result {
            Ok(item) => Ok(item),
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

    pub fn delete_detail_events_for_match(&self, match_id: &str) -> Result<(), AppError> {
        self.connection.execute(
            "DELETE FROM match_damage_events WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
        )?;
        self.connection.execute(
            "DELETE FROM match_kill_events WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
        )?;
        self.connection.execute(
            "DELETE FROM match_knock_events WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
        )?;
        self.connection.execute(
            "DELETE FROM match_revive_events WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
        )?;
        self.connection.execute(
            "DELETE FROM match_player_weapon_stats WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
        )?;
        Ok(())
    }

    pub fn get_players(&self, match_id: &str) -> Result<Vec<MatchPlayerDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot,
              team_id, damage, kills, assists, revives, placement, is_self, is_points_enabled_snapshot, points, created_at
             FROM match_players
             WHERE account_id = ?1 AND match_id = ?2
             ORDER BY points DESC, kills DESC, assists DESC, damage DESC",
        )?;

        let rows = statement.query_map(params![self.account_id, match_id], |row| {
            Self::map_player_row(row)
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_damage_events(&self, match_id: &str) -> Result<Vec<MatchDamageEventDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, match_id, attacker_account_id, attacker_name, victim_account_id, victim_name,
              damage, damage_type_category, damage_causer_name, event_at, created_at
             FROM match_damage_events
             WHERE account_id = ?1 AND match_id = ?2
             ORDER BY event_at ASC, id ASC",
        )?;
        let rows = statement.query_map(params![self.account_id, match_id], |row| {
            Self::map_damage_event_row(row)
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_kill_events(&self, match_id: &str) -> Result<Vec<MatchKillEventDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, match_id, killer_account_id, killer_name, victim_account_id, victim_name,
              assistant_account_id, assistant_name, damage_type_category, damage_causer_name, event_at, created_at
             FROM match_kill_events
             WHERE account_id = ?1 AND match_id = ?2
             ORDER BY event_at ASC, id ASC",
        )?;
        let rows = statement.query_map(params![self.account_id, match_id], |row| {
            Self::map_kill_event_row(row)
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_knock_events(&self, match_id: &str) -> Result<Vec<MatchKnockEventDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, match_id, attacker_account_id, attacker_name, victim_account_id, victim_name,
              damage_type_category, damage_causer_name, event_at, created_at
             FROM match_knock_events
             WHERE account_id = ?1 AND match_id = ?2
             ORDER BY event_at ASC, id ASC",
        )?;
        let rows = statement.query_map(params![self.account_id, match_id], |row| {
            Self::map_knock_event_row(row)
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_revive_events(&self, match_id: &str) -> Result<Vec<MatchReviveEventDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, match_id, reviver_account_id, reviver_name, victim_account_id, victim_name, event_at, created_at
             FROM match_revive_events
             WHERE account_id = ?1 AND match_id = ?2
             ORDER BY event_at ASC, id ASC",
        )?;
        let rows = statement.query_map(params![self.account_id, match_id], |row| {
            Self::map_revive_event_row(row)
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_weapon_stats(
        &self,
        match_id: &str,
    ) -> Result<Vec<MatchPlayerWeaponStatDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, match_id, pubg_account_id, pubg_player_name, weapon_name, total_damage, created_at
             FROM match_player_weapon_stats
             WHERE account_id = ?1 AND match_id = ?2
             ORDER BY total_damage DESC, pubg_player_name ASC, weapon_name ASC",
        )?;
        let rows = statement.query_map(params![self.account_id, match_id], |row| {
            Self::map_weapon_stat_row(row)
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
            assists: row.get(10)?,
            revives: row.get(11)?,
            placement: row.get(12)?,
            is_self: row.get::<_, i64>(13)? == 1,
            is_points_enabled_snapshot: row.get::<_, i64>(14)? == 1,
            points: row.get(15)?,
            created_at: row.get(16)?,
        })
    }

    fn map_damage_event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MatchDamageEventDto> {
        let damage_type_category: Option<String> = row.get(8)?;
        let damage_causer_name: Option<String> = row.get(9)?;

        Ok(MatchDamageEventDto {
            id: row.get(0)?,
            account_id: row.get(1)?,
            match_id: row.get(2)?,
            attacker_account_id: row.get(3)?,
            attacker_name: row.get(4)?,
            victim_account_id: row.get(5)?,
            victim_name: row.get(6)?,
            damage: row.get(7)?,
            damage_type_category: damage_type_category.clone(),
            damage_causer_name: Some(display_damage_causer_name(
                damage_causer_name.as_deref(),
                damage_type_category.as_deref(),
            )),
            event_at: row.get(10)?,
            created_at: row.get(11)?,
        })
    }

    fn map_kill_event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MatchKillEventDto> {
        let damage_type_category: Option<String> = row.get(9)?;
        let damage_causer_name: Option<String> = row.get(10)?;

        Ok(MatchKillEventDto {
            id: row.get(0)?,
            account_id: row.get(1)?,
            match_id: row.get(2)?,
            killer_account_id: row.get(3)?,
            killer_name: row.get(4)?,
            victim_account_id: row.get(5)?,
            victim_name: row.get(6)?,
            assistant_account_id: row.get(7)?,
            assistant_name: row.get(8)?,
            damage_type_category: damage_type_category.clone(),
            damage_causer_name: Some(display_damage_causer_name(
                damage_causer_name.as_deref(),
                damage_type_category.as_deref(),
            )),
            event_at: row.get(11)?,
            created_at: row.get(12)?,
        })
    }

    fn map_knock_event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MatchKnockEventDto> {
        let damage_type_category: Option<String> = row.get(7)?;
        let damage_causer_name: Option<String> = row.get(8)?;

        Ok(MatchKnockEventDto {
            id: row.get(0)?,
            account_id: row.get(1)?,
            match_id: row.get(2)?,
            attacker_account_id: row.get(3)?,
            attacker_name: row.get(4)?,
            victim_account_id: row.get(5)?,
            victim_name: row.get(6)?,
            damage_type_category: damage_type_category.clone(),
            damage_causer_name: Some(display_damage_causer_name(
                damage_causer_name.as_deref(),
                damage_type_category.as_deref(),
            )),
            event_at: row.get(9)?,
            created_at: row.get(10)?,
        })
    }

    fn map_revive_event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MatchReviveEventDto> {
        Ok(MatchReviveEventDto {
            id: row.get(0)?,
            account_id: row.get(1)?,
            match_id: row.get(2)?,
            reviver_account_id: row.get(3)?,
            reviver_name: row.get(4)?,
            victim_account_id: row.get(5)?,
            victim_name: row.get(6)?,
            event_at: row.get(7)?,
            created_at: row.get(8)?,
        })
    }

    fn map_weapon_stat_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MatchPlayerWeaponStatDto> {
        let weapon_name: String = row.get(5)?;

        Ok(MatchPlayerWeaponStatDto {
            id: row.get(0)?,
            account_id: row.get(1)?,
            match_id: row.get(2)?,
            pubg_account_id: row.get(3)?,
            pubg_player_name: row.get(4)?,
            weapon_name: display_damage_causer_name(Some(weapon_name.as_str()), None),
            total_damage: row.get(6)?,
            created_at: row.get(7)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::{CreateMatchInput, MatchesRepository};
    use crate::db::migrations::bootstrap_database;

    #[test]
    fn get_detail_maps_legacy_raw_damage_causer_values_from_db() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        bootstrap_database(&connection).expect("bootstrap db");

        let repo = MatchesRepository::new(&connection, 1);
        repo.create(CreateMatchInput {
            match_id: "match-1".to_string(),
            platform: "steam".to_string(),
            map_name: Some("Erangel".to_string()),
            game_mode: Some("squad".to_string()),
            played_at: "2026-01-01T10:00:00Z".to_string(),
            match_start_at: Some("2026-01-01T10:00:00Z".to_string()),
            match_end_at: Some("2026-01-01T10:30:00Z".to_string()),
            telemetry_url: None,
            status: "ready".to_string(),
        })
        .expect("create match");

        connection
            .execute(
                "INSERT INTO match_damage_events
                 (account_id, match_id, attacker_account_id, attacker_name, victim_account_id, victim_name,
                  damage, damage_type_category, damage_causer_name, event_at, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, CURRENT_TIMESTAMP)",
                params![
                    1,
                    "match-1",
                    "a1",
                    "self",
                    "a2",
                    "enemy",
                    35.0,
                    "Gun",
                    "WeapMk12_C",
                    "2026-01-01T10:05:00Z"
                ],
            )
            .expect("insert raw damage event");
        connection
            .execute(
                "INSERT INTO match_kill_events
                 (account_id, match_id, killer_account_id, killer_name, victim_account_id, victim_name,
                  assistant_account_id, assistant_name, damage_type_category, damage_causer_name, event_at, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, CURRENT_TIMESTAMP)",
                params![
                    1,
                    "match-1",
                    "a1",
                    "self",
                    "a2",
                    "enemy",
                    Option::<String>::None,
                    Option::<String>::None,
                    "Vehicle Crash",
                    "Uaz_A_01_C",
                    "2026-01-01T10:06:00Z"
                ],
            )
            .expect("insert raw kill event");
        connection
            .execute(
                "INSERT INTO match_player_weapon_stats
                 (account_id, match_id, pubg_account_id, pubg_player_name, weapon_name, total_damage, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)",
                params![1, "match-1", "a1", "self", "WeapMk12_C", 35.0],
            )
            .expect("insert raw weapon stat");

        let detail = repo
            .get_detail("match-1")
            .expect("load detail")
            .expect("detail exists");

        assert_eq!(
            detail.damage_events[0].damage_causer_name.as_deref(),
            Some("Mk12")
        );
        assert_eq!(
            detail.kill_events[0].damage_causer_name.as_deref(),
            Some("UAZ (open top)")
        );
        assert_eq!(detail.weapon_stats[0].weapon_name, "Mk12");
    }
}
