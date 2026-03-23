/**
 * Preload API Types
 * Type definitions for the exposed API from Electron's contextBridge
 */

import type {
  IpcInvokeFunction,
  AppSetting,
  Teammate,
  Match,
  MatchPlayer,
  RedbagRule,
  RedbagRecord,
  SyncStatus,
  AppStatus,
  CreateTeammateInput,
  UpdateTeammateInput,
  CreateRedbagRuleInput,
  UpdateRedbagRuleInput,
  CalculatedRedbag,
  OverwolfStatus,
} from '@pubg-point-rankings/shared';

/**
 * Settings API
 */
export interface SettingsAPI {
  get(key: string): Promise<AppSetting | null>;
  getAll(): Promise<AppSetting[]>;
  set(key: string, value: string): Promise<void>;
}

/**
 * Teammates API
 */
export interface TeammatesAPI {
  getAll(): Promise<Teammate[]>;
  getById(id: number): Promise<Teammate | null>;
  create(input: CreateTeammateInput): Promise<Teammate>;
  update(input: UpdateTeammateInput): Promise<Teammate>;
  getHistory(id: number): Promise<{
    teammate: Teammate;
    records: RedbagRecord[];
    totalMatches: number;
  }>;
}

/**
 * Rules API
 */
export interface RulesAPI {
  getAll(): Promise<RedbagRule[]>;
  getActive(): Promise<RedbagRule | null>;
  create(input: CreateRedbagRuleInput): Promise<RedbagRule>;
  update(input: UpdateRedbagRuleInput): Promise<RedbagRule>;
  delete(id: number): Promise<void>;
  activate(id: number): Promise<RedbagRule>;
}

/**
 * Matches API
 */
export interface MatchesAPI {
  getAll(limit?: number, offset?: number): Promise<Match[]>;
  getById(matchId: string): Promise<Match | null>;
  getPlayers(matchId: string): Promise<MatchPlayer[]>;
}

/**
 * Redbags API
 */
export interface RedbagsAPI {
  getAll(limit?: number, offset?: number): Promise<RedbagRecord[]>;
  getByMatch(matchId: string): Promise<RedbagRecord[]>;
}

/**
 * Sync API
 */
export interface SyncAPI {
  getStatus(): Promise<SyncStatus>;
  start(): Promise<{ success: boolean; error?: string }>;
  startMatch(matchId: string, platform?: string): Promise<{
    success: boolean;
    match?: Match;
    players?: MatchPlayer[];
    redbags?: CalculatedRedbag[];
    error?: string;
  }>;
}

/**
 * App API
 */
export interface AppAPI {
  getStatus(): Promise<AppStatus>;
}

/**
 * Overwolf/GEP API
 */
export interface OverwolfAPI {
  getStatus(): Promise<OverwolfStatus>;
  onStatusChange(listener: (status: OverwolfStatus) => void): () => void;
}

/**
 * Complete exposed API interface
 */
export interface ElectronAPI {
  settings: SettingsAPI;
  teammates: TeammatesAPI;
  rules: RulesAPI;
  matches: MatchesAPI;
  redbags: RedbagsAPI;
  sync: SyncAPI;
  app: AppAPI;
  overwolf: OverwolfAPI;
}

/**
 * Extend the Window interface to include our API
 */
declare global {
  interface Window {
    electronAPI?: ElectronAPI;
  }
}
