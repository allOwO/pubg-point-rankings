import { invoke } from '@tauri-apps/api/core';
import type {
  AppSetting,
  AppStatus,
  CalculatedPoints,
  CreatePointRuleInput,
  CreateTeammateInput,
  Match,
  MatchDetail,
  MatchDamageEvent,
  MatchKillEvent,
  MatchKnockEvent,
  MatchPlayer,
  MatchPlayerWeaponStat,
  MatchReviveEvent,
  PointRecord,
  PointRule,
  RecentTeammateCandidate,
  SettlePointMatchesInput,
  SyncStatus,
  Teammate,
  UpdatePointMatchNoteInput,
  UpdatePointRuleInput,
  UpdateTeammateInput,
  PointHistoryListItem,
  PointHistoryMatchGroup,
  PointHistoryRuleChangeMarker,
  PointHistoryPlayerBreakdown,
  PointBattleDelta,
  FailedNotificationSendStatus,
  RecalculateUnsettledPointsInput,
  RecalculateUnsettledPointsResult,
  NotificationEnvStatus,
  NotificationFailedTask,
  NotificationPageStatus,
  NotificationTemplateConfig,
  UnsettledBattleSummary,
  UnsettledPlayerSummary,
} from '@pubg-point-rankings/shared';

export interface Account {
  id: number;
  accountName: string;
  selfPlayerName: string;
  selfPlatform: 'steam' | 'xbox' | 'psn' | 'kakao';
  pubgApiKey: string;
  isActive: boolean;
  createdAt: Date;
  updatedAt: Date;
}

type SyncStartResult = { success: boolean; error?: string };
type SyncStartMatchResult = {
  success: boolean;
  match?: Match;
  players?: MatchPlayer[];
  points?: CalculatedPoints[];
  error?: string;
};

export type GameProcessState = 'not_running' | 'running' | 'cooldown_polling';

export interface GameProcessStatus {
  state: GameProcessState;
  lastSeenRunningAtMs: number | null;
  cooldownStartedAtMs: number | null;
  lastProcessCheckAtMs: number | null;
  lastRecentMatchCheckAtMs: number | null;
}

export type {
  MatchDetail,
  PointHistoryListItem,
  PointHistoryMatchGroup,
  PointHistoryRuleChangeMarker,
  PointHistoryPlayerBreakdown,
  PointBattleDelta,
  UnsettledBattleSummary,
  UnsettledPlayerSummary,
} from '@pubg-point-rankings/shared';

