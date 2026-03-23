/**
 * IPC Handlers
 * Registers all IPC handlers for communication between main and renderer processes
 */

import { ipcMain, IpcMainInvokeEvent } from 'electron';
import type { Database } from 'better-sqlite3';
import type { IpcChannel } from '@pubg-point-rankings/shared';
import { IPC_CHANNELS } from '@pubg-point-rankings/shared';
import {
  SettingsRepository,
  TeammatesRepository,
  MatchesRepository,
  RedbagRulesRepository,
  RedbagRecordsRepository,
} from '../repository';
import { SyncService } from '../services';
import type { IpcHandlerMap } from '@pubg-point-rankings/shared';
import { getOverwolfStatus } from '../main/bootstrap';

export interface IPCHandlerContext {
  db: Database;
  syncService: SyncService;
}

function withIpcErrorHandling<TResponse>(
  handler: (_event: IpcMainInvokeEvent) => Promise<TResponse> | TResponse
): (_event: IpcMainInvokeEvent) => Promise<TResponse>;
function withIpcErrorHandling<TRequest, TResponse>(
  handler: (_event: IpcMainInvokeEvent, request: TRequest) => Promise<TResponse> | TResponse
): (_event: IpcMainInvokeEvent, request: TRequest) => Promise<TResponse>;
function withIpcErrorHandling<TRequest, TResponse>(
  handler: (_event: IpcMainInvokeEvent, request?: TRequest) => Promise<TResponse> | TResponse
) {
  return async (event: IpcMainInvokeEvent, request?: TRequest): Promise<TResponse> => {
    try {
      return await handler(event, request);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unexpected IPC error';
      throw new Error(message);
    }
  };
}

/**
 * Register all IPC handlers
 */
