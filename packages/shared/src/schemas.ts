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
  isPointsEnabled: z.boolean(),
  totalPoints: z.number().int(),
  lastSeenAt: z.date().nullable(),
  createdAt: z.date(),
  updatedAt: z.date(),
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
  revives: z.number().int(),
  placement: z.number().int().nullable(),
  isSelf: z.boolean(),
  isPointsEnabledSnapshot: z.boolean(),
  points: z.number().int(),
  createdAt: z.date(),
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
  revives: z.number().int(),
  teamId: z.number().int().nullable(),
  placement: z.number().int().nullable(),
});

export const CalculatedPointsSchema = z.object({
  pubgAccountId: z.string(),
  pubgPlayerName: z.string(),
  damage: z.number(),
  kills: z.number().int(),
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
