use rusqlite::{params, Connection};
use serde::Serialize;

use crate::{
    error::AppError,
    repository::{rules::PointRulesRepository, settings::SettingsRepository},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDto {
    pub id: i64,
    pub account_name: String,
    pub self_player_name: String,
    pub self_platform: String,
    pub pubg_api_key: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct CreateAccountInput {
    pub account_name: String,
    pub self_player_name: String,
    pub self_platform: String,
    pub pubg_api_key: String,
    pub set_active: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateAccountInput {
    pub account_name: String,
    pub self_player_name: String,
    pub self_platform: String,
    pub pubg_api_key: String,
}

pub struct AccountsRepository<'a> {
    connection: &'a Connection,
}

impl<'a> AccountsRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn get_all(&self) -> Result<Vec<AccountDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_name, self_player_name, self_platform, pubg_api_key, is_active, created_at, updated_at
             FROM accounts ORDER BY is_active DESC, created_at ASC",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(AccountDto {
                id: row.get(0)?,
                account_name: row.get(1)?,
                self_player_name: row.get(2)?,
                self_platform: row.get(3)?,
                pubg_api_key: row.get(4)?,
                is_active: row.get::<_, i64>(5)? == 1,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<AccountDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, account_name, self_player_name, self_platform, pubg_api_key, is_active, created_at, updated_at
             FROM accounts WHERE id = ?1",
            [id],
            |row| {
                Ok(AccountDto {
                    id: row.get(0)?,
                    account_name: row.get(1)?,
                    self_player_name: row.get(2)?,
                    self_platform: row.get(3)?,
                    pubg_api_key: row.get(4)?,
                    is_active: row.get::<_, i64>(5)? == 1,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        );

        match result {
            Ok(account) => Ok(Some(account)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn get_active(&self) -> Result<Option<AccountDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, account_name, self_player_name, self_platform, pubg_api_key, is_active, created_at, updated_at
             FROM accounts WHERE is_active = 1 LIMIT 1",
            [],
            |row| {
                Ok(AccountDto {
                    id: row.get(0)?,
                    account_name: row.get(1)?,
                    self_player_name: row.get(2)?,
                    self_platform: row.get(3)?,
                    pubg_api_key: row.get(4)?,
                    is_active: row.get::<_, i64>(5)? == 1,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        );

        match result {
            Ok(account) => Ok(Some(account)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn require_active(&self) -> Result<AccountDto, AppError> {
        self.get_active()?
            .ok_or_else(|| AppError::Message("No active account selected".to_string()))
    }

    pub fn create(&self, input: CreateAccountInput) -> Result<AccountDto, AppError> {
        let tx = self.connection.unchecked_transaction()?;

        if input.set_active {
            tx.execute(
                "UPDATE accounts SET is_active = 0, updated_at = CURRENT_TIMESTAMP WHERE is_active = 1",
                [],
            )?;
        }

        tx.execute(
            "INSERT INTO accounts (account_name, self_player_name, self_platform, pubg_api_key, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![
                input.account_name,
                input.self_player_name,
                input.self_platform,
                input.pubg_api_key,
                if input.set_active { 1 } else { 0 },
            ],
        )?;

        let account_id = tx.last_insert_rowid();
        SettingsRepository::new(&tx).set_account(account_id, "last_sync_at", "")?;
        PointRulesRepository::new(&tx, account_id).ensure_default_rule()?;
        tx.commit()?;

        self.get_by_id(account_id)?
            .ok_or_else(|| AppError::Database(rusqlite::Error::ExecuteReturnedResults))
    }

    pub fn switch_active(&self, id: i64) -> Result<AccountDto, AppError> {
        if self.get_by_id(id)?.is_none() {
            return Err(AppError::Message("Account not found".to_string()));
        }

        let tx = self.connection.unchecked_transaction()?;
        tx.execute(
            "UPDATE accounts SET is_active = 0, updated_at = CURRENT_TIMESTAMP",
            [],
        )?;
        tx.execute(
            "UPDATE accounts SET is_active = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            [id],
        )?;
        tx.commit()?;

        self.get_by_id(id)?
            .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows))
    }

    pub fn logout(&self) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE accounts SET is_active = 0, updated_at = CURRENT_TIMESTAMP WHERE is_active = 1",
            [],
        )?;

        Ok(())
    }

    pub fn update_active(&self, input: UpdateAccountInput) -> Result<AccountDto, AppError> {
        let active_account = self.require_active()?;

        self.connection.execute(
            "UPDATE accounts 
             SET account_name = ?1, self_player_name = ?2, self_platform = ?3, pubg_api_key = ?4, updated_at = CURRENT_TIMESTAMP 
             WHERE id = ?5",
            params![
                input.account_name,
                input.self_player_name,
                input.self_platform,
                input.pubg_api_key,
                active_account.id,
            ],
        )?;

        self.get_by_id(active_account.id)?
            .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows))
    }
}
