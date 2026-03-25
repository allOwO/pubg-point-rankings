use rusqlite::{params, params_from_iter, Connection};
use serde::Serialize;

use crate::{error::AppError, repository::settings::SettingsRepository};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointRuleDto {
    pub id: i64,
    pub name: String,
    pub damage_points_per_damage: i64,
    pub kill_points: i64,
    pub revive_points: i64,
    pub is_active: bool,
    pub rounding_mode: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct CreatePointRuleInput {
    pub name: String,
    pub damage_points_per_damage: i64,
    pub kill_points: i64,
    pub revive_points: i64,
    pub rounding_mode: String,
}

#[derive(Debug, Clone)]
pub struct UpdatePointRuleInput {
    pub id: i64,
    pub name: Option<String>,
    pub damage_points_per_damage: Option<i64>,
    pub kill_points: Option<i64>,
    pub revive_points: Option<i64>,
    pub rounding_mode: Option<String>,
}

pub struct PointRulesRepository<'a> {
    connection: &'a Connection,
}

impl<'a> PointRulesRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn get_all(&self) -> Result<Vec<PointRuleDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, damage_points_per_damage, kill_points, revive_points, is_active, rounding_mode, created_at, updated_at 
             FROM point_rules ORDER BY created_at",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(PointRuleDto {
                id: row.get(0)?,
                name: row.get(1)?,
                damage_points_per_damage: row.get(2)?,
                kill_points: row.get(3)?,
                revive_points: row.get(4)?,
                is_active: row.get::<_, i64>(5)? == 1,
                rounding_mode: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        let rules = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(rules)
    }

    pub fn get_active(&self) -> Result<Option<PointRuleDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, name, damage_points_per_damage, kill_points, revive_points, is_active, rounding_mode, created_at, updated_at 
             FROM point_rules WHERE is_active = 1 LIMIT 1",
            [],
            |row| {
                Ok(PointRuleDto {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    damage_points_per_damage: row.get(2)?,
                    kill_points: row.get(3)?,
                    revive_points: row.get(4)?,
                    is_active: row.get::<_, i64>(5)? == 1,
                    rounding_mode: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        );

        match result {
            Ok(rule) => Ok(Some(rule)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<PointRuleDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, name, damage_points_per_damage, kill_points, revive_points, is_active, rounding_mode, created_at, updated_at 
             FROM point_rules WHERE id = ?1",
            [id],
            |row| {
                Ok(PointRuleDto {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    damage_points_per_damage: row.get(2)?,
                    kill_points: row.get(3)?,
                    revive_points: row.get(4)?,
                    is_active: row.get::<_, i64>(5)? == 1,
                    rounding_mode: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        );

        match result {
            Ok(rule) => Ok(Some(rule)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn create(&self, input: CreatePointRuleInput) -> Result<PointRuleDto, AppError> {
        self.connection.execute(
            "INSERT INTO point_rules 
             (name, damage_points_per_damage, kill_points, revive_points, rounding_mode, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![
                input.name,
                input.damage_points_per_damage,
                input.kill_points,
                input.revive_points,
                input.rounding_mode,
            ],
        )?;

        let id = self.connection.last_insert_rowid();
        self.get_by_id(id)?
            .ok_or_else(|| AppError::Database(rusqlite::Error::ExecuteReturnedResults))
    }

    pub fn update(&self, input: UpdatePointRuleInput) -> Result<PointRuleDto, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(name) = input.name {
            sets.push("name = ?".to_string());
            params.push(name.into());
        }
        if let Some(damage_points_per_damage) = input.damage_points_per_damage {
            sets.push("damage_points_per_damage = ?".to_string());
            params.push(damage_points_per_damage.into());
        }
        if let Some(kill_points) = input.kill_points {
            sets.push("kill_points = ?".to_string());
            params.push(kill_points.into());
        }
        if let Some(revive_points) = input.revive_points {
            sets.push("revive_points = ?".to_string());
            params.push(revive_points.into());
        }
        if let Some(rounding_mode) = input.rounding_mode {
            sets.push("rounding_mode = ?".to_string());
            params.push(rounding_mode.into());
        }

        if sets.is_empty() {
            return self
                .get_by_id(input.id)?
                .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows));
        }

        sets.push("updated_at = CURRENT_TIMESTAMP".to_string());
        params.push(input.id.into());

        let sql = format!("UPDATE point_rules SET {} WHERE id = ?", sets.join(", "));
        self.connection.execute(&sql, params_from_iter(params))?;

        self.get_by_id(input.id)?
            .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows))
    }

    pub fn delete(&self, id: i64) -> Result<(), AppError> {
        let is_active = self.connection.query_row(
            "SELECT is_active FROM point_rules WHERE id = ?1",
            [id],
            |row| row.get::<_, i64>(0),
        );

        if let Ok(1) = is_active {
            return Err(AppError::Message("Cannot delete active rule".to_string()));
        }

        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM point_records WHERE rule_id = ?1",
            [id],
            |row| row.get(0),
        )?;

        if count > 0 {
            return Err(AppError::Message(
                "Cannot delete rule that has been used".to_string(),
            ));
        }

        self.connection
            .execute("DELETE FROM point_rules WHERE id = ?1", [id])?;

        Ok(())
    }

    pub fn activate(&self, id: i64) -> Result<PointRuleDto, AppError> {
        if self.get_by_id(id)?.is_none() {
            return Err(AppError::Message("Rule not found".to_string()));
        }

        let tx = self.connection.unchecked_transaction()?;
        tx.execute(
            "UPDATE point_rules SET is_active = 0, updated_at = CURRENT_TIMESTAMP",
            [],
        )?;
        tx.execute(
            "UPDATE point_rules SET is_active = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            [id],
        )?;
        SettingsRepository::new(&tx)
            .set("active_rule_id", &id.to_string())
            .map(|_| ())?;
        tx.commit()?;

        self.get_by_id(id)?
            .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows))
    }
}
