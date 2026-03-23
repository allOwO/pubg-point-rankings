/**
 * App Settings Repository
 * Handles CRUD operations for app_settings table
 */

import type { Database } from 'better-sqlite3';
import type { AppSetting } from '@pubg-point-rankings/shared';

export class SettingsRepository {
  constructor(private db: Database) {}

  /**
   * Get a setting by key
   */
  get(key: string): AppSetting | null {
    const row = this.db.prepare(
      'SELECT key, value, updated_at as updatedAt FROM app_settings WHERE key = ?'
    ).get(key) as { key: string; value: string; updatedAt: string } | undefined;
    
    if (!row) return null;
    
    return {
      key: row.key,
      value: row.value,
      updatedAt: new Date(row.updatedAt),
    };
  }

  /**
   * Get all settings
   */
  getAll(): AppSetting[] {
    const rows = this.db.prepare(
      'SELECT key, value, updated_at as updatedAt FROM app_settings ORDER BY key'
    ).all() as Array<{ key: string; value: string; updatedAt: string }>;
    
    return rows.map(row => ({
      key: row.key,
      value: row.value,
      updatedAt: new Date(row.updatedAt),
    }));
  }

  /**
   * Set a setting value
   */
  set(key: string, value: string): void {
    this.db.prepare(
      'INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)'
    ).run(key, value);
  }

  /**
   * Set multiple settings at once
   */
  setMany(settings: Record<string, string>): void {
    const stmt = this.db.prepare(
      'INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)'
    );
    
    const transaction = this.db.transaction((items: Record<string, string>) => {
      for (const [key, value] of Object.entries(items)) {
        stmt.run(key, value);
      }
    });
    
    transaction(settings);
  }

  /**
   * Delete a setting
   */
  delete(key: string): void {
    this.db.prepare('DELETE FROM app_settings WHERE key = ?').run(key);
  }

  /**
   * Get setting as string with default
   */
  getString(key: string, defaultValue: string = ''): string {
    const setting = this.get(key);
    return setting?.value ?? defaultValue;
  }

  /**
   * Get setting as number with default
   */
  getNumber(key: string, defaultValue: number = 0): number {
    const setting = this.get(key);
    if (!setting?.value) return defaultValue;
    const num = parseInt(setting.value, 10);
    return isNaN(num) ? defaultValue : num;
  }

  /**
   * Get setting as boolean with default
   */
  getBoolean(key: string, defaultValue: boolean = false): boolean {
    const setting = this.get(key);
    if (!setting?.value) return defaultValue;
    return setting.value === '1' || setting.value.toLowerCase() === 'true';
  }
}
