/**
 * Zod schemas for PUBG Point Rankings
 * 
 * These schemas are used for runtime validation of data.
 */

import { z } from 'zod';

// Enums
export const PlatformSchema = z.enum(['steam', 'xbox', 'psn', 'kakao']);

export const MatchStatusSchema = z.enum(['detected', 'syncing', 'ready', 'failed']);

export const RoundingModeSchema = z.enum(['floor', 'round', 'ceil']);

export const PollingModeSchema = z.enum(['game', 'manual', 'auto']);

export const ManualSyncTaskStateSchema = z.enum(['idle', 'syncing', 'success', 'failed']);

export const ManualSyncTaskStatusSchema = z.object({
  state: ManualSyncTaskStateSchema,
  startedAt: z.date().nullable(),
  finishedAt: z.date().nullable(),
  errorMessage: z.string().nullable(),
  trigger: z.literal('manual'),
});

export const NotificationEnvStatusSchema = z.enum([
  'unsupported_os',
  'missing_runtime',
  'runtime_not_running',
  'not_logged_in',
  'missing_group_id',
  'ready',
]);

export const NotificationTaskStatusSchema = z.enum([
  'pending',
  'sending',
  'retrying',
  'failed_manual',
  'sent',
  'deleted',
  'cancelled_settled',
]);

export const FailedNotificationSendStatusSchema = z.enum(['sending', 'sent', 'failed']);

export const NotificationTemplateLineIdSchema = z.enum([
  'header',
  'player1',
  'player2',
  'player3',
  'player4',
  'battle',
]);

// Base schemas
export const AppSettingSchema = z.object({
  key: z.string(),
  value: z.string(),
  updatedAt: z.date(),
});

export const AccountSchema = z.object({
  id: z.number(),
  accountName: z.string(),
  selfPlayerName: z.string(),
  selfPlatform: PlatformSchema,
  pubgApiKey: z.string(),
  isActive: z.boolean(),
  createdAt: z.date(),
  updatedAt: z.date(),
});

export const TeammateSchema = z.object({
  id: z.number(),
  accountId: z.number(),
  platform: PlatformSchema,
  pubgAccountId: z.string().nullable(),
  pubgPlayerName: z.string(),
  displayNickname: z.string().nullable(),
  isFriend: z.boolean(),
  isPointsEnabled: z.boolean(),
  totalPoints: z.number().int(),
  lastSeenAt: z.date().nullable(),
  createdAt: z.date(),
  updatedAt: z.date(),
});

export const RecentTeammateCandidateSchema = z.object({
  platform: PlatformSchema,
  pubgAccountId: z.string().nullable(),
  pubgPlayerName: z.string(),
  lastTeammateAt: z.date(),
  isFriend: z.boolean(),
});

export const MatchSchema = z.object({
  id: z.number(),
  accountId: z.number(),
  matchId: z.string(),
  platform: PlatformSchema,
  mapName: z.string().nullable(),
  gameMode: z.string().nullable(),
  playedAt: z.date(),
  matchStartAt: z.date().nullable(),
  matchEndAt: z.date().nullable(),
  telemetryUrl: z.string().nullable(),
  status: MatchStatusSchema,
  createdAt: z.date(),
  updatedAt: z.date(),
});

export const MatchPlayerSchema = z.object({
  id: z.number(),
  accountId: z.number(),
  matchId: z.string(),
  teammateId: z.number().nullable(),
  pubgAccountId: z.string().nullable(),
  pubgPlayerName: z.string(),
  displayNicknameSnapshot: z.string().nullable(),
  teamId: z.number().nullable(),
  damage: z.number(),
  kills: z.number().int(),
  assists: z.number().int().optional(),
  revives: z.number().int(),
  placement: z.number().int().nullable(),
  isSelf: z.boolean(),
  isPointsEnabledSnapshot: z.boolean(),
  points: z.number().int(),
  createdAt: z.date(),
});

