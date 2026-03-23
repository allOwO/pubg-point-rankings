/**
 * Tests for repositories
 * Uses an in-memory SQLite database for testing
 */

import { test, describe, beforeEach, afterEach } from 'node:test';
import assert from 'node:assert';
import { bootstrapDatabase } from '../db/migrations.js';
import {
  SettingsRepository,
  TeammatesRepository,
  RedbagRulesRepository,
  MatchesRepository,
  RedbagRecordsRepository,
} from './index.js';
import type { Platform } from '@pubg-point-rankings/shared';

// Mock Database for testing
class MockDatabase {
  private data = new Map<string, Map<string, unknown>>();
  private idCounters = new Map<string, number>();

  prepare(sql: string) {
    return {
      get: (...params: unknown[]) => this.mockGet(sql, params),
      run: (...params: unknown[]) => this.mockRun(sql, params),
      all: (...params: unknown[]) => this.mockAll(sql, params),
    };
  }

  transaction<T>(fn: (args: unknown[]) => T) {
    return (args: unknown[]) => fn(args);
  }

  exec() {
    // No-op for mock
  }

  private mockGet(sql: string, params: unknown[]) {
    // Simplified mock implementation
    return undefined;
  }

  private mockRun(sql: string, params: unknown[]) {
    // Simplified mock implementation
    return { lastInsertRowid: 1, changes: 1 };
  }

  private mockAll(sql: string, params: unknown[]) {
    // Simplified mock implementation
    return [];
  }
}

describe('SettingsRepository', () => {
  let repo: SettingsRepository;

  beforeEach(() => {
    const mockDb = new MockDatabase() as unknown as import('better-sqlite3').Database;
    repo = new SettingsRepository(mockDb);
  });

  test('getString returns default for missing key', () => {
    const value = repo.getString('nonexistent', 'default');
    assert.strictEqual(value, 'default');
  });

  test('getNumber returns default for missing key', () => {
    const value = repo.getNumber('nonexistent', 42);
    assert.strictEqual(value, 42);
  });

  test('getBoolean returns default for missing key', () => {
    const value = repo.getBoolean('nonexistent', true);
    assert.strictEqual(value, true);
  });
});

describe('TeammatesRepository', () => {
  let repo: TeammatesRepository;

  beforeEach(() => {
    const mockDb = new MockDatabase() as unknown as import('better-sqlite3').Database;
    repo = new TeammatesRepository(mockDb);
  });

  test('getAll returns empty array by default', () => {
    const teammates = repo.getAll();
    assert.ok(Array.isArray(teammates));
    assert.strictEqual(teammates.length, 0);
  });

  test('getById returns null for nonexistent teammate', () => {
    const teammate = repo.getById(999);
    assert.strictEqual(teammate, null);
  });
});

describe('RedbagRulesRepository', () => {
  let repo: RedbagRulesRepository;

  beforeEach(() => {
    const mockDb = new MockDatabase() as unknown as import('better-sqlite3').Database;
    repo = new RedbagRulesRepository(mockDb);
  });

  test('getAll returns empty array by default', () => {
    const rules = repo.getAll();
    assert.ok(Array.isArray(rules));
    assert.strictEqual(rules.length, 0);
  });

  test('getActive returns null when no active rule', () => {
    const rule = repo.getActive();
    assert.strictEqual(rule, null);
  });
});

describe('MatchesRepository', () => {
  let repo: MatchesRepository;

  class RecoveryTestMatchesRepository extends MatchesRepository {
    constructor(
      db: import('better-sqlite3').Database,
      private readonly syncingMatches: Array<{ matchId: string; updatedAt: Date; status: 'syncing' }>
    ) {
      super(db);
    }

    override getByStatus() {
      return this.syncingMatches.map((match, index) => ({
        id: index + 1,
        matchId: match.matchId,
        platform: 'steam' as Platform,
        mapName: null,
        gameMode: null,
        playedAt: new Date('2026-03-24T09:00:00.000Z'),
        matchStartAt: null,
        matchEndAt: null,
        telemetryUrl: null,
        status: match.status,
        createdAt: new Date('2026-03-24T09:00:00.000Z'),
        updatedAt: match.updatedAt,
      }));
    }
  }

  beforeEach(() => {
    const mockDb = new MockDatabase() as unknown as import('better-sqlite3').Database;
    repo = new MatchesRepository(mockDb);
  });

  test('getAll returns empty array by default', () => {
    const matches = repo.getAll();
    assert.ok(Array.isArray(matches));
    assert.strictEqual(matches.length, 0);
  });

  test('getById returns null for nonexistent match', () => {
    const match = repo.getById('nonexistent');
    assert.strictEqual(match, null);
  });

  test('resetSyncingMatches retries recent syncs and fails stale ones', () => {
    const mockDb = new MockDatabase() as unknown as import('better-sqlite3').Database;
    const now = new Date('2026-03-24T10:00:00.000Z');
    const updatedStatuses = new Map<string, string>();
    repo = new RecoveryTestMatchesRepository(mockDb, [
      { matchId: 'recent-match', updatedAt: new Date(now.getTime() - 2 * 60 * 1000), status: 'syncing' },
      { matchId: 'stale-match', updatedAt: new Date(now.getTime() - 20 * 60 * 1000), status: 'syncing' },
    ]);

    const originalUpdateStatus = repo.updateStatus.bind(repo);
    repo.updateStatus = (matchId, status) => {
      updatedStatuses.set(matchId, status);
      originalUpdateStatus(matchId, status);
    };

    const result = repo.resetSyncingMatches(10 * 60 * 1000, now);

    assert.deepStrictEqual(result, { retried: 1, failed: 1 });
    assert.strictEqual(updatedStatuses.get('recent-match'), 'detected');
    assert.strictEqual(updatedStatuses.get('stale-match'), 'failed');
  });
});

describe('RedbagRecordsRepository', () => {
  let repo: RedbagRecordsRepository;

  beforeEach(() => {
    const mockDb = new MockDatabase() as unknown as import('better-sqlite3').Database;
    repo = new RedbagRecordsRepository(mockDb);
  });

  test('getAll returns empty array by default', () => {
    const records = repo.getAll();
    assert.ok(Array.isArray(records));
    assert.strictEqual(records.length, 0);
  });

  test('getByMatch returns empty array for nonexistent match', () => {
    const records = repo.getByMatch('nonexistent');
    assert.ok(Array.isArray(records));
    assert.strictEqual(records.length, 0);
  });

  test('existsForMatch returns false for nonexistent match', () => {
    const exists = repo.existsForMatch('nonexistent');
    assert.strictEqual(exists, false);
  });
});
