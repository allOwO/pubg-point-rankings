use rusqlite::{params, Connection};

use crate::{error::AppError, repository::rules::PointRulesRepository};

use super::schema::{DEFAULT_DATA_SQL, INITIAL_SCHEMA_SQL, SCHEMA_VERSION};

const ACCOUNT_SETTINGS_LAST_SYNC_AT_KEY: &str = "last_sync_at";
const ACCOUNT_SETTINGS_ACTIVE_RULE_ID_KEY: &str = "active_rule_id";

fn current_version(connection: &Connection) -> Result<i64, AppError> {
    let result = connection.query_row(
        "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
        [],
        |row| row.get::<_, i64>(0),
    );

    match result {
        Ok(version) => Ok(version),
        Err(rusqlite::Error::SqliteFailure(_, _)) | Err(rusqlite::Error::QueryReturnedNoRows) => {
            Ok(0)
        }
        Err(error) => Err(error.into()),
    }
}

fn set_version(connection: &Connection, version: i64) -> Result<(), AppError> {
    connection.execute(
        "INSERT OR REPLACE INTO schema_version (version, applied_at) VALUES (?1, CURRENT_TIMESTAMP)",
        [version],
    )?;

    Ok(())
}

fn table_exists(connection: &Connection, table_name: &str) -> Result<bool, AppError> {
    let count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [table_name],
        |row| row.get(0),
    )?;

    Ok(count > 0)
}

fn column_exists(
    connection: &Connection,
    table_name: &str,
    column_name: &str,
) -> Result<bool, AppError> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;

    for row in rows {
        if row?.eq_ignore_ascii_case(column_name) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn create_account_tables(connection: &Connection) -> Result<(), AppError> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS accounts (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          account_name TEXT NOT NULL UNIQUE,
          self_player_name TEXT NOT NULL,
          self_platform TEXT NOT NULL CHECK (self_platform IN ('steam', 'xbox', 'psn', 'kakao')),
          pubg_api_key TEXT NOT NULL DEFAULT '',
          is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
          created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_accounts_active ON accounts(is_active) WHERE is_active = 1;

        CREATE TABLE IF NOT EXISTS account_settings (
          account_id INTEGER NOT NULL,
          key TEXT NOT NULL,
          value TEXT NOT NULL,
          updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          PRIMARY KEY (account_id, key),
          FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
        );
        "#,
    )?;

    Ok(())
}

fn ensure_default_account(connection: &Connection) -> Result<i64, AppError> {
    let existing = connection.query_row(
        "SELECT id FROM accounts WHERE is_active = 1 LIMIT 1",
        [],
        |row| row.get::<_, i64>(0),
    );

    if let Ok(account_id) = existing {
        return Ok(account_id);
    }

    let existing_any =
        connection.query_row("SELECT id FROM accounts ORDER BY id LIMIT 1", [], |row| {
            row.get::<_, i64>(0)
        });

    if let Ok(account_id) = existing_any {
        connection.execute(
            "UPDATE accounts SET is_active = CASE WHEN id = ?1 THEN 1 ELSE 0 END, updated_at = CURRENT_TIMESTAMP",
            [account_id],
        )?;
        return Ok(account_id);
    }

    let self_player_name: String = connection
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'self_player_name' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();
    let self_platform: String = connection
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'self_platform' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "steam".to_string());
    let pubg_api_key: String = connection
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'pubg_api_key' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();

    let trimmed_name = self_player_name.trim();
    let account_name = if trimmed_name.is_empty() {
        "Default Account".to_string()
    } else {
        format!("{} Account", trimmed_name)
    };

    connection.execute(
        "INSERT INTO accounts (account_name, self_player_name, self_platform, pubg_api_key, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        (&account_name, &self_player_name, &self_platform, &pubg_api_key),
    )?;

    Ok(connection.last_insert_rowid())
}