export interface AppAPIClient {
  settings: {
    get(key: string): Promise<AppSetting | null>;
    getAll(): Promise<AppSetting[]>;
    set(key: string, value: string): Promise<void>;
  };
   accounts: {
     getAll(): Promise<Account[]>;
     getActive(): Promise<Account | null>;
     create(input: {
       accountName: string;
       selfPlayerName: string;
       selfPlatform: Account['selfPlatform'];
       pubgApiKey: string;
       setActive?: boolean;
     }): Promise<Account>;
     switch(id: number): Promise<Account>;
     updateActive(input: {
       accountName?: string;
       selfPlayerName?: string;
       selfPlatform?: Account['selfPlatform'];
       pubgApiKey?: string;
     }): Promise<Account>;
     logout(): Promise<void>;
   };
  teammates: {
    getAll(): Promise<Teammate[]>;
    getById(id: number): Promise<Teammate | null>;
    create(input: CreateTeammateInput): Promise<Teammate>;
    update(input: UpdateTeammateInput): Promise<Teammate>;
    getHistory(id: number): Promise<{ teammate: Teammate; records: PointRecord[]; totalMatches: number }>;
    getRecentCandidates(): Promise<RecentTeammateCandidate[]>;
    delete(id: number): Promise<void>;
  };
  rules: {
    getAll(): Promise<PointRule[]>;
    getActive(): Promise<PointRule | null>;
    create(input: CreatePointRuleInput): Promise<PointRule>;
    update(input: UpdatePointRuleInput): Promise<PointRule>;
    delete(id: number): Promise<void>;
    activate(id: number): Promise<PointRule>;
  };
  matches: {
    getAll(limit?: number, offset?: number): Promise<Match[]>;
    getById(matchId: string): Promise<Match | null>;
    getPlayers(matchId: string): Promise<MatchPlayer[]>;
    getDetail(matchId: string): Promise<MatchDetail | null>;
  };
  points: {
    getAll(limit?: number, offset?: number): Promise<PointRecord[]>;
    getByMatch(matchId: string): Promise<PointRecord[]>;
    getHistoryGroups(limit?: number, offset?: number): Promise<PointHistoryListItem[]>;
    getUnsettledSummary(): Promise<UnsettledBattleSummary>;
    recalculateUnsettled(input: RecalculateUnsettledPointsInput): Promise<RecalculateUnsettledPointsResult>;
    settleThroughMatch(input: SettlePointMatchesInput): Promise<{ settlementBatchId: number; settledMatchCount: number }>;
    updateMatchNote(input: UpdatePointMatchNoteInput): Promise<void>;
  };
  notifications: {
    getStatus(): Promise<NotificationPageStatus>;
    getFailedTasks(): Promise<NotificationFailedTask[]>;
    sendSelected(taskIds: number[]): Promise<{ sentIds: number[]; failedIds: number[] }>;
    deleteFailedTask(taskId: number): Promise<void>;
    installRuntime(): Promise<NotificationPageStatus>;
    startRuntime(): Promise<NotificationPageStatus>;
    stopRuntime(): Promise<void>;
    restartRuntime(): Promise<NotificationPageStatus>;
    sendTest(): Promise<void>;
    saveGroupId(groupId: string): Promise<NotificationPageStatus>;
    getTemplateConfig(): Promise<NotificationTemplateConfig>;
    saveTemplateConfig(config: NotificationTemplateConfig): Promise<NotificationTemplateConfig>;
  };
  sync: {
    getStatus(): Promise<SyncStatus>;
    start(): Promise<SyncStartResult>;
    startMatch(matchId: string, platform?: string): Promise<SyncStartMatchResult>;
  };
  app: {
    getStatus(): Promise<AppStatus>;
    getGameProcessStatus(): Promise<GameProcessStatus>;
  };
}

interface DateSettingDto {
  key: string;
  value: string;
  updatedAt: string;
}

