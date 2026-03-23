/**
 * Preload Script
 * Exposes a safe API to the renderer process via contextBridge
 * 
 * This file should be used as the preload script in Electron's BrowserWindow
 */

import { contextBridge, ipcRenderer } from 'electron';
import { IPC_CHANNELS } from '@pubg-point-rankings/shared';
import type { ElectronAPI } from './types';

const OVERWOLF_STATUS_CHANGED_CHANNEL = 'overwolf:statusChanged';

/**
 * Create the API object to expose to the renderer
 */
const api: ElectronAPI = {
  settings: {
    get: (key: string) => ipcRenderer.invoke(IPC_CHANNELS.SETTINGS_GET, { key }),
    getAll: () => ipcRenderer.invoke(IPC_CHANNELS.SETTINGS_GET_ALL),
    set: (key: string, value: string) => ipcRenderer.invoke(IPC_CHANNELS.SETTINGS_SET, { key, value }),
  },

  teammates: {
    getAll: () => ipcRenderer.invoke(IPC_CHANNELS.TEAMMATES_GET_ALL),
    getById: (id: number) => ipcRenderer.invoke(IPC_CHANNELS.TEAMMATES_GET_BY_ID, { id }),
    create: (input) => ipcRenderer.invoke(IPC_CHANNELS.TEAMMATES_CREATE, input),
    update: (input) => ipcRenderer.invoke(IPC_CHANNELS.TEAMMATES_UPDATE, input),
    getHistory: (id: number) => ipcRenderer.invoke(IPC_CHANNELS.TEAMMATES_GET_HISTORY, { id }),
  },

  rules: {
    getAll: () => ipcRenderer.invoke(IPC_CHANNELS.RULES_GET_ALL),
    getActive: () => ipcRenderer.invoke(IPC_CHANNELS.RULES_GET_ACTIVE),
    create: (input) => ipcRenderer.invoke(IPC_CHANNELS.RULES_CREATE, input),
    update: (input) => ipcRenderer.invoke(IPC_CHANNELS.RULES_UPDATE, input),
    delete: (id: number) => ipcRenderer.invoke(IPC_CHANNELS.RULES_DELETE, { id }),
    activate: (id: number) => ipcRenderer.invoke(IPC_CHANNELS.RULES_ACTIVATE, { id }),
  },

  matches: {
    getAll: (limit?: number, offset?: number) => ipcRenderer.invoke(IPC_CHANNELS.MATCHES_GET_ALL, { limit, offset }),
    getById: (matchId: string) => ipcRenderer.invoke(IPC_CHANNELS.MATCHES_GET_BY_ID, { matchId }),
    getPlayers: (matchId: string) => ipcRenderer.invoke(IPC_CHANNELS.MATCHES_GET_PLAYERS, { matchId }),
  },

  redbags: {
    getAll: (limit?: number, offset?: number) => ipcRenderer.invoke(IPC_CHANNELS.REDBAGS_GET_ALL, { limit, offset }),
    getByMatch: (matchId: string) => ipcRenderer.invoke(IPC_CHANNELS.REDBAGS_GET_BY_MATCH, { matchId }),
  },

  sync: {
    getStatus: () => ipcRenderer.invoke(IPC_CHANNELS.SYNC_GET_STATUS),
    start: () => ipcRenderer.invoke(IPC_CHANNELS.SYNC_START),
    startMatch: (matchId: string, platform?: string) => ipcRenderer.invoke(IPC_CHANNELS.SYNC_START_MATCH, { matchId, platform }),
  },

  app: {
    getStatus: () => ipcRenderer.invoke(IPC_CHANNELS.APP_GET_STATUS),
  },

  overwolf: {
    getStatus: () => ipcRenderer.invoke(IPC_CHANNELS.OVERWOLF_GET_STATUS),
    onStatusChange: (listener) => {
      const wrappedListener = (_event: Electron.IpcRendererEvent, status: Awaited<ReturnType<ElectronAPI['overwolf']['getStatus']>>) => {
        listener(status);
      };

      ipcRenderer.on(OVERWOLF_STATUS_CHANGED_CHANNEL, wrappedListener);

      return () => {
        ipcRenderer.removeListener(OVERWOLF_STATUS_CHANGED_CHANNEL, wrappedListener);
      };
    },
  },
};

/**
 * Expose the API to the renderer process
 */
contextBridge.exposeInMainWorld('electronAPI', api);

console.log('Preload script loaded and API exposed');