fn migrate_legacy_settings_to_account_settings(
    connection: &Connection,
    account_id: i64,
) -> Result<(), AppError> {
    for key in [
        ACCOUNT_SETTINGS_LAST_SYNC_AT_KEY,
        ACCOUNT_SETTINGS_ACTIVE_RULE_ID_KEY,
    ] {
        let value: String = connection
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1 LIMIT 1",
                [key],
                |row| row.get(0),
            )
            .unwrap_or_default();

        connection.execute(
            "INSERT INTO account_settings (account_id, key, value, updated_at)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
             ON CONFLICT(account_id, key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![account_id, key, value],
        )?;
    }

    Ok(())
}

fn create_empty_v3_domain_tables(connection: &Connection) -> Result<(), AppError> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS teammates (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          account_id INTEGER NOT NULL,
          platform TEXT NOT NULL CHECK (platform IN ('steam', 'xbox', 'psn', 'kakao')),
          pubg_account_id TEXT,
          pubg_player_name TEXT NOT NULL,
          display_nickname TEXT,
          is_points_enabled INTEGER NOT NULL DEFAULT 1 CHECK (is_points_enabled IN (0, 1)),
          total_points INTEGER NOT NULL DEFAULT 0 CHECK (total_points >= 0),
          last_seen_at DATETIME,
          created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS matches (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          account_id INTEGER NOT NULL,
          match_id TEXT NOT NULL,
          platform TEXT NOT NULL CHECK (platform IN ('steam', 'xbox', 'psn', 'kakao')),
          map_name TEXT,
          game_mode TEXT,
          played_at DATETIME NOT NULL,
          match_start_at DATETIME,
          match_end_at DATETIME,
          telemetry_url TEXT,
          status TEXT NOT NULL CHECK (status IN ('detected', 'syncing', 'ready', 'failed')),
          created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
          UNIQUE (account_id, match_id)
        );

        CREATE TABLE IF NOT EXISTS point_rules (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          account_id INTEGER NOT NULL,
          name TEXT NOT NULL,
          damage_points_per_damage INTEGER NOT NULL DEFAULT 0 CHECK (damage_points_per_damage >= 0),
          kill_points INTEGER NOT NULL DEFAULT 0 CHECK (kill_points >= 0),
          revive_points INTEGER NOT NULL DEFAULT 0 CHECK (revive_points >= 0),
          is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
          is_deleted INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0, 1)),
          rounding_mode TEXT NOT NULL DEFAULT 'round' CHECK (rounding_mode IN ('floor', 'round', 'ceil')),
          created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_point_rules_active_per_account ON point_rules(account_id) WHERE is_active = 1;

        CREATE TABLE IF NOT EXISTS match_players (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          account_id INTEGER NOT NULL,
          match_id TEXT NOT NULL,
          teammate_id INTEGER,
          pubg_account_id TEXT,
          pubg_player_name TEXT NOT NULL,
          display_nickname_snapshot TEXT,
          team_id INTEGER,
          damage REAL NOT NULL DEFAULT 0 CHECK (damage >= 0),
          kills INTEGER NOT NULL DEFAULT 0 CHECK (kills >= 0),
          revives INTEGER NOT NULL DEFAULT 0 CHECK (revives >= 0),
          placement INTEGER CHECK (placement IS NULL OR placement > 0),
          is_self INTEGER NOT NULL DEFAULT 0 CHECK (is_self IN (0, 1)),
          is_points_enabled_snapshot INTEGER NOT NULL DEFAULT 1 CHECK (is_points_enabled_snapshot IN (0, 1)),
          points INTEGER NOT NULL DEFAULT 0 CHECK (points >= 0),
          created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          FOREIGN KEY (account_id, match_id) REFERENCES matches(account_id, match_id) ON DELETE CASCADE,
          FOREIGN KEY (teammate_id) REFERENCES teammates(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS point_records (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          account_id INTEGER NOT NULL,
          match_id TEXT NOT NULL,
          match_player_id INTEGER NOT NULL,
          teammate_id INTEGER,
          rule_id INTEGER NOT NULL,
          rule_name_snapshot TEXT NOT NULL,
          damage_points_per_damage_snapshot INTEGER NOT NULL,
          kill_points_snapshot INTEGER NOT NULL,
          revive_points_snapshot INTEGER NOT NULL,
          rounding_mode_snapshot TEXT NOT NULL CHECK (rounding_mode_snapshot IN ('floor', 'round', 'ceil')),
          points INTEGER NOT NULL CHECK (points >= 0),
          note TEXT,
          created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          FOREIGN KEY (account_id, match_id) REFERENCES matches(account_id, match_id) ON DELETE CASCADE,
          FOREIGN KEY (match_player_id) REFERENCES match_players(id) ON DELETE CASCADE,
          FOREIGN KEY (teammate_id) REFERENCES teammates(id) ON DELETE SET NULL,
          FOREIGN KEY (rule_id) REFERENCES point_rules(id) ON DELETE RESTRICT
        );

        CREATE INDEX IF NOT EXISTS idx_teammates_account_name ON teammates(account_id, pubg_player_name);
        CREATE INDEX IF NOT EXISTS idx_teammates_account_account_id ON teammates(account_id, pubg_account_id);
        CREATE INDEX IF NOT EXISTS idx_matches_account_played_at ON matches(account_id, played_at DESC);
        CREATE INDEX IF NOT EXISTS idx_matches_account_status ON matches(account_id, status);
        CREATE INDEX IF NOT EXISTS idx_match_players_account_match_id ON match_players(account_id, match_id);
        CREATE INDEX IF NOT EXISTS idx_match_players_teammate_id ON match_players(teammate_id);
        CREATE INDEX IF NOT EXISTS idx_point_records_account_match_id ON point_records(account_id, match_id);
        CREATE INDEX IF NOT EXISTS idx_point_records_teammate_id ON point_records(teammate_id);
        CREATE INDEX IF NOT EXISTS idx_point_records_account_created_at ON point_records(account_id, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_point_rules_account_created_at ON point_rules(account_id, created_at);
        "#,
    )?;

    Ok(())
}

fn migrate_domain_tables_to_account_scope(
    connection: &Connection,
    account_id: i64,
) -> Result<(), AppError> {
    if table_exists(connection, "matches")? && column_exists(connection, "matches", "account_id")? {
        return Ok(());
    }

    if !table_exists(connection, "teammates")?
        || !table_exists(connection, "matches")?
        || !table_exists(connection, "match_players")?
        || !table_exists(connection, "point_records")?
        || !table_exists(connection, "point_rules")?
    {
        create_empty_v3_domain_tables(connection)?;
        return Ok(());
    }

    connection.execute_batch("PRAGMA foreign_keys = OFF;")?;
    let tx = connection.unchecked_transaction()?;

    tx.execute_batch(
        r#"
        CREATE TABLE teammates_new (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          account_id INTEGER NOT NULL,
          platform TEXT NOT NULL CHECK (platform IN ('steam', 'xbox', 'psn', 'kakao')),
          pubg_account_id TEXT,
          pubg_player_name TEXT NOT NULL,
          display_nickname TEXT,
          is_points_enabled INTEGER NOT NULL DEFAULT 1 CHECK (is_points_enabled IN (0, 1)),
          total_points INTEGER NOT NULL DEFAULT 0 CHECK (total_points >= 0),
          last_seen_at DATETIME,
          created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
        );

        CREATE TABLE matches_new (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          account_id INTEGER NOT NULL,
          match_id TEXT NOT NULL,
          platform TEXT NOT NULL CHECK (platform IN ('steam', 'xbox', 'psn', 'kakao')),
          map_name TEXT,
          game_mode TEXT,
          played_at DATETIME NOT NULL,
          match_start_at DATETIME,
          match_end_at DATETIME,
          telemetry_url TEXT,
          status TEXT NOT NULL CHECK (status IN ('detected', 'syncing', 'ready', 'failed')),
          created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
          UNIQUE (account_id, match_id)
        );

        CREATE TABLE point_rules_new (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          account_id INTEGER NOT NULL,
          name TEXT NOT NULL,
          damage_points_per_damage INTEGER NOT NULL DEFAULT 0 CHECK (damage_points_per_damage >= 0),
          kill_points INTEGER NOT NULL DEFAULT 0 CHECK (kill_points >= 0),
          revive_points INTEGER NOT NULL DEFAULT 0 CHECK (revive_points >= 0),
          is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
          is_deleted INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0, 1)),
          rounding_mode TEXT NOT NULL DEFAULT 'round' CHECK (rounding_mode IN ('floor', 'round', 'ceil')),
          created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
        );

        CREATE TABLE match_players_new (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          account_id INTEGER NOT NULL,
          match_id TEXT NOT NULL,
          teammate_id INTEGER,
          pubg_account_id TEXT,
          pubg_player_name TEXT NOT NULL,
          display_nickname_snapshot TEXT,
          team_id INTEGER,
          damage REAL NOT NULL DEFAULT 0 CHECK (damage >= 0),
          kills INTEGER NOT NULL DEFAULT 0 CHECK (kills >= 0),
          revives INTEGER NOT NULL DEFAULT 0 CHECK (revives >= 0),
          placement INTEGER CHECK (placement IS NULL OR placement > 0),
          is_self INTEGER NOT NULL DEFAULT 0 CHECK (is_self IN (0, 1)),
          is_points_enabled_snapshot INTEGER NOT NULL DEFAULT 1 CHECK (is_points_enabled_snapshot IN (0, 1)),
          points INTEGER NOT NULL DEFAULT 0 CHECK (points >= 0),
          created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          FOREIGN KEY (account_id, match_id) REFERENCES matches_new(account_id, match_id) ON DELETE CASCADE,
          FOREIGN KEY (teammate_id) REFERENCES teammates_new(id) ON DELETE SET NULL
        );

        CREATE TABLE point_records_new (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          account_id INTEGER NOT NULL,
          match_id TEXT NOT NULL,
          match_player_id INTEGER NOT NULL,
          teammate_id INTEGER,
          rule_id INTEGER NOT NULL,
          rule_name_snapshot TEXT NOT NULL,
          damage_points_per_damage_snapshot INTEGER NOT NULL,
          kill_points_snapshot INTEGER NOT NULL,
          revive_points_snapshot INTEGER NOT NULL,
          rounding_mode_snapshot TEXT NOT NULL CHECK (rounding_mode_snapshot IN ('floor', 'round', 'ceil')),
          points INTEGER NOT NULL CHECK (points >= 0),
          note TEXT,
          created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          FOREIGN KEY (account_id, match_id) REFERENCES matches_new(account_id, match_id) ON DELETE CASCADE,
          FOREIGN KEY (match_player_id) REFERENCES match_players_new(id) ON DELETE CASCADE,
          FOREIGN KEY (teammate_id) REFERENCES teammates_new(id) ON DELETE SET NULL,
          FOREIGN KEY (rule_id) REFERENCES point_rules_new(id) ON DELETE RESTRICT
        );
        "#,
    )?;

    tx.execute(
        "INSERT INTO teammates_new (id, account_id, platform, pubg_account_id, pubg_player_name, display_nickname, is_points_enabled, total_points, last_seen_at, created_at, updated_at)
         SELECT id, ?1, platform, pubg_account_id, pubg_player_name, display_nickname, is_points_enabled, total_points, last_seen_at, created_at, updated_at FROM teammates",
        [account_id],
    )?;
    tx.execute(
        "INSERT INTO matches_new (id, account_id, match_id, platform, map_name, game_mode, played_at, match_start_at, match_end_at, telemetry_url, status, created_at, updated_at)
         SELECT id, ?1, match_id, platform, map_name, game_mode, played_at, match_start_at, match_end_at, telemetry_url, status, created_at, updated_at FROM matches",
        [account_id],
    )?;
    tx.execute(
        "INSERT INTO point_rules_new (id, account_id, name, damage_points_per_damage, kill_points, revive_points, is_active, is_deleted, rounding_mode, created_at, updated_at)
         SELECT id, ?1, name, damage_points_per_damage, kill_points, revive_points, is_active, 0, rounding_mode, created_at, updated_at FROM point_rules",
        [account_id],
    )?;
    tx.execute(
        "INSERT INTO match_players_new (id, account_id, match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot, team_id, damage, kills, revives, placement, is_self, is_points_enabled_snapshot, points, created_at)
         SELECT id, ?1, match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot, team_id, damage, kills, revives, placement, is_self, is_points_enabled_snapshot, points, created_at FROM match_players",
        [account_id],
    )?;
    tx.execute(
        "INSERT INTO point_records_new (id, account_id, match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot, damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot, rounding_mode_snapshot, points, note, created_at)
         SELECT id, ?1, match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot, damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot, rounding_mode_snapshot, points, note, created_at FROM point_records",
        [account_id],
    )?;

    tx.execute_batch(
        r#"
        DROP TABLE point_records;
        DROP TABLE match_players;
        DROP TABLE point_rules;
        DROP TABLE matches;
        DROP TABLE teammates;

        ALTER TABLE teammates_new RENAME TO teammates;
        ALTER TABLE matches_new RENAME TO matches;
        ALTER TABLE point_rules_new RENAME TO point_rules;
        ALTER TABLE match_players_new RENAME TO match_players;
        ALTER TABLE point_records_new RENAME TO point_records;

        CREATE UNIQUE INDEX IF NOT EXISTS idx_point_rules_active_per_account ON point_rules(account_id) WHERE is_active = 1;
        CREATE INDEX IF NOT EXISTS idx_teammates_account_name ON teammates(account_id, pubg_player_name);
        CREATE INDEX IF NOT EXISTS idx_teammates_account_account_id ON teammates(account_id, pubg_account_id);
        CREATE INDEX IF NOT EXISTS idx_matches_account_played_at ON matches(account_id, played_at DESC);
        CREATE INDEX IF NOT EXISTS idx_matches_account_status ON matches(account_id, status);
        CREATE INDEX IF NOT EXISTS idx_match_players_account_match_id ON match_players(account_id, match_id);
        CREATE INDEX IF NOT EXISTS idx_match_players_teammate_id ON match_players(teammate_id);
        CREATE INDEX IF NOT EXISTS idx_point_records_account_match_id ON point_records(account_id, match_id);
        CREATE INDEX IF NOT EXISTS idx_point_records_teammate_id ON point_records(teammate_id);
        CREATE INDEX IF NOT EXISTS idx_point_records_account_created_at ON point_records(account_id, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_point_rules_account_created_at ON point_rules(account_id, created_at);
        "#,
    )?;

    tx.commit()?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

    Ok(())
}