export function registerIPCHandlers(context: IPCHandlerContext): void {
  const { db, syncService } = context;

  // Initialize repositories
  const settingsRepo = new SettingsRepository(db);
  const teammatesRepo = new TeammatesRepository(db);
  const matchesRepo = new MatchesRepository(db);
  const rulesRepo = new RedbagRulesRepository(db);
  const redbagsRepo = new RedbagRecordsRepository(db);

  // Settings handlers
  ipcMain.handle(
    IPC_CHANNELS.SETTINGS_GET,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.SETTINGS_GET]['request']) => {
      return settingsRepo.get(request.key);
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.SETTINGS_GET_ALL,
    withIpcErrorHandling(async () => {
      return settingsRepo.getAll();
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.SETTINGS_SET,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.SETTINGS_SET]['request']) => {
      settingsRepo.set(request.key, request.value);
      
      // Update sync service API key if it changed
      if (request.key === 'pubg_api_key') {
        syncService.setApiKey(request.value);
      }
    })
  );

  // Teammates handlers
  ipcMain.handle(
    IPC_CHANNELS.TEAMMATES_GET_ALL,
    withIpcErrorHandling(async () => {
      return teammatesRepo.getAll();
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.TEAMMATES_GET_BY_ID,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.TEAMMATES_GET_BY_ID]['request']) => {
      return teammatesRepo.getById(request.id);
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.TEAMMATES_CREATE,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.TEAMMATES_CREATE]['request']) => {
      return teammatesRepo.create(request);
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.TEAMMATES_UPDATE,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.TEAMMATES_UPDATE]['request']) => {
      return teammatesRepo.update(request);
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.TEAMMATES_GET_HISTORY,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.TEAMMATES_GET_HISTORY]['request']) => {
      const teammate = teammatesRepo.getById(request.id);
      if (!teammate) {
        throw new Error('Teammate not found');
      }
      const records = redbagsRepo.getByTeammate(request.id);
      const totalMatches = records.length;
      return { teammate, records, totalMatches };
    })
  );

  // Rules handlers
  ipcMain.handle(
    IPC_CHANNELS.RULES_GET_ALL,
    withIpcErrorHandling(async () => {
      return rulesRepo.getAll();
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.RULES_GET_ACTIVE,
    withIpcErrorHandling(async () => {
      return rulesRepo.getActive();
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.RULES_CREATE,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.RULES_CREATE]['request']) => {
      return rulesRepo.create(request);
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.RULES_UPDATE,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.RULES_UPDATE]['request']) => {
      return rulesRepo.update(request);
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.RULES_DELETE,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.RULES_DELETE]['request']) => {
      rulesRepo.delete(request.id);
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.RULES_ACTIVATE,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.RULES_ACTIVATE]['request']) => {
      const rule = rulesRepo.activate(request.id);
      settingsRepo.set('active_rule_id', String(rule.id));
      return rule;
    })
  );

  // Matches handlers
  ipcMain.handle(
    IPC_CHANNELS.MATCHES_GET_ALL,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.MATCHES_GET_ALL]['request']) => {
      return matchesRepo.getAll(request?.limit, request?.offset);
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.MATCHES_GET_BY_ID,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.MATCHES_GET_BY_ID]['request']) => {
      return matchesRepo.getById(request.matchId);
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.MATCHES_GET_PLAYERS,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.MATCHES_GET_PLAYERS]['request']) => {
      return matchesRepo.getPlayers(request.matchId);
    })
  );

  // Redbags handlers
  ipcMain.handle(
    IPC_CHANNELS.REDBAGS_GET_ALL,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.REDBAGS_GET_ALL]['request']) => {
      return redbagsRepo.getAll(request?.limit, request?.offset);
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.REDBAGS_GET_BY_MATCH,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.REDBAGS_GET_BY_MATCH]['request']) => {
      return redbagsRepo.getByMatch(request.matchId);
    })
  );

  // Sync handlers
  ipcMain.handle(
    IPC_CHANNELS.SYNC_GET_STATUS,
    withIpcErrorHandling(async () => {
      const lastSync = settingsRepo.getString('last_sync_at');
      const syncStatus = syncService.getStatus();
      return {
        isSyncing: syncStatus.isSyncing,
        lastSyncAt: lastSync ? new Date(lastSync) : null,
        currentMatchId: syncStatus.currentMatchId,
        error: syncStatus.lastError,
      };
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.SYNC_START,
    withIpcErrorHandling(async () => {
      const result = await syncService.syncRecentMatch();
      return {
        success: result.success,
        error: result.error,
      };
    })
  );

  ipcMain.handle(
    IPC_CHANNELS.SYNC_START_MATCH,
    withIpcErrorHandling(async (_event: IpcMainInvokeEvent, request: IpcHandlerMap[typeof IPC_CHANNELS.SYNC_START_MATCH]['request']) => {
      const result = await syncService.syncMatch(
        request.matchId, 
        request.platform as 'steam' | 'xbox' | 'psn' | 'kakao' | undefined
      );
      return {
        success: result.success,
        match: result.match,
        players: result.players,
        redbags: result.redbags,
        error: result.error,
      };
    })
  );

  // App handlers
  ipcMain.handle(
    IPC_CHANNELS.APP_GET_STATUS,
    withIpcErrorHandling(async () => {
      const dbPath = settingsRepo.getString('database_path', '');
      const version = settingsRepo.getString('app_version', '0.1.0');
      const lastSync = settingsRepo.getString('last_sync_at');
      const syncStatus = syncService.getStatus();

      return {
        version,
        databasePath: dbPath,
        isDatabaseReady: true, // TODO: Implement proper DB health check
        syncStatus: {
          isSyncing: syncStatus.isSyncing,
          lastSyncAt: lastSync ? new Date(lastSync) : null,
          currentMatchId: syncStatus.currentMatchId,
          error: syncStatus.lastError,
        },
      };
    })
  );

  // Overwolf/GEP handlers
  ipcMain.handle(
    IPC_CHANNELS.OVERWOLF_GET_STATUS,
    withIpcErrorHandling(async () => {
      return getOverwolfStatus();
    })
  );
}

/**
 * Remove all IPC handlers (for cleanup)
 */
export function unregisterIPCHandlers(): void {
  Object.values(IPC_CHANNELS).forEach((channel: IpcChannel) => {
    ipcMain.removeHandler(channel);
  });
}
