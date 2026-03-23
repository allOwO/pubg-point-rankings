/**
 * Electron Main Process Bootstrap
 * Initializes the database, sync service, IPC handlers, and creates the main window
 */

import { app } from 'electron';
import * as path from 'path';
import * as os from 'os';
import type { Database } from 'better-sqlite3';
import type { OverwolfStatus } from '@pubg-point-rankings/shared';
import { initDatabase, getDatabase, closeDatabase, bootstrapDatabase } from '../db';
import { OverwolfGepService, SyncService } from '../services';
import { registerIPCHandlers, unregisterIPCHandlers } from '../ipc';
import { MatchesRepository, SettingsRepository } from '../repository';
import { createMainWindow, getMainWindow, sendToRenderer } from './window';

// App version
const APP_VERSION = '0.1.0';
const SYNC_RECOVERY_TIMEOUT_MS = 10 * 60 * 1000;

let overwolfStatus: OverwolfStatus = {
  isRunning: false,
  isGEPAvailable: false,
  isPUBGRunning: false,
  lastError: null,
  gameInfo: null,
};

/**
 * Get the database path based on the platform
 */
function getDatabasePath(): string {
  const homeDir = os.homedir();
  
  // Platform-specific paths
  if (process.platform === 'win32') {
    const localAppData = process.env.LOCALAPPDATA || path.join(homeDir, 'AppData', 'Local');
    return path.join(localAppData, 'pubg-redbag-plugin', 'app.db');
  } else if (process.platform === 'darwin') {
    return path.join(homeDir, 'Library', 'Application Support', 'pubg-redbag-plugin', 'app.db');
  } else {
    // Linux
    const configDir = process.env.XDG_CONFIG_HOME || path.join(homeDir, '.config');
    return path.join(configDir, 'pubg-redbag-plugin', 'app.db');
  }
}

/**
 * Initialize the main process
 */
export async function initializeMain(): Promise<void> {
  console.log('Initializing PUBG Point Rankings main process...');
  console.log('App version:', APP_VERSION);
  console.log('Electron version:', process.versions.electron);
  console.log('Node version:', process.versions.node);

  // Wait for app ready
  await app.whenReady();
  console.log('Electron app ready');

  // Get database path
  const dbPath = getDatabasePath();
  console.log('Database path:', dbPath);

  // Initialize database
  const db = initDatabase({ 
    path: dbPath,
    verbose: process.env.NODE_ENV === 'development',
  });

  // Run migrations
  bootstrapDatabase(db);
  console.log('Database initialized and migrated');

  // Initialize settings
  const settingsRepo = new SettingsRepository(db);
  const matchesRepo = new MatchesRepository(db);

  const recoveredMatches = matchesRepo.resetSyncingMatches(SYNC_RECOVERY_TIMEOUT_MS);
  if (recoveredMatches.retried > 0 || recoveredMatches.failed > 0) {
    console.warn(
      `Recovered interrupted sync records: ${recoveredMatches.retried} reset to detected, ${recoveredMatches.failed} marked failed`
    );
  }
  
  // Set default settings if not present
  const defaults = {
    'app_version': APP_VERSION,
    'database_path': dbPath,
    'pubg_api_key': '',
    'self_player_name': '',
    'self_platform': 'steam',
    'active_rule_id': '1',
    'last_sync_at': '',
  };

  for (const [key, value] of Object.entries(defaults)) {
    const existing = settingsRepo.get(key);
    if (!existing) {
      settingsRepo.set(key, value);
    }
  }

  // Get API key for sync service
  const apiKey = settingsRepo.getString('pubg_api_key');

  // Initialize sync service
  const syncService = new SyncService(db, apiKey);
  console.log('Sync service initialized');

  const overwolfService = new OverwolfGepService();
  overwolfService.onStatusChanged((status) => {
    updateOverwolfStatus(status);
  });
  console.log('Overwolf service initialized');

  // Register IPC handlers
  registerIPCHandlers({ db, syncService });
  console.log('IPC handlers registered');

  // Create the main window
  createMainWindow();
  console.log('Main window created');

  // Initialize Overwolf/GEP integration
  await overwolfService.initialize();

  // Handle app events
  app.on('window-all-closed', () => {
    console.log('All windows closed');
    // On macOS, keep app running until explicitly quit
    if (process.platform !== 'darwin') {
      app.quit();
    }
  });

  app.on('activate', () => {
    console.log('App activated');
    // Re-create window on macOS when dock icon is clicked
    if (getMainWindow() === null) {
      createMainWindow();
    }
  });

  app.on('before-quit', () => {
    console.log('App quitting, cleaning up...');
    unregisterIPCHandlers();
    closeDatabase();
  });

  console.log('Main process initialization complete');
}

/**
 * Get the initialized database instance
 */
export function getAppDatabase(): Database {
  return getDatabase();
}

/**
 * Get app version
 */
export function getAppVersion(): string {
  return APP_VERSION;
}

/**
 * Get Overwolf/GEP status
 */
export function getOverwolfStatus(): OverwolfStatus {
  return {
    ...overwolfStatus,
    gameInfo: overwolfStatus.gameInfo ? { ...overwolfStatus.gameInfo } : null,
  };
}

/**
 * Update Overwolf/GEP status
 */
export function updateOverwolfStatus(updates: Partial<OverwolfStatus>): void {
  overwolfStatus = { ...overwolfStatus, ...updates };
  sendToRenderer('overwolf:statusChanged', overwolfStatus);
}

// Export for use in main entry point
export * from '../db';
export * from '../repository';
export * from '../services';
export * from '../ipc';
export * from '../engine';
export * from '../parser';
export * from '../pubg';
export * from './window';
