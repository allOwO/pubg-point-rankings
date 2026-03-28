/**
 * Tests for the points calculator
 */

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { 
  applyRounding, 
  calculatePlayerPoints,
  calculatePoints,
  calculateTotalPoints,
  formatPoints,
  parsePoints,
} from './calculator';
import type { PointRule, PlayerStats } from '@pubg-point-rankings/shared';

describe('applyRounding', () => {
  test('floor rounds down', () => {
    assert.strictEqual(applyRounding(10.9, 'floor'), 10);
    assert.strictEqual(applyRounding(10.1, 'floor'), 10);
  });

  test('ceil rounds up', () => {
    assert.strictEqual(applyRounding(10.1, 'ceil'), 11);
    assert.strictEqual(applyRounding(10.9, 'ceil'), 11);
  });

  test('round rounds to nearest', () => {
    assert.strictEqual(applyRounding(10.4, 'round'), 10);
    assert.strictEqual(applyRounding(10.5, 'round'), 11);
    assert.strictEqual(applyRounding(10.6, 'round'), 11);
  });
});

describe('calculatePlayerPoints', () => {
  const rule: PointRule = {
    id: 1,
    accountId: 1,
    name: 'Test Rule',
    damagePointsPerDamage: 2,
    killPoints: 300,
    revivePoints: 150,
    isActive: true,
    roundingMode: 'round',
    createdAt: new Date(),
    updatedAt: new Date(),
  };

  const player: PlayerStats = {
    pubgAccountId: 'account123',
    pubgPlayerName: 'TestPlayer',
    damage: 256.4,
    kills: 3,
    assists: 1,
    revives: 2,
    teamId: 1,
    placement: 5,
  };

  test('calculates correct amounts for enabled player', () => {
    const result = calculatePlayerPoints(player, rule, true);

    assert.strictEqual(result.pubgAccountId, 'account123');
    assert.strictEqual(result.pubgPlayerName, 'TestPlayer');
    assert.strictEqual(result.damage, 256.4);
    assert.strictEqual(result.kills, 3);
    assert.strictEqual(result.revives, 2);
    assert.strictEqual(result.damagePoints, 513); // 256.4 * 2 = 512.8 -> round to 513
    assert.strictEqual(result.killPoints, 900); // 3 * 300
    assert.strictEqual(result.revivePoints, 300); // 2 * 150
    assert.strictEqual(result.totalPoints, 1713); // 513 + 900 + 300
    assert.strictEqual(result.isPointsEnabled, true);
  });

  test('returns zero for disabled player', () => {
    const result = calculatePlayerPoints(player, rule, false);

    assert.strictEqual(result.damagePoints, 513);
    assert.strictEqual(result.killPoints, 900);
    assert.strictEqual(result.revivePoints, 300);
    assert.strictEqual(result.totalPoints, 0); // Disabled = no payout
    assert.strictEqual(result.isPointsEnabled, false);
  });
});

describe('calculatePoints', () => {
  const rule: PointRule = {
    id: 1,
    accountId: 1,
    name: 'Test Rule',
    damagePointsPerDamage: 2,
    killPoints: 300,
    revivePoints: 150,
    isActive: true,
    roundingMode: 'round',
    createdAt: new Date(),
    updatedAt: new Date(),
  };

  const players: PlayerStats[] = [
    {
      pubgAccountId: 'player1',
      pubgPlayerName: 'Player 1',
      damage: 100,
      kills: 2,
      assists: 0,
      revives: 1,
      teamId: 1,
      placement: 3,
    },
    {
      pubgAccountId: 'player2',
      pubgPlayerName: 'Player 2',
      damage: 200,
      kills: 1,
      assists: 0,
      revives: 0,
      teamId: 1,
      placement: 3,
    },
    {
      pubgAccountId: 'player3',
      pubgPlayerName: 'Player 3',
      damage: 50,
      kills: 0,
      assists: 0,
      revives: 2,
      teamId: 1,
      placement: 3,
    },
  ];

  test('calculates points for all players', () => {
    const enabledIds = new Set(['player1', 'player2', 'player3']);
    const results = calculatePoints({ rule, players, enabledPlayerIds: enabledIds });

    assert.strictEqual(results.length, 3);
    
    // Player 1: 200 + 600 + 150 = 950
    assert.strictEqual(results[0].totalPoints, 950);
    
    // Player 2: 400 + 300 + 0 = 700
    assert.strictEqual(results[1].totalPoints, 700);
    
    // Player 3: 100 + 0 + 300 = 400
    assert.strictEqual(results[2].totalPoints, 400);
  });

  test('excludes disabled players from payout', () => {
    const enabledIds = new Set(['player1', 'player2']); // player3 disabled
    const results = calculatePoints({ rule, players, enabledPlayerIds: enabledIds });

    assert.strictEqual(results[0].totalPoints, 950);
    assert.strictEqual(results[1].totalPoints, 700);
    assert.strictEqual(results[2].totalPoints, 0); // Disabled
    assert.strictEqual(results[2].isPointsEnabled, false);
  });
});

describe('calculateTotalPoints', () => {
  test('sums all point amounts', () => {
    const points = [
      {
        pubgAccountId: 'p1',
        pubgPlayerName: 'Player 1',
        damage: 100,
        kills: 1,
        assists: 0,
        revives: 0,
        damagePoints: 200,
        killPoints: 300,
        revivePoints: 0,
        totalPoints: 500,
        isPointsEnabled: true,
      },
      {
        pubgAccountId: 'p2',
        pubgPlayerName: 'Player 2',
        damage: 200,
        kills: 2,
        assists: 0,
        revives: 1,
        damagePoints: 400,
        killPoints: 600,
        revivePoints: 150,
        totalPoints: 1150,
        isPointsEnabled: true,
      },
    ];

    assert.strictEqual(calculateTotalPoints(points), 1650);
  });

  test('returns zero for empty array', () => {
    assert.strictEqual(calculateTotalPoints([]), 0);
  });
});

describe('formatPoints', () => {
  test('formats points string', () => {
    assert.strictEqual(formatPoints(100), '100 pts');
    assert.strictEqual(formatPoints(150), '150 pts');
    assert.strictEqual(formatPoints(0), '0 pts');
    assert.strictEqual(formatPoints(1234), '1,234 pts');
  });
});

describe('parsePoints', () => {
  test('parses points string to integer points', () => {
    assert.strictEqual(parsePoints('100'), 100);
    assert.strictEqual(parsePoints('150'), 150);
    assert.strictEqual(parsePoints('150 pts'), 150);
    assert.strictEqual(parsePoints('1,234 pts'), 1234);
  });

  test('handles invalid input', () => {
    assert.strictEqual(parsePoints('invalid'), 0);
    assert.strictEqual(parsePoints(''), 0);
  });
});
