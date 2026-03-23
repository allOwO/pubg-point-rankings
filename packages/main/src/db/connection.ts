/**
 * Database configuration and connection management
 * Uses better-sqlite3 for synchronous SQLite operations
 */

import type { Database as DatabaseType } from 'better-sqlite3';

let db: DatabaseType | null = null;

export interface DatabaseConfig {
  path: string;
  verbose?: boolean;
}

/**
 * Initialize the database connection
 * Note: This is a placeholder - actual better-sqlite3 import will work after npm install
 */
export function initDatabase(config: DatabaseConfig): DatabaseType {
  const Database = require('better-sqlite3');
  const instance = new Database(
    config.path,
    config.verbose ? { verbose: (sql: string) => console.log('SQL:', sql) } : undefined
  );
  db = instance;

  // Enable WAL mode for better concurrency
  instance.pragma('journal_mode = WAL');
  instance.pragma('foreign_keys = ON');
  
  return instance;
}

/**
 * Get the current database instance
 */
export function getDatabase(): DatabaseType {
  if (!db) {
    throw new Error('Database not initialized. Call initDatabase() first.');
  }
  return db;
}

/**
 * Close the database connection
 */
export function closeDatabase(): void {
  if (db) {
    db.close();
    db = null;
  }
}

/**
 * Check if database is initialized
 */
export function isDatabaseInitialized(): boolean {
  return db !== null;
}