export const MatchDamageEventSchema = z.object({
  id: z.number(),
  accountId: z.number(),
  matchId: z.string(),
  attackerAccountId: z.string().nullable(),
  attackerName: z.string().nullable(),
  victimAccountId: z.string().nullable(),
  victimName: z.string().nullable(),
  damage: z.number(),
  damageTypeCategory: z.string().nullable(),
  damageCauserName: z.string().nullable(),
  eventAt: z.date().nullable(),
  createdAt: z.date(),
});

export const MatchKillEventSchema = z.object({
  id: z.number(),
  accountId: z.number(),
  matchId: z.string(),
  killerAccountId: z.string().nullable(),
  killerName: z.string().nullable(),
  victimAccountId: z.string().nullable(),
  victimName: z.string().nullable(),
  assistantAccountId: z.string().nullable(),
  assistantName: z.string().nullable(),
  damageTypeCategory: z.string().nullable(),
  damageCauserName: z.string().nullable(),
  eventAt: z.date().nullable(),
  createdAt: z.date(),
});

export const MatchKnockEventSchema = z.object({
  id: z.number(),
  accountId: z.number(),
  matchId: z.string(),
  attackerAccountId: z.string().nullable(),
  attackerName: z.string().nullable(),
  victimAccountId: z.string().nullable(),
  victimName: z.string().nullable(),
  damageTypeCategory: z.string().nullable(),
  damageCauserName: z.string().nullable(),
  eventAt: z.date().nullable(),
  createdAt: z.date(),
});

export const MatchReviveEventSchema = z.object({
  id: z.number(),
  accountId: z.number(),
  matchId: z.string(),
  reviverAccountId: z.string().nullable(),
  reviverName: z.string().nullable(),
  victimAccountId: z.string().nullable(),
  victimName: z.string().nullable(),
  eventAt: z.date().nullable(),
  createdAt: z.date(),
});

export const MatchPlayerWeaponStatSchema = z.object({
  id: z.number(),
  accountId: z.number(),
  matchId: z.string(),
  pubgAccountId: z.string().nullable(),
  pubgPlayerName: z.string(),
  weaponName: z.string(),
  totalDamage: z.number(),
  createdAt: z.date(),
});

export const MatchDetailSchema = z.object({
  match: MatchSchema,
  players: z.array(MatchPlayerSchema),
  damageEvents: z.array(MatchDamageEventSchema),
  killEvents: z.array(MatchKillEventSchema),
  knockEvents: z.array(MatchKnockEventSchema),
  reviveEvents: z.array(MatchReviveEventSchema),
  weaponStats: z.array(MatchPlayerWeaponStatSchema),
});

export const PointRuleSchema = z.object({
  id: z.number(),
  accountId: z.number(),
  name: z.string(),
  damagePointsPerDamage: z.number().int(),
  killPoints: z.number().int(),
  revivePoints: z.number().int(),
  isActive: z.boolean(),
  roundingMode: RoundingModeSchema,
  createdAt: z.date(),
  updatedAt: z.date(),
});

export const PointRecordSchema = z.object({
  id: z.number(),
  accountId: z.number(),
  matchId: z.string(),
  matchPlayerId: z.number(),
  teammateId: z.number().nullable(),
  ruleId: z.number(),
  ruleNameSnapshot: z.string(),
  damagePointsPerDamageSnapshot: z.number().int(),
  killPointsSnapshot: z.number().int(),
  revivePointsSnapshot: z.number().int(),
  roundingModeSnapshot: RoundingModeSchema,
  points: z.number().int(),
  note: z.string().nullable(),
  createdAt: z.date(),
});

export const PlayerStatsSchema = z.object({
  pubgAccountId: z.string(),
  pubgPlayerName: z.string(),
  damage: z.number(),
  kills: z.number().int(),
  assists: z.number().int().optional(),
  revives: z.number().int(),
  teamId: z.number().int().nullable(),
  placement: z.number().int().nullable(),
});

