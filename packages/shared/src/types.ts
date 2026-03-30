/**
 * Domain types for PUBG Point Rankings
 * 
 * This file contains core domain types that are shared across main and renderer processes.
 * User-facing scores are stored as integer points.
 */

export type Platform = 'steam' | 'xbox' | 'psn' | 'kakao';

export type MatchStatus = 'detected' | 'syncing' | 'ready' | 'failed';

export type RoundingMode = 'floor' | 'round' | 'ceil';

export interface AppSetting {
  key: string;
  value: string;
  updatedAt: Date;
}

export interface Account {
  id: number;
  accountName: string;
  selfPlayerName: string;
  selfPlatform: Platform;
  pubgApiKey: string;
  isActive: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface Teammate {
  id: number;
  accountId: number;
  platform: Platform;
  pubgAccountId: string | null;
  pubgPlayerName: string;
  displayNickname: string | null;
  isFriend: boolean;
  isPointsEnabled: boolean;
  totalPoints: number;
  lastSeenAt: Date | null;
  createdAt: Date;
  updatedAt: Date;
}

export interface RecentTeammateCandidate {
  platform: Platform;
  pubgAccountId: string | null;
  pubgPlayerName: string;
  lastTeammateAt: Date;
  isFriend: boolean;
}

export interface Match {
  id: number;
  accountId: number;
  matchId: string;
  platform: Platform;
  mapName: string | null;
  gameMode: string | null;
  playedAt: Date;
  matchStartAt: Date | null;
  matchEndAt: Date | null;
  telemetryUrl: string | null;
  status: MatchStatus;
  createdAt: Date;
  updatedAt: Date;
}

export interface MatchPlayer {
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
  createdAt: Date;
}

export interface MatchDamageEvent {
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
  eventAt: Date | null;
  createdAt: Date;
}

export interface MatchKillEvent {
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
  eventAt: Date | null;
  createdAt: Date;
}

export interface MatchKnockEvent {
  id: number;
  accountId: number;
  matchId: string;
  attackerAccountId: string | null;
  attackerName: string | null;
  victimAccountId: string | null;
  victimName: string | null;
  damageTypeCategory: string | null;
  damageCauserName: string | null;
  eventAt: Date | null;
  createdAt: Date;
}

export interface MatchReviveEvent {
  id: number;
  accountId: number;
  matchId: string;
  reviverAccountId: string | null;
  reviverName: string | null;
  victimAccountId: string | null;
  victimName: string | null;
  eventAt: Date | null;
  createdAt: Date;
}

export interface MatchPlayerWeaponStat {
  id: number;
  accountId: number;
  matchId: string;
  pubgAccountId: string | null;
  pubgPlayerName: string;
  weaponName: string;
  totalDamage: number;
  createdAt: Date;
}

export interface MatchDetail {
  match: Match;
  players: MatchPlayer[];
  damageEvents: MatchDamageEvent[];
  killEvents: MatchKillEvent[];
  knockEvents: MatchKnockEvent[];
  reviveEvents: MatchReviveEvent[];
  weaponStats: MatchPlayerWeaponStat[];
}

export interface PointRule {
  id: number;
  accountId: number;
  name: string;
  damagePointsPerDamage: number;
  killPoints: number;
  revivePoints: number;
  isActive: boolean;
  roundingMode: RoundingMode;
  createdAt: Date;
  updatedAt: Date;
}

export interface PointRecord {
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
  roundingModeSnapshot: RoundingMode;
  points: number;
  note: string | null;
  createdAt: Date;
}

export interface PlayerStats {
  pubgAccountId: string;
  pubgPlayerName: string;
  damage: number;
  kills: number;
  assists: number;
  revives: number;
  teamId: number | null;
  placement: number | null;
}

export interface CalculatedPoints {
  pubgAccountId: string;
  pubgPlayerName: string;
  damage: number;
  kills: number;
  assists: number;
  revives: number;
  damagePoints: number;
  killPoints: number;
  revivePoints: number;
  totalPoints: number;
  isPointsEnabled: boolean;
}

export interface SyncStatus {
  isSyncing: boolean;
  lastSyncAt: Date | null;
  currentMatchId: string | null;
  error: string | null;
}

export interface AppStatus {
  version: string;
  databasePath: string;
  isDatabaseReady: boolean;
  syncStatus: SyncStatus;
}

export type NotificationEnvStatus =
  | 'unsupported_os'
  | 'missing_runtime'
  | 'runtime_not_running'
  | 'not_logged_in'
  | 'missing_group_id'
  | 'ready';

export type NotificationTaskStatus =
  | 'pending'
  | 'sending'
  | 'retrying'
  | 'failed_manual'
  | 'sent'
  | 'deleted'
  | 'cancelled_settled';

export type FailedNotificationSendStatus = 'sending' | 'sent' | 'failed';

export interface NotificationPageStatus {
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

export interface NotificationFailedTask {
  id: number;
  matchId: string;
  matchTime: Date;
  placement: number | null;
  battleSummary: string;
  lastError: string | null;
  sendStatus: FailedNotificationSendStatus;
}

export interface NotificationTemplateLineConfig {
  id: 'header' | 'player1' | 'player2' | 'player3' | 'player4' | 'battle';
  prefix: string;
  suffix: string;
}

export interface NotificationTemplateConfig {
  order: Array<'header' | 'player1' | 'player2' | 'player3' | 'player4' | 'battle'>;
  lines: Record<NotificationTemplateLineConfig['id'], NotificationTemplateLineConfig>;
}

export interface PointHistoryPlayerBreakdown {
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

export interface PointBattleDelta {
  matchPlayerId: number;
  teammateId: number | null;
  pubgPlayerName: string;
  displayNicknameSnapshot: string | null;
  delta: number;
}

export interface PointHistoryMatchGroup {
  type: 'match_group';
  matchId: string;
  playedAt: Date;
  mapName: string | null;
  gameMode: string | null;
  ruleId: number;
  ruleNameSnapshot: string;
  isSettled: boolean;
  settledAt: Date | null;
  settlementBatchId: number | null;
  note: string | null;
  players: PointHistoryPlayerBreakdown[];
  battleDeltas: PointBattleDelta[];
}

export interface PointHistoryRuleChangeMarker {
  type: 'rule_change_marker';
  previousRuleName: string;
  nextRuleName: string;
  createdAt: Date;
}

export type PointHistoryListItem =
  | PointHistoryMatchGroup
  | PointHistoryRuleChangeMarker;

export interface UnsettledPlayerSummary {
  teammateId: number | null;
  pubgPlayerName: string;
  displayNickname: string | null;
  isSelf: boolean;
  totalDelta: number;
}

export interface UnsettledBattleSummary {
  ruleId: number | null;
  activeRuleName: string | null;
  unsettledMatchCount: number;
  players: UnsettledPlayerSummary[];
}