interface AccountDto {
  id: number;
  accountName: string;
  selfPlayerName: Account['selfPlayerName'];
  selfPlatform: Account['selfPlatform'];
  pubgApiKey: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

interface SyncStatusDto {
  isSyncing: boolean;
  lastSyncAt: string | null;
  currentMatchId: string | null;
  error: string | null;
}

interface AppStatusDto {
  version: string;
  databasePath: string;
  isDatabaseReady: boolean;
  syncStatus: SyncStatusDto;
}

interface GameProcessStatusDto {
  state: GameProcessState;
  lastSeenRunningAtMs: number | null;
  cooldownStartedAtMs: number | null;
  lastProcessCheckAtMs: number | null;
  lastRecentMatchCheckAtMs: number | null;
}

interface TeammateDto {
  id: number;
  accountId: number;
  platform: Teammate['platform'];
  pubgAccountId: string | null;
  pubgPlayerName: string;
  displayNickname: string | null;
  isFriend: boolean;
  isPointsEnabled: boolean;
  totalPoints: number;
  lastSeenAt: string | null;
  createdAt: string;
  updatedAt: string;
}

interface RecentTeammateCandidateDto {
  platform: RecentTeammateCandidate['platform'];
  pubgAccountId: string | null;
  pubgPlayerName: string;
  lastTeammateAt: string;
  isFriend: boolean;
}

interface MatchDto {
  id: number;
  accountId: number;
  matchId: string;
  platform: Match['platform'];
  mapName: string | null;
  gameMode: string | null;
  playedAt: string;
  matchStartAt: string | null;
  matchEndAt: string | null;
  telemetryUrl: string | null;
  status: Match['status'];
  createdAt: string;
  updatedAt: string;
}

interface MatchPlayerDto {
  id: number;
  accountId: number;
  matchId: string;
  teammateId: number | null;
  pubgAccountId: string | null;
  pubgPlayerName: string;
  displayNicknameSnapshot: string | null;
  teamId: number | null;
  damage: number;
  kills: number;
  assists: number;
  revives: number;
  placement: number | null;
  isSelf: boolean;
  isPointsEnabledSnapshot: boolean;
  points: number;
  createdAt: string;
}

interface MatchDamageEventDto {
  id: number;
  accountId: number;
  matchId: string;
  attackerAccountId: string | null;
  attackerName: string | null;
  victimAccountId: string | null;
  victimName: string | null;
  damage: number;
  damageTypeCategory: string | null;
  damageCauserName: string | null;
  eventAt: string | null;
  createdAt: string;
}

interface MatchKillEventDto {
  id: number;
  accountId: number;
  matchId: string;
  killerAccountId: string | null;
  killerName: string | null;
  victimAccountId: string | null;
  victimName: string | null;
  assistantAccountId: string | null;
  assistantName: string | null;
  damageTypeCategory: string | null;
  damageCauserName: string | null;
  eventAt: string | null;
  createdAt: string;
}

interface MatchKnockEventDto {
  id: number;
  accountId: number;
  matchId: string;
  attackerAccountId: string | null;
  attackerName: string | null;
  victimAccountId: string | null;
  victimName: string | null;
  damageTypeCategory: string | null;
  damageCauserName: string | null;
  eventAt: string | null;
  createdAt: string;
}

interface MatchReviveEventDto {
  id: number;
  accountId: number;
  matchId: string;
  reviverAccountId: string | null;
  reviverName: string | null;
  victimAccountId: string | null;
  victimName: string | null;
  eventAt: string | null;
  createdAt: string;
}

interface MatchPlayerWeaponStatDto {
  id: number;
  accountId: number;
  matchId: string;
  pubgAccountId: string | null;
  pubgPlayerName: string;
  weaponName: string;
  totalDamage: number;
  createdAt: string;
}

interface MatchDetailDto {
  match: MatchDto;
  players: MatchPlayerDto[];
  damageEvents: MatchDamageEventDto[];
  killEvents: MatchKillEventDto[];
  knockEvents: MatchKnockEventDto[];
  reviveEvents: MatchReviveEventDto[];
  weaponStats: MatchPlayerWeaponStatDto[];
}

interface PointRuleDto {
  id: number;
  accountId: number;
  name: string;
  damagePointsPerDamage: number;
  killPoints: number;
  revivePoints: number;
  isActive: boolean;
  roundingMode: PointRule['roundingMode'];
  createdAt: string;
  updatedAt: string;
}

interface PointRecordDto {
  id: number;
  accountId: number;
  matchId: string;
  matchPlayerId: number;
  teammateId: number | null;
  ruleId: number;
  ruleNameSnapshot: string;
  damagePointsPerDamageSnapshot: number;
  killPointsSnapshot: number;
  revivePointsSnapshot: number;
  roundingModeSnapshot: PointRecord['roundingModeSnapshot'];
  points: number;
  note: string | null;
  createdAt: string;
}

interface PointHistoryPlayerBreakdownDto {
  matchPlayerId: number;
  teammateId: number | null;
  pubgAccountId: string | null;
  pubgPlayerName: string;
  displayNicknameSnapshot: string | null;
  isSelf: boolean;
  isPointsEnabledSnapshot: boolean;
  damage: number;
  kills: number;
  revives: number;
  damagePointsPerDamageSnapshot: number;
  killPointsSnapshot: number;
  revivePointsSnapshot: number;
  damagePoints: number;
  killPoints: number;
  revivePoints: number;
  totalPoints: number;
}

interface PointBattleDeltaDto {
  matchPlayerId: number;
  teammateId: number | null;
  pubgPlayerName: string;
  displayNicknameSnapshot: string | null;
  delta: number;
}

interface PointHistoryMatchGroupDto {
  type: 'match_group';
  matchId: string;
  playedAt: string;
  mapName: string | null;
  gameMode: string | null;
  ruleId: number;
  ruleNameSnapshot: string;
  isSettled: boolean;
  settledAt: string | null;
  settlementBatchId: number | null;
  note: string | null;
  players: PointHistoryPlayerBreakdownDto[];
  battleDeltas: PointBattleDeltaDto[];
}

interface PointHistoryRuleChangeMarkerDto {
  type: 'rule_change_marker';
  previousRuleName: string;
  nextRuleName: string;
  createdAt: string;
}

type PointHistoryListItemDto =
  | PointHistoryMatchGroupDto
  | PointHistoryRuleChangeMarkerDto;

interface UnsettledPlayerSummaryDto {
  teammateId: number | null;
  pubgPlayerName: string;
  displayNickname: string | null;
  isSelf: boolean;
  totalDelta: number;
}

interface UnsettledBattleSummaryDto {
  ruleId: number | null;
  activeRuleName: string | null;
  unsettledMatchCount: number;
  players: UnsettledPlayerSummaryDto[];
}

interface RecalculateUnsettledPointsResultDto {
  ruleId: number;
  ruleName: string;
  recalculatedMatchCount: number;
}

interface NotificationPageStatusDto {
  envStatus: NotificationEnvStatus;
  isEnabled: boolean;
  canInstallRuntime: boolean;
  runtimeVersion: string;
  installDir: string | null;
  webUiUrl: string | null;
  oneBotUrl: string | null;
  qqNumber: string | null;
  groupId: string;
  lastError: string | null;
}

interface NotificationFailedTaskDto {
  id: number;
  matchId: string;
  matchTime: string;
  placement: number | null;
  battleSummary: string;
  lastError: string | null;
  sendStatus: FailedNotificationSendStatus;
}

function isCommandMissingError(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error);
  return /not found|unknown command|command .* not found|desktop capability not found/i.test(message);
}

