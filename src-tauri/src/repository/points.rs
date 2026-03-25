use rusqlite::Connection;
use serde::Serialize;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointRecordDto {
    pub id: i64,
    pub match_id: String,
    pub match_player_id: i64,
    pub teammate_id: Option<i64>,
    pub rule_id: i64,
    pub rule_name_snapshot: String,
    pub damage_points_per_damage_snapshot: i64,
    pub kill_points_snapshot: i64,
    pub revive_points_snapshot: i64,
    pub rounding_mode_snapshot: String,
    pub points: i64,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct CreatePointRecordInput {
    pub match_id: String,
    pub match_player_id: i64,
    pub teammate_id: Option<i64>,
    pub rule_id: i64,
    pub rule_name_snapshot: String,
    pub damage_points_per_damage_snapshot: i64,
    pub kill_points_snapshot: i64,
    pub revive_points_snapshot: i64,
    pub rounding_mode_snapshot: String,
    pub points: i64,
    pub note: Option<String>,
}

pub struct PointRecordsRepository<'a> {
    connection: &'a Connection,
}

impl<'a> PointRecordsRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn get_all(&self, limit: i64, offset: i64) -> Result<Vec<PointRecordDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot, 
             damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot, rounding_mode_snapshot, 
             points, note, created_at 
             FROM point_records ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
        )?;

        let rows = statement.query_map([limit, offset], |row| {
            Ok(PointRecordDto {
                id: row.get(0)?,
                match_id: row.get(1)?,
                match_player_id: row.get(2)?,
                teammate_id: row.get(3)?,
                rule_id: row.get(4)?,
                rule_name_snapshot: row.get(5)?,
                damage_points_per_damage_snapshot: row.get(6)?,
                kill_points_snapshot: row.get(7)?,
                revive_points_snapshot: row.get(8)?,
                rounding_mode_snapshot: row.get(9)?,
                points: row.get(10)?,
                note: row.get(11)?,
                created_at: row.get(12)?,
            })
        })?;

        let records = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(records)
    }

    pub fn get_by_match(&self, match_id: &str) -> Result<Vec<PointRecordDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot, 
             damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot, rounding_mode_snapshot, 
             points, note, created_at 
             FROM point_records WHERE match_id = ?1 ORDER BY points DESC",
        )?;

        let rows = statement.query_map([match_id], |row| {
            Ok(PointRecordDto {
                id: row.get(0)?,
                match_id: row.get(1)?,
                match_player_id: row.get(2)?,
                teammate_id: row.get(3)?,
                rule_id: row.get(4)?,
                rule_name_snapshot: row.get(5)?,
                damage_points_per_damage_snapshot: row.get(6)?,
                kill_points_snapshot: row.get(7)?,
                revive_points_snapshot: row.get(8)?,
                rounding_mode_snapshot: row.get(9)?,
                points: row.get(10)?,
                note: row.get(11)?,
                created_at: row.get(12)?,
            })
        })?;

        let records = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(records)
    }

    pub fn get_by_teammate(&self, teammate_id: i64) -> Result<Vec<PointRecordDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot, 
             damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot, rounding_mode_snapshot, 
             points, note, created_at 
             FROM point_records WHERE teammate_id = ?1 ORDER BY created_at DESC",
        )?;

        let rows = statement.query_map([teammate_id], |row| {
            Ok(PointRecordDto {
                id: row.get(0)?,
                match_id: row.get(1)?,
                match_player_id: row.get(2)?,
                teammate_id: row.get(3)?,
                rule_id: row.get(4)?,
                rule_name_snapshot: row.get(5)?,
                damage_points_per_damage_snapshot: row.get(6)?,
                kill_points_snapshot: row.get(7)?,
                revive_points_snapshot: row.get(8)?,
                rounding_mode_snapshot: row.get(9)?,
                points: row.get(10)?,
                note: row.get(11)?,
                created_at: row.get(12)?,
            })
        })?;

        let records = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(records)
    }

    pub fn exists_for_match(&self, match_id: &str) -> Result<bool, AppError> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM point_records WHERE match_id = ?1",
            [match_id],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    pub fn create(&self, input: CreatePointRecordInput) -> Result<PointRecordDto, AppError> {
        self.connection.execute(
            "INSERT INTO point_records
             (match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot, damage_points_per_damage_snapshot,
              kill_points_snapshot, revive_points_snapshot, rounding_mode_snapshot, points, note, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, CURRENT_TIMESTAMP)",
            (
                input.match_id,
                input.match_player_id,
                input.teammate_id,
                input.rule_id,
                input.rule_name_snapshot,
                input.damage_points_per_damage_snapshot,
                input.kill_points_snapshot,
                input.revive_points_snapshot,
                input.rounding_mode_snapshot,
                input.points,
                input.note,
            ),
        )?;

        let id = self.connection.last_insert_rowid();
        let result = self.connection.query_row(
            "SELECT id, match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot,
              damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot, rounding_mode_snapshot,
              points, note, created_at
             FROM point_records
             WHERE id = ?1",
            [id],
            |row| {
                Ok(PointRecordDto {
                    id: row.get(0)?,
                    match_id: row.get(1)?,
                    match_player_id: row.get(2)?,
                    teammate_id: row.get(3)?,
                    rule_id: row.get(4)?,
                    rule_name_snapshot: row.get(5)?,
                    damage_points_per_damage_snapshot: row.get(6)?,
                    kill_points_snapshot: row.get(7)?,
                    revive_points_snapshot: row.get(8)?,
                    rounding_mode_snapshot: row.get(9)?,
                    points: row.get(10)?,
                    note: row.get(11)?,
                    created_at: row.get(12)?,
                })
            },
        );

        match result {
            Ok(record) => Ok(record),
            Err(error) => Err(error.into()),
        }
    }

    pub fn get_total_for_teammate(&self, teammate_id: i64) -> Result<i64, AppError> {
        let total: i64 = self.connection.query_row(
            "SELECT COALESCE(SUM(points), 0) FROM point_records WHERE teammate_id = ?1",
            [teammate_id],
            |row| row.get(0),
        )?;

        Ok(total)
    }

    pub fn delete_for_match(&self, match_id: &str) -> Result<(), AppError> {
        self.connection
            .execute("DELETE FROM point_records WHERE match_id = ?1", [match_id])?;
        Ok(())
    }
}
