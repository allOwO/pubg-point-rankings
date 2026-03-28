/**
 * Tests for the telemetry parser
 */

import { test, describe } from 'node:test';
import assert from 'node:assert';
import {
  parseTelemetry,
  aggregatePlayerStats,
  getTeammates,
  filterRealPlayers,
  findPlayerByName,
  findPlayerByAccountId,
  type LogPlayerTakeDamage,
  type LogPlayerKillV2,
  type LogPlayerRevive,
  type LogMatchStart,
  type LogMatchEnd,
  type TelemetryData,
} from './telemetry';

describe('parseTelemetry', () => {
  test('parses valid JSON telemetry data', () => {
    const events: TelemetryData[] = [
      { _T: 'LogMatchStart', _D: '2024-01-01T00:00:00Z', characters: [] },
      { _T: 'LogMatchEnd', _D: '2024-01-01T00:30:00Z', characters: [] },
    ];

    const result = parseTelemetry(JSON.stringify(events));
    assert.strictEqual(result.length, 2);
    assert.strictEqual(result[0]._T, 'LogMatchStart');
  });

  test('throws on invalid JSON', () => {
    assert.throws(() => {
      parseTelemetry('invalid json');
    });
  });
});

describe('aggregatePlayerStats', () => {
  test('aggregates damage from LogPlayerTakeDamage', () => {
    const events: TelemetryData[] = [
      {
        _T: 'LogMatchStart',
        _D: '2024-01-01T00:00:00Z',
        characters: [
          { accountId: 'player1', name: 'Player 1', teamId: 1 },
          { accountId: 'player2', name: 'Player 2', teamId: 1 },
        ],
      } as LogMatchStart,
      {
        _T: 'LogPlayerTakeDamage',
        _D: '2024-01-01T00:01:00Z',
        attacker: { accountId: 'player1', name: 'Player 1' },
        victim: { accountId: 'player2', name: 'Player 2' },
        damage: 25.5,
        damageTypeCategory: 'Damage_Gun',
      } as LogPlayerTakeDamage,
      {
        _T: 'LogPlayerTakeDamage',
        _D: '2024-01-01T00:02:00Z',
        attacker: { accountId: 'player1', name: 'Player 1' },
        victim: { accountId: 'player2', name: 'Player 2' },
        damage: 30.0,
        damageTypeCategory: 'Damage_Gun',
      } as LogPlayerTakeDamage,
    ];

    const stats = aggregatePlayerStats(events);
    
    const player1 = stats.find(s => s.pubgAccountId === 'player1');
    assert.ok(player1);
    assert.strictEqual(player1!.damage, 55.5);
  });

  test('aggregates kills from LogPlayerKillV2', () => {
    const events: TelemetryData[] = [
      {
        _T: 'LogMatchStart',
        _D: '2024-01-01T00:00:00Z',
        characters: [
          { accountId: 'player1', name: 'Player 1', teamId: 1 },
        ],
      } as LogMatchStart,
      {
        _T: 'LogPlayerKillV2',
        _D: '2024-01-01T00:01:00Z',
        killer: { accountId: 'player1', name: 'Player 1' },
        victim: { accountId: 'victim1', name: 'Victim 1' },
      } as LogPlayerKillV2,
      {
        _T: 'LogPlayerKillV2',
        _D: '2024-01-01T00:02:00Z',
        killer: { accountId: 'player1', name: 'Player 1' },
        victim: { accountId: 'victim2', name: 'Victim 2' },
      } as LogPlayerKillV2,
    ];

    const stats = aggregatePlayerStats(events);
    
    const player1 = stats.find(s => s.pubgAccountId === 'player1');
    assert.ok(player1);
    assert.strictEqual(player1!.kills, 2);
  });

  test('aggregates revives from LogPlayerRevive', () => {
    const events: TelemetryData[] = [
      {
        _T: 'LogMatchStart',
        _D: '2024-01-01T00:00:00Z',
        characters: [
          { accountId: 'player1', name: 'Player 1', teamId: 1 },
        ],
      } as LogMatchStart,
      {
        _T: 'LogPlayerRevive',
        _D: '2024-01-01T00:01:00Z',
        reviver: { accountId: 'player1', name: 'Player 1' },
        victim: { accountId: 'player2', name: 'Player 2' },
      } as LogPlayerRevive,
      {
        _T: 'LogPlayerRevive',
        _D: '2024-01-01T00:02:00Z',
        reviver: { accountId: 'player1', name: 'Player 1' },
        victim: { accountId: 'player3', name: 'Player 3' },
      } as LogPlayerRevive,
    ];

    const stats = aggregatePlayerStats(events);
    
    const player1 = stats.find(s => s.pubgAccountId === 'player1');
    assert.ok(player1);
    assert.strictEqual(player1!.revives, 2);
  });

  test('captures placement from LogMatchEnd', () => {
    const events: TelemetryData[] = [
      {
        _T: 'LogMatchStart',
        _D: '2024-01-01T00:00:00Z',
        characters: [
          { accountId: 'player1', name: 'Player 1', teamId: 1 },
        ],
      } as LogMatchStart,
      {
        _T: 'LogMatchEnd',
        _D: '2024-01-01T00:30:00Z',
        characters: [
          { accountId: 'player1', name: 'Player 1', teamId: 1, ranking: 3 },
        ],
      } as LogMatchEnd,
    ];

    const stats = aggregatePlayerStats(events);
    
    const player1 = stats.find(s => s.pubgAccountId === 'player1');
    assert.ok(player1);
    assert.strictEqual(player1!.placement, 3);
  });

  test('ignores self-damage', () => {
    const events: TelemetryData[] = [
      {
        _T: 'LogMatchStart',
        _D: '2024-01-01T00:00:00Z',
        characters: [
          { accountId: 'player1', name: 'Player 1', teamId: 1 },
        ],
      } as LogMatchStart,
      {
        _T: 'LogPlayerTakeDamage',
        _D: '2024-01-01T00:01:00Z',
        attacker: { accountId: 'player1', name: 'Player 1' },
        victim: { accountId: 'player1', name: 'Player 1' },
        damage: 10.0,
        damageTypeCategory: 'Damage_Gun',
      } as LogPlayerTakeDamage,
      {
        _T: 'LogPlayerTakeDamage',
        _D: '2024-01-01T00:02:00Z',
        attacker: { accountId: 'player1', name: 'Player 1' },
        victim: { accountId: 'player2', name: 'Player 2' },
        damage: 25.0,
        damageTypeCategory: 'Damage_Gun',
      } as LogPlayerTakeDamage,
    ];

    const stats = aggregatePlayerStats(events);
    
    const player1 = stats.find(s => s.pubgAccountId === 'player1');
    assert.ok(player1);
    assert.strictEqual(player1!.damage, 25.0); // Only external damage counted
  });
});