function toDate(value: string | null): Date | null {
  return value ? new Date(value) : null;
}

function hydrateSetting(dto: DateSettingDto): AppSetting {
  return { ...dto, updatedAt: new Date(dto.updatedAt) };
}

function hydrateAccount(dto: AccountDto): Account {
  return {
    ...dto,
    createdAt: new Date(dto.createdAt),
    updatedAt: new Date(dto.updatedAt),
  };
}

function hydrateSyncStatus(dto: SyncStatusDto): SyncStatus {
  return {
    ...dto,
    lastSyncAt: toDate(dto.lastSyncAt),
  };
}

function hydrateAppStatus(dto: AppStatusDto): AppStatus {
  return {
    ...dto,
    syncStatus: hydrateSyncStatus(dto.syncStatus),
  };
}

function hydrateGameProcessStatus(dto: GameProcessStatusDto): GameProcessStatus {
  return dto;
}

function hydrateTeammate(dto: TeammateDto): Teammate {
  return {
    ...dto,
    lastSeenAt: toDate(dto.lastSeenAt),
    createdAt: new Date(dto.createdAt),
    updatedAt: new Date(dto.updatedAt),
  };
}

function hydrateRecentTeammateCandidate(dto: RecentTeammateCandidateDto): RecentTeammateCandidate {
  return {
    ...dto,
    lastTeammateAt: new Date(dto.lastTeammateAt),
  };
}

function hydrateMatch(dto: MatchDto): Match {
  return {
    ...dto,
    playedAt: new Date(dto.playedAt),
    matchStartAt: toDate(dto.matchStartAt),
    matchEndAt: toDate(dto.matchEndAt),
    createdAt: new Date(dto.createdAt),
    updatedAt: new Date(dto.updatedAt),
  };
}

function hydrateMatchPlayer(dto: MatchPlayerDto): MatchPlayer {
  return {
    ...dto,
    createdAt: new Date(dto.createdAt),
  };
}

function hydrateMatchDamageEvent(dto: MatchDamageEventDto): MatchDamageEvent {
  return {
    ...dto,
    eventAt: toDate(dto.eventAt),
    createdAt: new Date(dto.createdAt),
  };
}

function hydrateMatchKillEvent(dto: MatchKillEventDto): MatchKillEvent {
  return {
    ...dto,
    eventAt: toDate(dto.eventAt),
    createdAt: new Date(dto.createdAt),
  };
}

function hydrateMatchKnockEvent(dto: MatchKnockEventDto): MatchKnockEvent {
  return {
    ...dto,
    eventAt: toDate(dto.eventAt),
    createdAt: new Date(dto.createdAt),
  };
}

function hydrateMatchReviveEvent(dto: MatchReviveEventDto): MatchReviveEvent {
  return {
    ...dto,
    eventAt: toDate(dto.eventAt),
    createdAt: new Date(dto.createdAt),
  };
}

function hydrateMatchPlayerWeaponStat(dto: MatchPlayerWeaponStatDto): MatchPlayerWeaponStat {
  return {
    ...dto,
    createdAt: new Date(dto.createdAt),
  };
}

