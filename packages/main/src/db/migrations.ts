/**
 * Database migration runner
 */

import type { Database } from 'better-sqlite3';
import { SCHEMA_VERSION, MIGRATIONS, INITIAL_SCHEMA_SQL, DEFAULT_DATA_SQL } from './schema';

/**
 * Get current schema version from database
 */
function getCurrentVersion(db: Database): number {
  try {
    const result = db.prepare('SELECT version FROM schema_version ORDER BY version DESC LIMIT 1').get() as { version: number } | undefined;
    return result?.version ?? 0;
  } catch {
    // Table doesn't exist, return 0
    return 0;
  }
}

/**
 * Set schema version in database
 */
function setVersion(db: Database, version: number): void {
  db.prepare('INSERT OR REPLACE INTO schema_version (version, applied_at) VALUES (?, CURRENT_TIMESTAMP)').run(version);
}

/**
 * Initialize database schema
 */
export function initSchema(db: Database): void {
  // Run initial schema
  db.exec(INITIAL_SCHEMA_SQL);
  
  // Insert default data
  db.exec(DEFAULT_DATA_SQL);
}

/**
 * Run pending migrations
 */
export function runMigrations(db: Database): void {
  const currentVersion = getCurrentVersion(db);
  
  if (currentVersion === 0) {
    // Fresh database - run initial schema
    initSchema(db);
    setVersion(db, SCHEMA_VERSION);
    return;
  }
  
  // Run pending migrations
  for (const migration of MIGRATIONS) {
    if (migration.version > currentVersion) {
      db.exec(migration.sql);
      setVersion(db, migration.version);
    }
  }
}

/**
 * Bootstrap the database - initialize schema and run migrations
 */
export function bootstrapDatabase(db: Database): void {
  runMigrations(db);
}
