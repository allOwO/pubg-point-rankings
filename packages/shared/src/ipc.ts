/**
 * IPC Contract Types for PUBG Point Rankings
 * 
 * These types define the contract between the main and renderer processes.
 * They are used for type-safe IPC communication.
 */

import type {
  AppSetting,
  RecentTeammateCandidate,
  Teammate,
  Match,
  MatchPlayer,
  PointRule,
  PointRecord,
  SyncStatus,
  AppStatus,
  CalculatedPoints,
} from './types';
import type { OverwolfStatus } from './overwolf';
import type {
  CreateTeammateInput,
  UpdateTeammateInput,
  CreatePointRuleInput,
  UpdatePointRuleInput,
} from './schemas';

// Channel names
export const IPC_CHANNELS = {
  // Settings
  SETTINGS_GET: 'settings:get',
  SETTINGS_GET_ALL: 'settings:getAll',
  SETTINGS_SET: 'settings:set',
  
  // Teammates
  TEAMMATES_GET_ALL: 'teammates:getAll',
  TEAMMATES_GET_BY_ID: 'teammates:getById',
  TEAMMATES_CREATE: 'teammates:create',
  TEAMMATES_UPDATE: 'teammates:update',
  TEAMMATES_GET_HISTORY: 'teammates:getHistory',
  TEAMMATES_GET_RECENT_CANDIDATES: 'teammates:getRecentCandidates',
  TEAMMATES_DELETE: 'teammates:delete',
  
  // Point Rules
  RULES_GET_ALL: 'rules:getAll',
  RULES_GET_ACTIVE: 'rules:getActive',
  RULES_CREATE: 'rules:create',
  RULES_UPDATE: 'rules:update',
  RULES_DELETE: 'rules:delete',
  RULES_ACTIVATE: 'rules:activate',
  
  // Matches
  MATCHES_GET_ALL: 'matches:getAll',
  MATCHES_GET_BY_ID: 'matches:getById',
  MATCHES_GET_PLAYERS: 'matches:getPlayers',
  
  // Points
  POINTS_GET_ALL: 'points:getAll',
  POINTS_GET_BY_MATCH: 'points:getByMatch',
  
  // Sync
  SYNC_GET_STATUS: 'sync:getStatus',
  SYNC_START: 'sync:start',
  SYNC_START_MATCH: 'sync:startMatch',
  
  // App
  APP_GET_STATUS: 'app:getStatus',

  // Overwolf/GEP
  OVERWOLF_GET_STATUS: 'overwolf:getStatus',
} as const;

export type IpcChannel = typeof IPC_CHANNELS[keyof typeof IPC_CHANNELS];

// Request/Response types

// Settings
export type SettingsGetRequest = { key: string };
export type SettingsGetResponse = AppSetting | null;
export type SettingsGetAllRequest = void;
export type SettingsGetAllResponse = AppSetting[];
export type SettingsSetRequest = { key: string; value: string };
export type SettingsSetResponse = void;

// Teammates
export type TeammatesGetAllRequest = void;
export type TeammatesGetAllResponse = Teammate[];
export type TeammatesGetByIdRequest = { id: number };
export type TeammatesGetByIdResponse = Teammate | null;
export type TeammatesCreateRequest = CreateTeammateInput;
export type TeammatesCreateResponse = Teammate;
export type TeammatesUpdateRequest = UpdateTeammateInput;
export type TeammatesUpdateResponse = Teammate;
export type TeammatesGetHistoryRequest = { id: number };
export type TeammatesGetHistoryResponse = {
  teammate: Teammate;
  records: PointRecord[];
  totalMatches: number;
};
export type TeammatesGetRecentCandidatesRequest = void;
export type TeammatesGetRecentCandidatesResponse = RecentTeammateCandidate[];
export type TeammatesDeleteRequest = { id: number };
export type TeammatesDeleteResponse = void;

// Rules
export type RulesGetAllRequest = void;
export type RulesGetAllResponse = PointRule[];
export type RulesGetActiveRequest = void;
export type RulesGetActiveResponse = PointRule | null;
export type RulesCreateRequest = CreatePointRuleInput;
export type RulesCreateResponse = PointRule;
export type RulesUpdateRequest = UpdatePointRuleInput;
export type RulesUpdateResponse = PointRule;
export type RulesDeleteRequest = { id: number };
export type RulesDeleteResponse = void;
export type RulesActivateRequest = { id: number };
export type RulesActivateResponse = PointRule;

// Matches
export type MatchesGetAllRequest = { limit?: number; offset?: number };
export type MatchesGetAllResponse = Match[];
export type MatchesGetByIdRequest = { matchId: string };
export type MatchesGetByIdResponse = Match | null;
export type MatchesGetPlayersRequest = { matchId: string };
export type MatchesGetPlayersResponse = MatchPlayer[];

// Points
export type PointsGetAllRequest = { limit?: number; offset?: number };
export type PointsGetAllResponse = PointRecord[];
export type PointsGetByMatchRequest = { matchId: string };
export type PointsGetByMatchResponse = PointRecord[];