function hydrateMatchDetail(dto: MatchDetailDto): MatchDetail {
  return {
    match: hydrateMatch(dto.match),
    players: dto.players.map(hydrateMatchPlayer),
    damageEvents: dto.damageEvents.map(hydrateMatchDamageEvent),
    killEvents: dto.killEvents.map(hydrateMatchKillEvent),
    knockEvents: dto.knockEvents.map(hydrateMatchKnockEvent),
    reviveEvents: dto.reviveEvents.map(hydrateMatchReviveEvent),
    weaponStats: dto.weaponStats.map(hydrateMatchPlayerWeaponStat),
  };
}

function hydrateRule(dto: PointRuleDto): PointRule {
  return {
    ...dto,
    createdAt: new Date(dto.createdAt),
    updatedAt: new Date(dto.updatedAt),
  };
}

function hydratePointRecord(dto: PointRecordDto): PointRecord {
  return {
    ...dto,
    createdAt: new Date(dto.createdAt),
  };
}

function hydratePointHistoryPlayerBreakdown(dto: PointHistoryPlayerBreakdownDto): PointHistoryPlayerBreakdown {
  return dto;
}

function hydratePointBattleDelta(dto: PointBattleDeltaDto): PointBattleDelta {
  return dto;
}

function hydratePointHistoryMatchGroup(dto: PointHistoryMatchGroupDto): PointHistoryMatchGroup {
  return {
    ...dto,
    playedAt: new Date(dto.playedAt),
    settledAt: toDate(dto.settledAt),
    players: dto.players.map(hydratePointHistoryPlayerBreakdown),
    battleDeltas: dto.battleDeltas.map(hydratePointBattleDelta),
  };
}

function hydratePointHistoryRuleChangeMarker(dto: PointHistoryRuleChangeMarkerDto): PointHistoryRuleChangeMarker {
  return {
    ...dto,
    createdAt: new Date(dto.createdAt),
  };
}

function hydratePointHistoryListItem(dto: PointHistoryListItemDto): PointHistoryListItem {
  if (dto.type === 'match_group') {
    return hydratePointHistoryMatchGroup(dto);
  } else {
    return hydratePointHistoryRuleChangeMarker(dto);
  }
}

function hydrateUnsettledPlayerSummary(dto: UnsettledPlayerSummaryDto): UnsettledPlayerSummary {
  return dto;
}

function hydrateUnsettledBattleSummary(dto: UnsettledBattleSummaryDto): UnsettledBattleSummary {
  return {
    ...dto,
    players: dto.players.map(hydrateUnsettledPlayerSummary),
  };
}

function hydrateRecalculateUnsettledPointsResult(
  dto: RecalculateUnsettledPointsResultDto,
): RecalculateUnsettledPointsResult {
  return dto;
}

function hydrateNotificationPageStatus(dto: NotificationPageStatusDto): NotificationPageStatus {
  return {
    envStatus: dto.envStatus,
    isEnabled: dto.isEnabled,
    canInstallRuntime: dto.canInstallRuntime,
    runtimeVersion: dto.runtimeVersion,
    installDir: dto.installDir,
    webUiUrl: dto.webUiUrl,
    oneBotUrl: dto.oneBotUrl,
    qqNumber: dto.qqNumber,
    groupId: dto.groupId,
    lastError: dto.lastError,
  };
}

function hydrateNotificationFailedTask(dto: NotificationFailedTaskDto): NotificationFailedTask {
  return {
    ...dto,
    matchTime: new Date(dto.matchTime),
  };
}

async function invokeOptional<T>(command: string, args: Record<string, unknown> | undefined, fallback: T): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    if (isCommandMissingError(error)) {
      return fallback;
    }
    throw error;
  }
}

async function invokeRequired<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args);
}

export function getRuntimeHost(): 'tauri' {
  return 'tauri';
}