fn ensure_account_defaults(connection: &Connection, account_id: i64) -> Result<(), AppError> {
    migrate_legacy_settings_to_account_settings(connection, account_id)?;
    PointRulesRepository::new(connection, account_id).ensure_default_rule()?;
    Ok(())
}

fn migrate_v1_to_v2(connection: &Connection) -> Result<(), AppError> {
    create_account_tables(connection)?;
    let account_id = ensure_default_account(connection)?;
    migrate_legacy_settings_to_account_settings(connection, account_id)?;
    Ok(())
}

fn migrate_v2_to_v3(connection: &Connection) -> Result<(), AppError> {
    create_account_tables(connection)?;
    let account_id = ensure_default_account(connection)?;
    migrate_domain_tables_to_account_scope(connection, account_id)?;
    ensure_account_defaults(connection, account_id)?;
    Ok(())
}

fn migrate_v3_to_v4(connection: &Connection) -> Result<(), AppError> {
    if !column_exists(connection, "point_rules", "is_deleted")? {
        connection.execute(
            "ALTER TABLE point_rules ADD COLUMN is_deleted INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0, 1))",
            [],
        )?;
    }

    connection.execute(
        "UPDATE point_rules SET is_deleted = 0 WHERE is_deleted IS NULL",
        [],
    )?;

    Ok(())
}