// Sync
export type SyncGetStatusRequest = void;
export type SyncGetStatusResponse = SyncStatus;
export type SyncStartRequest = void;
export type SyncStartResponse = { success: boolean; error?: string };
export type SyncStartMatchRequest = { matchId: string; platform?: string };
export type SyncStartMatchResponse = { 
  success: boolean; 
  match?: Match; 
  players?: MatchPlayer[];
  points?: CalculatedPoints[];
  error?: string;
};

// App
export type AppGetStatusRequest = void;
export type AppGetStatusResponse = AppStatus;

// Overwolf/GEP
export type OverwolfGetStatusRequest = void;
export type OverwolfGetStatusResponse = OverwolfStatus;

// IPC Handler type map
export interface IpcHandlerMap {
  // Settings
  [IPC_CHANNELS.SETTINGS_GET]: {
    request: SettingsGetRequest;
    response: SettingsGetResponse;
  };
  [IPC_CHANNELS.SETTINGS_GET_ALL]: {
    request: SettingsGetAllRequest;
    response: SettingsGetAllResponse;
  };
  [IPC_CHANNELS.SETTINGS_SET]: {
    request: SettingsSetRequest;
    response: SettingsSetResponse;
  };
  
  // Teammates
  [IPC_CHANNELS.TEAMMATES_GET_ALL]: {
    request: TeammatesGetAllRequest;
    response: TeammatesGetAllResponse;
  };
  [IPC_CHANNELS.TEAMMATES_GET_BY_ID]: {
    request: TeammatesGetByIdRequest;
    response: TeammatesGetByIdResponse;
  };
  [IPC_CHANNELS.TEAMMATES_CREATE]: {
    request: TeammatesCreateRequest;
    response: TeammatesCreateResponse;
  };
  [IPC_CHANNELS.TEAMMATES_UPDATE]: {
    request: TeammatesUpdateRequest;
    response: TeammatesUpdateResponse;
  };
  [IPC_CHANNELS.TEAMMATES_GET_HISTORY]: {
    request: TeammatesGetHistoryRequest;
    response: TeammatesGetHistoryResponse;
  };
  [IPC_CHANNELS.TEAMMATES_GET_RECENT_CANDIDATES]: {
    request: TeammatesGetRecentCandidatesRequest;
    response: TeammatesGetRecentCandidatesResponse;
  };
  [IPC_CHANNELS.TEAMMATES_DELETE]: {
    request: TeammatesDeleteRequest;
    response: TeammatesDeleteResponse;
  };
  
  // Rules
  [IPC_CHANNELS.RULES_GET_ALL]: {
    request: RulesGetAllRequest;
    response: RulesGetAllResponse;
  };
  [IPC_CHANNELS.RULES_GET_ACTIVE]: {
    request: RulesGetActiveRequest;
    response: RulesGetActiveResponse;
  };
  [IPC_CHANNELS.RULES_CREATE]: {
    request: RulesCreateRequest;
    response: RulesCreateResponse;
  };
  [IPC_CHANNELS.RULES_UPDATE]: {
    request: RulesUpdateRequest;
    response: RulesUpdateResponse;
  };
  [IPC_CHANNELS.RULES_DELETE]: {
    request: RulesDeleteRequest;
    response: RulesDeleteResponse;
  };
  [IPC_CHANNELS.RULES_ACTIVATE]: {
    request: RulesActivateRequest;
    response: RulesActivateResponse;
  };
  
  // Matches
  [IPC_CHANNELS.MATCHES_GET_ALL]: {
    request: MatchesGetAllRequest;
    response: MatchesGetAllResponse;
  };
  [IPC_CHANNELS.MATCHES_GET_BY_ID]: {
    request: MatchesGetByIdRequest;
    response: MatchesGetByIdResponse;
  };
  [IPC_CHANNELS.MATCHES_GET_PLAYERS]: {
    request: MatchesGetPlayersRequest;
    response: MatchesGetPlayersResponse;
  };
  
  // Points
  [IPC_CHANNELS.POINTS_GET_ALL]: {
    request: PointsGetAllRequest;
    response: PointsGetAllResponse;
  };
  [IPC_CHANNELS.POINTS_GET_BY_MATCH]: {
    request: PointsGetByMatchRequest;
    response: PointsGetByMatchResponse;
  };
  
  // Sync
  [IPC_CHANNELS.SYNC_GET_STATUS]: {
    request: SyncGetStatusRequest;
    response: SyncGetStatusResponse;
  };
  [IPC_CHANNELS.SYNC_START]: {
    request: SyncStartRequest;
    response: SyncStartResponse;
  };
  [IPC_CHANNELS.SYNC_START_MATCH]: {
    request: SyncStartMatchRequest;
    response: SyncStartMatchResponse;
  };
  
  // App
  [IPC_CHANNELS.APP_GET_STATUS]: {
    request: AppGetStatusRequest;
    response: AppGetStatusResponse;
  };

  // Overwolf/GEP
  [IPC_CHANNELS.OVERWOLF_GET_STATUS]: {
    request: OverwolfGetStatusRequest;
    response: OverwolfGetStatusResponse;
  };
}

// Type-safe IPC invoke function type (for renderer)
export type IpcInvokeFunction = <T extends IpcChannel>(
  channel: T,
  ...args: IpcHandlerMap[T]['request'] extends void 
    ? [] 
    : [IpcHandlerMap[T]['request']]
) => Promise<IpcHandlerMap[T]['response']>;