describe('getTeammates', () => {
  const stats = [
    { pubgAccountId: 'p1', pubgPlayerName: 'Player 1', damage: 0, kills: 0, assists: 0, revives: 0, teamId: 1, placement: 3 },
    { pubgAccountId: 'p2', pubgPlayerName: 'Player 2', damage: 0, kills: 0, assists: 0, revives: 0, teamId: 1, placement: 3 },
    { pubgAccountId: 'p3', pubgPlayerName: 'Player 3', damage: 0, kills: 0, assists: 0, revives: 0, teamId: 2, placement: 5 },
  ];

  test('returns teammates on same team', () => {
    const teammates = getTeammates(stats, 'p1');
    assert.strictEqual(teammates.length, 2);
    assert.ok(teammates.some(t => t.pubgAccountId === 'p1'));
    assert.ok(teammates.some(t => t.pubgAccountId === 'p2'));
  });

  test('returns empty array if player not found', () => {
    const teammates = getTeammates(stats, 'unknown');
    assert.strictEqual(teammates.length, 0);
  });
});

describe('filterRealPlayers', () => {
  test('filters out AI/bot players', () => {
    const stats = [
      { pubgAccountId: 'player.steam.123', pubgPlayerName: 'Real Player', damage: 0, kills: 0, assists: 0, revives: 0, teamId: 1, placement: null },
      { pubgAccountId: 'ai.player.1', pubgPlayerName: 'AI Player', damage: 0, kills: 0, assists: 0, revives: 0, teamId: 1, placement: null },
      { pubgAccountId: 'bot_player_123', pubgPlayerName: 'Bot', damage: 0, kills: 0, assists: 0, revives: 0, teamId: 1, placement: null },
    ];

    const filtered = filterRealPlayers(stats);
    assert.strictEqual(filtered.length, 1);
    assert.strictEqual(filtered[0].pubgAccountId, 'player.steam.123');
  });
});

describe('findPlayerByName', () => {
  const stats = [
    { pubgAccountId: 'p1', pubgPlayerName: 'PlayerOne', damage: 0, kills: 0, assists: 0, revives: 0, teamId: 1, placement: null },
    { pubgAccountId: 'p2', pubgPlayerName: 'PlayerTwo', damage: 0, kills: 0, assists: 0, revives: 0, teamId: 1, placement: null },
  ];

  test('finds player by exact name', () => {
    const player = findPlayerByName(stats, 'PlayerOne');
    assert.ok(player);
    assert.strictEqual(player!.pubgAccountId, 'p1');
  });

  test('finds player by case-insensitive name', () => {
    const player = findPlayerByName(stats, 'playerone');
    assert.ok(player);
    assert.strictEqual(player!.pubgAccountId, 'p1');
  });

  test('returns undefined for unknown player', () => {
    const player = findPlayerByName(stats, 'Unknown');
    assert.strictEqual(player, undefined);
  });
});

describe('findPlayerByAccountId', () => {
  const stats = [
    { pubgAccountId: 'account123', pubgPlayerName: 'Player 1', damage: 0, kills: 0, assists: 0, revives: 0, teamId: 1, placement: null },
  ];

  test('finds player by account ID', () => {
    const player = findPlayerByAccountId(stats, 'account123');
    assert.ok(player);
    assert.strictEqual(player!.pubgPlayerName, 'Player 1');
  });

  test('returns undefined for unknown account', () => {
    const player = findPlayerByAccountId(stats, 'unknown');
    assert.strictEqual(player, undefined);
  });
});
