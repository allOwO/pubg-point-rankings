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

export interface Teammate {
  id: number;
  platform: Platform;
  pubgAccountId: string | null;
  pubgPlayerName: string;
  displayNickname: string | null;
  isRedbagEnabled: boolean;
  totalRedbagCents: number;
  lastSeenAt: Date | null;
  createdAt: Date;
  updatedAt: Date;
}

export interface Match {
  id: number;
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
  matchId: string;
  teammateId: number | null;
  pubgAccountId: string | null;
  pubgPlayerName: string;
  displayNicknameSnapshot: string | null;
  teamId: number | null;
  damage: number;
  kills: number;
  revives: number;
  placement: number | null;
  isSelf: boolean;
  isRedbagEnabledSnapshot: boolean;
  redbagCents: number;
  createdAt: Date;
}

export interface RedbagRule {
  id: number;
  name: string;
  damageCentPerPoint: number;
  killCent: number;
  reviveCent: number;
  isActive: boolean;
  roundingMode: RoundingMode;
  createdAt: Date;
  updatedAt: Date;
}

export interface RedbagRecord {
  id: number;
  matchId: string;
  matchPlayerId: number;
  teammateId: number | null;
  ruleId: number;
  ruleNameSnapshot: string;
  damageCentPerPointSnapshot: number;
  killCentSnapshot: number;
  reviveCentSnapshot: number;
  roundingModeSnapshot: RoundingMode;
  amountCents: number;
  note: string | null;
  createdAt: Date;
}

export interface PlayerStats {
  pubgAccountId: string;
  pubgPlayerName: string;
  damage: number;
  kills: number;
  revives: number;
  teamId: number | null;
  placement: number | null;
}

export interface CalculatedRedbag {
  pubgAccountId: string;
  pubgPlayerName: string;
  damage: number;
  kills: number;
  revives: number;
  damageCents: number;
  killsCents: number;
  revivesCents: number;
  totalCents: number;
  isRedbagEnabled: boolean;
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