export const CalculatedPointsSchema = z.object({
  pubgAccountId: z.string(),
  pubgPlayerName: z.string(),
  damage: z.number(),
  kills: z.number().int(),
  assists: z.number().int().optional(),
  revives: z.number().int(),
  damagePoints: z.number().int(),
  killPoints: z.number().int(),
  revivePoints: z.number().int(),
  totalPoints: z.number().int(),
  isPointsEnabled: z.boolean(),
});

export const SyncStatusSchema = z.object({
  isSyncing: z.boolean(),
  lastSyncAt: z.date().nullable(),
  currentMatchId: z.string().nullable(),
  error: z.string().nullable(),
});

export const AppStatusSchema = z.object({
  version: z.string(),
  databasePath: z.string(),
  isDatabaseReady: z.boolean(),
  syncStatus: SyncStatusSchema,
});

export const NotificationPageStatusSchema = z.object({
  envStatus: NotificationEnvStatusSchema,
  isEnabled: z.boolean(),
  canInstallRuntime: z.boolean(),
  runtimeVersion: z.string(),
  installDir: z.string().nullable(),
  webUiUrl: z.string().nullable(),
  oneBotUrl: z.string().nullable(),
  qqNumber: z.string().nullable(),
  groupId: z.string(),
  lastError: z.string().nullable(),
});

export const NotificationFailedTaskSchema = z.object({
  id: z.number(),
  matchId: z.string(),
  matchTime: z.date(),
  placement: z.number().int().nullable(),
  battleSummary: z.string(),
  lastError: z.string().nullable(),
  sendStatus: FailedNotificationSendStatusSchema,
});

export const NotificationTemplateLineConfigSchema = z.object({
  id: NotificationTemplateLineIdSchema,
  prefix: z.string(),
  suffix: z.string(),
});

export const NotificationTemplateConfigSchema = z.object({
  order: z.array(NotificationTemplateLineIdSchema),
  lines: z.object({
    header: NotificationTemplateLineConfigSchema,
    player1: NotificationTemplateLineConfigSchema,
    player2: NotificationTemplateLineConfigSchema,
    player3: NotificationTemplateLineConfigSchema,
    player4: NotificationTemplateLineConfigSchema,
    battle: NotificationTemplateLineConfigSchema,
  }),
});

export const PointHistoryPlayerBreakdownSchema = z.object({
  matchPlayerId: z.number(),
  teammateId: z.number().nullable(),
  pubgAccountId: z.string().nullable(),
  pubgPlayerName: z.string(),
  displayNicknameSnapshot: z.string().nullable(),
  isSelf: z.boolean(),
  isPointsEnabledSnapshot: z.boolean(),
  damage: z.number(),
  kills: z.number().int(),
  revives: z.number().int(),
  damagePointsPerDamageSnapshot: z.number().int(),
  killPointsSnapshot: z.number().int(),
  revivePointsSnapshot: z.number().int(),
  damagePoints: z.number().int(),
  killPoints: z.number().int(),
  revivePoints: z.number().int(),
  totalPoints: z.number().int(),
});

export const PointBattleDeltaSchema = z.object({
  matchPlayerId: z.number(),
  teammateId: z.number().nullable(),
  pubgPlayerName: z.string(),
  displayNicknameSnapshot: z.string().nullable(),
  delta: z.number().int(),
});

export const PointHistoryMatchGroupSchema = z.object({
  type: z.literal('match_group'),
  matchId: z.string(),
  playedAt: z.date(),
  mapName: z.string().nullable(),
  gameMode: z.string().nullable(),
  ruleId: z.number(),
  ruleNameSnapshot: z.string(),
  isSettled: z.boolean(),
  settledAt: z.date().nullable(),
  settlementBatchId: z.number().nullable(),
  note: z.string().nullable(),
  players: z.array(PointHistoryPlayerBreakdownSchema),
  battleDeltas: z.array(PointBattleDeltaSchema),
});

export const PointHistoryRuleChangeMarkerSchema = z.object({
  type: z.literal('rule_change_marker'),
  previousRuleName: z.string(),
  nextRuleName: z.string(),
  createdAt: z.date(),
});

