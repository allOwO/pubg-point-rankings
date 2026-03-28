/**
 * Test Utilities and Helpers
 * Shared utilities for unit tests
 */

import { type PlayerStats } from '@pubg-point-rankings/shared';

/**
 * Create mock player stats for testing
 */
export function createMockPlayerStats(overrides: Partial<PlayerStats> = {}): PlayerStats {
  return {
    pubgAccountId: 'account.test.123',
    pubgPlayerName: 'TestPlayer',
    damage: 100,
    kills: 2,
    assists: 0,
    revives: 1,
    teamId: 1,
    placement: 5,
    ...overrides,
  };
}

/**
 * Create mock telemetry events for testing
 */
export function createMockTelemetryEvents() {
  return {
    matchStart: {
      _T: 'LogMatchStart',
      _D: new Date().toISOString(),
      characters: [
        { accountId: 'player1', name: 'Player 1', teamId: 1 },
        { accountId: 'player2', name: 'Player 2', teamId: 1 },
        { accountId: 'player3', name: 'Player 3', teamId: 1 },
        { accountId: 'player4', name: 'Player 4', teamId: 1 },
      ],
    },
    damage: (attackerId: string, victimId: string, damage: number) => ({
      _T: 'LogPlayerTakeDamage',
      _D: new Date().toISOString(),
      attacker: { accountId: attackerId, name: attackerId },
      victim: { accountId: victimId, name: victimId },
      damage,
      damageTypeCategory: 'Damage_Gun',
    }),
    kill: (killerId: string, victimId: string) => ({
      _T: 'LogPlayerKillV2',
      _D: new Date().toISOString(),
      killer: { accountId: killerId, name: killerId },
      victim: { accountId: victimId, name: victimId },
    }),
    revive: (reviverId: string, victimId: string) => ({
      _T: 'LogPlayerRevive',
      _D: new Date().toISOString(),
      reviver: { accountId: reviverId, name: reviverId },
      victim: { accountId: victimId, name: victimId },
    }),
    matchEnd: (placements: Array<{ accountId: string; ranking: number }>) => ({
      _T: 'LogMatchEnd',
      _D: new Date().toISOString(),
      characters: placements.map(p => ({
        accountId: p.accountId,
        name: p.accountId,
        teamId: 1,
        ranking: p.ranking,
      })),
    }),
  };
}

/**
 * Assert that two numbers are approximately equal (for floating point comparisons)
 */
export function assertApproxEqual(
  actual: number,
  expected: number,
  tolerance: number = 0.001,
  message?: string
): void {
  const diff = Math.abs(actual - expected);
  if (diff > tolerance) {
    throw new Error(
      message || `Expected ${actual} to be approximately ${expected} (diff: ${diff}, tolerance: ${tolerance})`
    );
  }
}
