/**
 * Database schema definitions and migration SQL
 */

// Schema version - increment when adding migrations
export const SCHEMA_VERSION = 1;

// Initial schema creation SQL
export const INITIAL_SCHEMA_SQL = `
-- App settings table
CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY NOT NULL,
  value TEXT NOT NULL,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Teammates table
CREATE TABLE IF NOT EXISTS teammates (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  platform TEXT NOT NULL CHECK (platform IN ('steam', 'xbox', 'psn', 'kakao')),
  pubg_account_id TEXT,
  pubg_player_name TEXT NOT NULL,
  display_nickname TEXT,
  is_redbag_enabled INTEGER NOT NULL DEFAULT 1 CHECK (is_redbag_enabled IN (0, 1)),
  total_redbag_cents INTEGER NOT NULL DEFAULT 0 CHECK (total_redbag_cents >= 0),
  last_seen_at DATETIME,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Matches table
CREATE TABLE IF NOT EXISTS matches (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  match_id TEXT NOT NULL UNIQUE,
  platform TEXT NOT NULL CHECK (platform IN ('steam', 'xbox', 'psn', 'kakao')),
  map_name TEXT,
  game_mode TEXT,
  played_at DATETIME NOT NULL,
  match_start_at DATETIME,
  match_end_at DATETIME,
  telemetry_url TEXT,
  status TEXT NOT NULL CHECK (status IN ('detected', 'syncing', 'ready', 'failed')),
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Match players table
CREATE TABLE IF NOT EXISTS match_players (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
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
  is_redbag_enabled_snapshot INTEGER NOT NULL DEFAULT 1 CHECK (is_redbag_enabled_snapshot IN (0, 1)),
  redbag_cents INTEGER NOT NULL DEFAULT 0 CHECK (redbag_cents >= 0),
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (match_id) REFERENCES matches(match_id) ON DELETE CASCADE,
  FOREIGN KEY (teammate_id) REFERENCES teammates(id) ON DELETE SET NULL
);

-- Redbag rules table
CREATE TABLE IF NOT EXISTS redbag_rules (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  damage_cent_per_point INTEGER NOT NULL DEFAULT 0 CHECK (damage_cent_per_point >= 0),
  kill_cent INTEGER NOT NULL DEFAULT 0 CHECK (kill_cent >= 0),
  revive_cent INTEGER NOT NULL DEFAULT 0 CHECK (revive_cent >= 0),
  is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
  rounding_mode TEXT NOT NULL DEFAULT 'round' CHECK (rounding_mode IN ('floor', 'round', 'ceil')),
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Redbag records table
CREATE TABLE IF NOT EXISTS redbag_records (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  match_id TEXT NOT NULL,
  match_player_id INTEGER NOT NULL,
  teammate_id INTEGER,
  rule_id INTEGER NOT NULL,
  rule_name_snapshot TEXT NOT NULL,
  damage_cent_per_point_snapshot INTEGER NOT NULL,
  kill_cent_snapshot INTEGER NOT NULL,
  revive_cent_snapshot INTEGER NOT NULL,
  rounding_mode_snapshot TEXT NOT NULL CHECK (rounding_mode_snapshot IN ('floor', 'round', 'ceil')),
  amount_cents INTEGER NOT NULL CHECK (amount_cents >= 0),
  note TEXT,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (match_id) REFERENCES matches(match_id) ON DELETE CASCADE,
  FOREIGN KEY (match_player_id) REFERENCES match_players(id) ON DELETE CASCADE,
  FOREIGN KEY (teammate_id) REFERENCES teammates(id) ON DELETE SET NULL,
  FOREIGN KEY (rule_id) REFERENCES redbag_rules(id) ON DELETE RESTRICT
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_matches_played_at ON matches(played_at DESC);
CREATE INDEX IF NOT EXISTS idx_matches_status ON matches(status);
CREATE INDEX IF NOT EXISTS idx_match_players_match_id ON match_players(match_id);
CREATE INDEX IF NOT EXISTS idx_match_players_teammate_id ON match_players(teammate_id);
CREATE INDEX IF NOT EXISTS idx_redbag_records_match_id ON redbag_records(match_id);
CREATE INDEX IF NOT EXISTS idx_redbag_records_teammate_id ON redbag_records(teammate_id);
CREATE INDEX IF NOT EXISTS idx_redbag_records_created_at ON redbag_records(created_at DESC);

-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
  version INTEGER PRIMARY KEY,
  applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert initial schema version
INSERT OR IGNORE INTO schema_version (version) VALUES (1);
`;

// Default data
export const DEFAULT_DATA_SQL = `
-- Insert default redbag rule
INSERT OR IGNORE INTO redbag_rules (id, name, damage_cent_per_point, kill_cent, revive_cent, is_active, rounding_mode)
VALUES (1, '默认方案', 2, 300, 150, 1, 'round');

-- Insert default settings
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('schema_version', '1', CURRENT_TIMESTAMP);
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('pubg_api_key', '', CURRENT_TIMESTAMP);
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('self_player_name', '', CURRENT_TIMESTAMP);
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('self_platform', 'steam', CURRENT_TIMESTAMP);
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('active_rule_id', '1', CURRENT_TIMESTAMP);
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('last_sync_at', '', CURRENT_TIMESTAMP);
`;

// Migrations array - add new migrations here
export const MIGRATIONS: Array<{ version: number; sql: string }> = [
  // Future migrations go here
  // { version: 2, sql: 'ALTER TABLE ...' },
];