export const PointHistoryListItemSchema = z.discriminatedUnion('type', [
  PointHistoryMatchGroupSchema,
  PointHistoryRuleChangeMarkerSchema,
]);

export const UnsettledPlayerSummarySchema = z.object({
  teammateId: z.number().nullable(),
  pubgPlayerName: z.string(),
  displayNickname: z.string().nullable(),
  isSelf: z.boolean(),
  totalDelta: z.number().int(),
});

export const UnsettledBattleSummarySchema = z.object({
  ruleId: z.number().nullable(),
  activeRuleName: z.string().nullable(),
  unsettledMatchCount: z.number().int(),
  players: z.array(UnsettledPlayerSummarySchema),
});

export const RecalculateUnsettledPointsInputSchema = z.object({
  ruleId: z.number().int().positive(),
});

export const RecalculateUnsettledPointsResultSchema = z.object({
  ruleId: z.number().int(),
  ruleName: z.string(),
  recalculatedMatchCount: z.number().int(),
});

export const UpdatePointMatchNoteInputSchema = z.object({
  matchId: z.string(),
  note: z.string().nullable(),
});

export const SettlePointMatchesInputSchema = z.object({
  endMatchId: z.string(),
});

// Input schemas for create/update operations

export const CreateTeammateInputSchema = z.object({
  platform: PlatformSchema,
  pubgAccountId: z.string().nullable(),
  pubgPlayerName: z.string().min(1),
  displayNickname: z.string().nullable().optional(),
  isPointsEnabled: z.boolean().optional().default(true),
});

export const UpdateTeammateInputSchema = z.object({
  id: z.number(),
  displayNickname: z.string().nullable().optional(),
  isPointsEnabled: z.boolean().optional(),
});

export const CreatePointRuleInputSchema = z.object({
  name: z.string().min(1),
  damagePointsPerDamage: z.number().int().min(0),
  killPoints: z.number().int().min(0),
  revivePoints: z.number().int().min(0),
  roundingMode: RoundingModeSchema.default('round'),
});

export const UpdatePointRuleInputSchema = z.object({
  id: z.number(),
  name: z.string().min(1).optional(),
  damagePointsPerDamage: z.number().int().min(0).optional(),
  killPoints: z.number().int().min(0).optional(),
  revivePoints: z.number().int().min(0).optional(),
  roundingMode: RoundingModeSchema.optional(),
});

export const CreateMatchInputSchema = z.object({
  matchId: z.string().min(1),
  platform: PlatformSchema,
  mapName: z.string().nullable().optional(),
  gameMode: z.string().nullable().optional(),
  playedAt: z.date(),
  matchStartAt: z.date().nullable().optional(),
  matchEndAt: z.date().nullable().optional(),
  telemetryUrl: z.string().nullable().optional(),
});

export const UpdateSettingsInputSchema = z.record(z.string());

// Type exports
export type CreateTeammateInput = z.infer<typeof CreateTeammateInputSchema>;
export type UpdateTeammateInput = z.infer<typeof UpdateTeammateInputSchema>;
export type CreatePointRuleInput = z.infer<typeof CreatePointRuleInputSchema>;
export type UpdatePointRuleInput = z.infer<typeof UpdatePointRuleInputSchema>;
export type CreateMatchInput = z.infer<typeof CreateMatchInputSchema>;
export type UpdateSettingsInput = z.infer<typeof UpdateSettingsInputSchema>;
export type UpdatePointMatchNoteInput = z.infer<typeof UpdatePointMatchNoteInputSchema>;
export type SettlePointMatchesInput = z.infer<typeof SettlePointMatchesInputSchema>;
export type RecalculateUnsettledPointsInput = z.infer<typeof RecalculateUnsettledPointsInputSchema>;
export type RecalculateUnsettledPointsResult = z.infer<typeof RecalculateUnsettledPointsResultSchema>;