fn migrate_v4_to_v5(connection: &Connection) -> Result<(), AppError> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS point_settlement_batches (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          account_id INTEGER NOT NULL,
          start_match_id TEXT NOT NULL,
          end_match_id TEXT NOT NULL,
          rule_id_snapshot INTEGER,
          rule_name_snapshot TEXT,
          created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
          FOREIGN KEY (account_id, start_match_id) REFERENCES matches(account_id, match_id) ON DELETE CASCADE,
          FOREIGN KEY (account_id, end_match_id) REFERENCES matches(account_id, match_id) ON DELETE CASCADE,
          FOREIGN KEY (rule_id_snapshot) REFERENCES point_rules(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS point_match_meta (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          account_id INTEGER NOT NULL,
          match_id TEXT NOT NULL,
          note TEXT,
          settled_at DATETIME,
          settlement_batch_id INTEGER,
          created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          UNIQUE (account_id, match_id),
          FOREIGN KEY (account_id, match_id) REFERENCES matches(account_id, match_id) ON DELETE CASCADE,
          FOREIGN KEY (settlement_batch_id) REFERENCES point_settlement_batches(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_point_settlement_batches_account_created_at ON point_settlement_batches(account_id, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_point_match_meta_account_settled_at ON point_match_meta(account_id, settled_at);
        CREATE INDEX IF NOT EXISTS idx_point_match_meta_account_settlement_batch_id ON point_match_meta(account_id, settlement_batch_id);
        "#,
    )?;

    Ok(())
}

pub fn bootstrap_database(connection: &Connection) -> Result<(), AppError> {
    let version = current_version(connection)?;

    if version == 0 {
        connection.execute_batch(INITIAL_SCHEMA_SQL)?;
        set_version(connection, SCHEMA_VERSION)?;
    } else {
        if version < 2 {
            migrate_v1_to_v2(connection)?;
            set_version(connection, 2)?;
        }
        if version < 3 {
            migrate_v2_to_v3(connection)?;
            set_version(connection, 3)?;
        }
        if version < 4 {
            migrate_v3_to_v4(connection)?;
            set_version(connection, 4)?;
        }
        if version < 5 {
            migrate_v4_to_v5(connection)?;
            set_version(connection, 5)?;
        }
    }

    connection.execute_batch(DEFAULT_DATA_SQL)?;
    create_account_tables(connection)?;
    let account_id = ensure_default_account(connection)?;
    ensure_account_defaults(connection, account_id)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::bootstrap_database;

    const LEGACY_SCHEMA_SQL: &str = r#"
    CREATE TABLE app_settings (
      key TEXT PRIMARY KEY NOT NULL,
      value TEXT NOT NULL,
      updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE teammates (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      platform TEXT NOT NULL,
      pubg_account_id TEXT,
      pubg_player_name TEXT NOT NULL,
      display_nickname TEXT,
      is_points_enabled INTEGER NOT NULL DEFAULT 1,
      total_points INTEGER NOT NULL DEFAULT 0,
      last_seen_at DATETIME,
      created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
      updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE matches (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      match_id TEXT NOT NULL UNIQUE,
      platform TEXT NOT NULL,
      map_name TEXT,
      game_mode TEXT,
      played_at DATETIME NOT NULL,
      match_start_at DATETIME,
      match_end_at DATETIME,
      telemetry_url TEXT,
      status TEXT NOT NULL,
      created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
      updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE match_players (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      match_id TEXT NOT NULL,
      teammate_id INTEGER,
      pubg_account_id TEXT,
      pubg_player_name TEXT NOT NULL,
      display_nickname_snapshot TEXT,
      team_id INTEGER,
      damage REAL NOT NULL DEFAULT 0,
      kills INTEGER NOT NULL DEFAULT 0,
      revives INTEGER NOT NULL DEFAULT 0,
      placement INTEGER,
      is_self INTEGER NOT NULL DEFAULT 0,
      is_points_enabled_snapshot INTEGER NOT NULL DEFAULT 1,
      points INTEGER NOT NULL DEFAULT 0,
      created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE point_rules (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT NOT NULL,
      damage_points_per_damage INTEGER NOT NULL DEFAULT 0,
      kill_points INTEGER NOT NULL DEFAULT 0,
      revive_points INTEGER NOT NULL DEFAULT 0,
      is_active INTEGER NOT NULL DEFAULT 0,
      rounding_mode TEXT NOT NULL DEFAULT 'round',
      created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
      updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE point_records (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      match_id TEXT NOT NULL,
      match_player_id INTEGER NOT NULL,
      teammate_id INTEGER,
      rule_id INTEGER NOT NULL,
      rule_name_snapshot TEXT NOT NULL,
      damage_points_per_damage_snapshot INTEGER NOT NULL,
      kill_points_snapshot INTEGER NOT NULL,
      revive_points_snapshot INTEGER NOT NULL,
      rounding_mode_snapshot TEXT NOT NULL,
      points INTEGER NOT NULL,
      note TEXT,
      created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE schema_version (
      version INTEGER PRIMARY KEY,
      applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    INSERT INTO schema_version (version) VALUES (1);
    "#;

    #[test]
    fn bootstrap_creates_default_account_for_new_database() {
        let connection = Connection::open_in_memory().expect("open in-memory db");

        bootstrap_database(&connection).expect("bootstrap new db");

        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))
            .expect("count accounts");
        let active_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM accounts WHERE is_active = 1",
                [],
                |row| row.get(0),
            )
            .expect("count active accounts");
        let teammate_has_account_id: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('teammates') WHERE name = 'account_id'",
                [],
                |row| row.get(0),
            )
            .expect("teammates account_id exists");

        assert_eq!(count, 1);
        assert_eq!(active_count, 1);
        assert_eq!(teammate_has_account_id, 1);
    }

    #[test]
    fn bootstrap_migrates_legacy_last_sync_to_account_settings() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        connection
            .execute_batch(LEGACY_SCHEMA_SQL)
            .expect("create legacy schema");
        connection
            .execute(
                "INSERT INTO app_settings (key, value) VALUES ('self_player_name', 'LegacyPlayer')",
                [],
            )
            .expect("insert legacy player name");
        connection
            .execute(
                "INSERT INTO app_settings (key, value) VALUES ('self_platform', 'steam')",
                [],
            )
            .expect("insert legacy platform");
        connection
            .execute(
                "INSERT INTO app_settings (key, value) VALUES ('pubg_api_key', 'legacy-key')",
                [],
            )
            .expect("insert legacy api key");
        connection
            .execute(
                "INSERT INTO app_settings (key, value) VALUES ('last_sync_at', '2026-01-01T00:00:00Z')",
                [],
            )
            .expect("insert legacy sync time");
        connection
            .execute(
                "INSERT INTO point_rules (name, damage_points_per_damage, kill_points, revive_points, is_active, rounding_mode)
                 VALUES ('Legacy Rule', 2, 300, 150, 1, 'round')",
                [],
            )
            .expect("insert legacy rule");

        bootstrap_database(&connection).expect("migrate legacy db");

        let account_name: String = connection
            .query_row(
                "SELECT account_name FROM accounts WHERE is_active = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("select active account name");
        let last_sync_at: String = connection
            .query_row(
                "SELECT value FROM account_settings WHERE key = 'last_sync_at' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("select migrated last sync");
        let rules_with_account: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM point_rules WHERE account_id IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .expect("count migrated rules");

        assert_eq!(account_name, "LegacyPlayer Account");
        assert_eq!(last_sync_at, "2026-01-01T00:00:00Z");
        assert_eq!(rules_with_account, 1);
    }
}
