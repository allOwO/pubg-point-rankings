pub const SCHEMA_VERSION: i64 = 3;

pub const INITIAL_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY NOT NULL,
  value TEXT NOT NULL,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

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

CREATE TABLE IF NOT EXISTS schema_version (
  version INTEGER PRIMARY KEY,
  applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO schema_version (version) VALUES (3);
"#;

pub const DEFAULT_DATA_SQL: &str = r#"
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('schema_version', '3', CURRENT_TIMESTAMP);
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('auto_recent_match_enabled', '1', CURRENT_TIMESTAMP);
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('running_process_check_interval_seconds', '5', CURRENT_TIMESTAMP);
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('not_running_process_check_interval_seconds', '30', CURRENT_TIMESTAMP);
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('running_recent_match_interval_seconds', '30', CURRENT_TIMESTAMP);
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('cooldown_polling_interval_seconds', '120', CURRENT_TIMESTAMP);
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('cooldown_window_minutes', '40', CURRENT_TIMESTAMP);
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('recent_match_retry_delay_seconds', '15', CURRENT_TIMESTAMP);
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('recent_match_retry_limit', '2', CURRENT_TIMESTAMP);
"#;