export function getAPI(): AppAPIClient {
  return {
    settings: {
      get: async (key) => {
        const settings = await invokeOptional<DateSettingDto[]>('settings_get_all', undefined, []);
        const setting = settings.find((entry) => entry.key === key);
        return setting ? hydrateSetting(setting) : null;
      },
      getAll: async () => {
        const settings = await invokeOptional<DateSettingDto[]>('settings_get_all', undefined, []);
        return settings.map(hydrateSetting);
      },
      set: async (key, value) => {
        await invokeRequired('settings_set', { key, value });
      },
    },
    accounts: {
      getAll: async () => (await invokeOptional<AccountDto[]>('accounts_get_all', undefined, [])).map(hydrateAccount),
      getActive: async () => {
        const account = await invokeOptional<AccountDto | null>('accounts_get_active', undefined, null);
        return account ? hydrateAccount(account) : null;
      },
      create: async (input) => hydrateAccount(await invokeRequired<AccountDto>('accounts_create', { input })),
       switch: async (id) => hydrateAccount(await invokeRequired<AccountDto>('accounts_switch', { id })),
       updateActive: async (input) => hydrateAccount(await invokeRequired<AccountDto>('accounts_update_active', { input })),
       logout: async () => {
         await invokeRequired('accounts_logout');
       },
    },
    teammates: {
      getAll: async () => (await invokeOptional<TeammateDto[]>('teammates_get_all', undefined, [])).map(hydrateTeammate),
      getById: async (id) => {
        const teammate = await invokeOptional<TeammateDto | null>('teammates_get_by_id', { id }, null);
        return teammate ? hydrateTeammate(teammate) : null;
      },
      create: async (input) => hydrateTeammate(await invokeRequired<TeammateDto>('teammates_create', { input })),
      update: async (input) => hydrateTeammate(await invokeRequired<TeammateDto>('teammates_update', { input })),
      getHistory: async (id) => {
        const history = await invokeOptional<{
          teammate: TeammateDto;
          records: PointRecordDto[];
          totalMatches: number;
        } | null>('teammates_get_history', { id }, null);

        if (!history) {
          throw new Error('Teammate history is not available in the current backend.');
        }

        return {
          teammate: hydrateTeammate(history.teammate),
          records: history.records.map(hydratePointRecord),
          totalMatches: history.totalMatches,
        };
      },
      getRecentCandidates: async () => (
        await invokeRequired<RecentTeammateCandidateDto[]>('teammates_get_recent_candidates')
      ).map(hydrateRecentTeammateCandidate),
      delete: async (id) => {
        await invokeRequired('teammates_delete', { id });
      },
    },
    rules: {
      getAll: async () => (await invokeOptional<PointRuleDto[]>('rules_get_all', undefined, [])).map(hydrateRule),
      getActive: async () => {
        const rule = await invokeOptional<PointRuleDto | null>('rules_get_active', undefined, null);
        return rule ? hydrateRule(rule) : null;
      },
      create: async (input) => hydrateRule(await invokeRequired<PointRuleDto>('rules_create', { input })),
      update: async (input) => hydrateRule(await invokeRequired<PointRuleDto>('rules_update', { input })),
      delete: async (id) => {
        await invokeRequired('rules_delete', { id });
      },
      activate: async (id) => hydrateRule(await invokeRequired<PointRuleDto>('rules_activate', { id })),
    },
    matches: {
      getAll: async (limit, offset) => (await invokeOptional<MatchDto[]>('matches_get_all', { limit, offset }, [])).map(hydrateMatch),
      getById: async (matchId) => {
        const match = await invokeOptional<MatchDto | null>('matches_get_by_id', { matchId }, null);
        return match ? hydrateMatch(match) : null;
      },
      getPlayers: async (matchId) => (await invokeOptional<MatchPlayerDto[]>('matches_get_players', { matchId }, [])).map(hydrateMatchPlayer),
      getDetail: async (matchId) => {
        const detail = await invokeOptional<MatchDetailDto | null>('matches_get_detail', { matchId }, null);
        return detail ? hydrateMatchDetail(detail) : null;
      },
    },
    points: {
      getAll: async (limit, offset) => (await invokeOptional<PointRecordDto[]>('points_get_all', { limit, offset }, [])).map(hydratePointRecord),
      getByMatch: async (matchId) => (await invokeOptional<PointRecordDto[]>('points_get_by_match', { matchId }, [])).map(hydratePointRecord),
      getHistoryGroups: async (limit, offset) => (await invokeOptional<PointHistoryListItemDto[]>('points_get_history_groups', { limit, offset }, [])).map(hydratePointHistoryListItem),
      getUnsettledSummary: async () => hydrateUnsettledBattleSummary(await invokeRequired<UnsettledBattleSummaryDto>('points_get_unsettled_summary')),
      recalculateUnsettled: async (input) => hydrateRecalculateUnsettledPointsResult(
        await invokeRequired<RecalculateUnsettledPointsResultDto>('points_recalculate_unsettled', { input }),
      ),
      settleThroughMatch: async (input) => invokeRequired<{ settlementBatchId: number; settledMatchCount: number }>('points_settle_through_match', { input }),
      updateMatchNote: async (input) => invokeRequired('points_update_match_note', { input }),
    },
    notifications: {
      getStatus: async () => hydrateNotificationPageStatus(await invokeOptional<NotificationPageStatusDto>('notifications_get_status', undefined, {
        envStatus: 'missing_runtime',
        isEnabled: false,
        canInstallRuntime: false,
        runtimeVersion: '',
        installDir: null,
        webUiUrl: null,
        oneBotUrl: null,
        qqNumber: null,
        groupId: '',
        lastError: null,
      })),
      getFailedTasks: async () => (
        await invokeOptional<NotificationFailedTaskDto[]>('notifications_get_failed_tasks', undefined, [])
      ).map(hydrateNotificationFailedTask),
      sendSelected: async (taskIds) => invokeRequired<{ sentIds: number[]; failedIds: number[] }>('notifications_send_selected', {
        input: { taskIds },
      }),
      deleteFailedTask: async (taskId) => {
        await invokeRequired('notifications_delete_failed_task', {
          input: { taskId },
        });
      },
      installRuntime: async () => {
        const dto = await invokeRequired<NotificationPageStatusDto>('notifications_install_runtime');
        return hydrateNotificationPageStatus(dto);
      },
      startRuntime: async () => {
        const dto = await invokeRequired<NotificationPageStatusDto>('notifications_start_runtime');
        return hydrateNotificationPageStatus(dto);
      },
      stopRuntime: async () => {
        await invokeRequired('notifications_stop_runtime');
      },
      restartRuntime: async () => {
        const dto = await invokeRequired<NotificationPageStatusDto>('notifications_restart_runtime');
        return hydrateNotificationPageStatus(dto);
      },
      sendTest: async () => {
        await invokeRequired('notifications_send_test');
      },
      saveGroupId: async (groupId) => {
        const dto = await invokeRequired<NotificationPageStatusDto>('notifications_save_group_id', {
          input: { groupId },
        });
        return hydrateNotificationPageStatus(dto);
      },
      getTemplateConfig: async () => invokeRequired<NotificationTemplateConfig>('notifications_get_template_config'),
      saveTemplateConfig: async (config) => invokeRequired<NotificationTemplateConfig>('notifications_save_template_config', {
        input: { config },
      }),
    },
    sync: {
      getStatus: async () => hydrateSyncStatus(await invokeOptional<SyncStatusDto>('sync_get_status', undefined, {
        isSyncing: false,
        lastSyncAt: null,
        currentMatchId: null,
        error: null,
      })),
      start: async () => invokeOptional<SyncStartResult>('sync_start', undefined, {
        success: false,
        error: 'Sync start is not available in the current Tauri backend yet.',
      }),
      startMatch: async (matchId, platform) => {
        const result = await invokeOptional<{
          success: boolean;
          match?: MatchDto;
          players?: MatchPlayerDto[];
          points?: CalculatedPoints[];
          error?: string;
        }>('sync_start_match', { matchId, platform }, {
          success: false,
          error: 'Match sync is not available in the current Tauri backend yet.',
        });

        return {
          ...result,
          match: result.match ? hydrateMatch(result.match) : undefined,
          players: result.players?.map(hydrateMatchPlayer),
        };
      },
    },
    app: {
      getStatus: async () => hydrateAppStatus(await invokeRequired<AppStatusDto>('app_get_status')),
      getGameProcessStatus: async () => hydrateGameProcessStatus(await invokeOptional<GameProcessStatusDto>(
        'app_get_game_process_status',
        undefined,
        {
          state: 'not_running',
          lastSeenRunningAtMs: null,
          cooldownStartedAtMs: null,
          lastProcessCheckAtMs: null,
          lastRecentMatchCheckAtMs: null,
        },
      )),
    },
  };
}
