use rusqlite::{params, params_from_iter, Connection};
use serde::Serialize;

use crate::{error::AppError, repository::settings::SettingsRepository};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointRuleDto {
    pub id: i64,
    pub account_id: i64,
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
    account_id: i64,
}

impl<'a> PointRulesRepository<'a> {
    pub fn new(connection: &'a Connection, account_id: i64) -> Self {
        Self {
            connection,
            account_id,
        }
    }

    pub fn get_all(&self) -> Result<Vec<PointRuleDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, name, damage_points_per_damage, kill_points, revive_points, is_active, rounding_mode, created_at, updated_at
             FROM point_rules WHERE account_id = ?1 AND is_deleted = 0 ORDER BY created_at",
        )?;

        let rows = statement.query_map([self.account_id], |row| Self::map_row(row))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_active(&self) -> Result<Option<PointRuleDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, account_id, name, damage_points_per_damage, kill_points, revive_points, is_active, rounding_mode, created_at, updated_at
             FROM point_rules WHERE account_id = ?1 AND is_active = 1 AND is_deleted = 0 LIMIT 1",
            [self.account_id],
            Self::map_row,
        );

        match result {
            Ok(rule) => Ok(Some(rule)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<PointRuleDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, account_id, name, damage_points_per_damage, kill_points, revive_points, is_active, rounding_mode, created_at, updated_at
             FROM point_rules WHERE account_id = ?1 AND id = ?2 AND is_deleted = 0",
            params![self.account_id, id],
            Self::map_row,
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
             (account_id, name, damage_points_per_damage, kill_points, revive_points, rounding_mode, is_active, is_deleted, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![
                self.account_id,
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

    pub fn ensure_default_rule(&self) -> Result<PointRuleDto, AppError> {
        if let Some(rule) = self.get_active()? {
            return Ok(rule);
        }

        let first_rule = self.connection.query_row(
            "SELECT id FROM point_rules WHERE account_id = ?1 AND is_deleted = 0 ORDER BY created_at LIMIT 1",
            [self.account_id],
            |row| row.get::<_, i64>(0),
        );

        if let Ok(rule_id) = first_rule {
            return self.activate(rule_id);
        }

        let rule = self.connection.query_row(
            "INSERT INTO point_rules
             (account_id, name, damage_points_per_damage, kill_points, revive_points, rounding_mode, is_active, is_deleted, created_at, updated_at)
             VALUES (?1, 'Default Rules', 2, 300, 150, 'round', 1, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             RETURNING id, account_id, name, damage_points_per_damage, kill_points, revive_points, is_active, rounding_mode, created_at, updated_at",
            [self.account_id],
            Self::map_row,
        )?;

        SettingsRepository::new(self.connection).set_account(
            self.account_id,
            "active_rule_id",
            &rule.id.to_string(),
        )?;

        Ok(rule)
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
        params.push(self.account_id.into());
        params.push(input.id.into());

        let sql = format!(
            "UPDATE point_rules SET {} WHERE account_id = ? AND id = ? AND is_deleted = 0",
            sets.join(", ")
        );
        self.connection.execute(&sql, params_from_iter(params))?;

        self.get_by_id(input.id)?
            .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows))
    }

    pub fn delete(&self, id: i64) -> Result<(), AppError> {
        let is_active = self.connection.query_row(
            "SELECT is_active FROM point_rules WHERE account_id = ?1 AND id = ?2 AND is_deleted = 0",
            params![self.account_id, id],
            |row| row.get::<_, i64>(0),
        );

        if let Ok(1) = is_active {
            return Err(AppError::Message("Cannot delete active rule".to_string()));
        }
        self.connection.execute(
            "UPDATE point_rules
             SET is_deleted = 1, is_active = 0, updated_at = CURRENT_TIMESTAMP
             WHERE account_id = ?1 AND id = ?2 AND is_deleted = 0",
            params![self.account_id, id],
        )?;

        Ok(())
    }

    pub fn activate(&self, id: i64) -> Result<PointRuleDto, AppError> {
        if self.get_by_id(id)?.is_none() {
            return Err(AppError::Message("Rule not found".to_string()));
        }

        let tx = self.connection.unchecked_transaction()?;
        tx.execute(
            "UPDATE point_rules SET is_active = 0, updated_at = CURRENT_TIMESTAMP WHERE account_id = ?1",
            [self.account_id],
        )?;
        tx.execute(
            "UPDATE point_rules SET is_active = 1, updated_at = CURRENT_TIMESTAMP WHERE account_id = ?1 AND id = ?2",
            params![self.account_id, id],
        )?;
        SettingsRepository::new(&tx)
            .set_account(self.account_id, "active_rule_id", &id.to_string())
            .map(|_| ())?;
        tx.commit()?;

        self.get_by_id(id)?
            .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows))
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PointRuleDto> {
        Ok(PointRuleDto {
            id: row.get(0)?,
            account_id: row.get(1)?,
            name: row.get(2)?,
            damage_points_per_damage: row.get(3)?,
            kill_points: row.get(4)?,
            revive_points: row.get(5)?,
            is_active: row.get::<_, i64>(6)? == 1,
            rounding_mode: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }
}
