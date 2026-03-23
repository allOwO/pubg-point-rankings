/**
 * Teammates Repository
 * Handles CRUD operations for teammates table
 */

import type { Database } from 'better-sqlite3';
import type { Teammate, Platform, CreateTeammateInput, UpdateTeammateInput } from '@pubg-point-rankings/shared';

interface TeammateRow {
  id: number;
  platform: string;
  pubg_account_id: string | null;
  pubg_player_name: string;
  display_nickname: string | null;
  is_redbag_enabled: number;
  total_redbag_cents: number;
  last_seen_at: string | null;
  created_at: string;
  updated_at: string;
}

function mapRowToTeammate(row: TeammateRow): Teammate {
  return {
    id: row.id,
    platform: row.platform as Platform,
    pubgAccountId: row.pubg_account_id,
    pubgPlayerName: row.pubg_player_name,
    displayNickname: row.display_nickname,
    isRedbagEnabled: row.is_redbag_enabled === 1,
    totalRedbagCents: row.total_redbag_cents,
    lastSeenAt: row.last_seen_at ? new Date(row.last_seen_at) : null,
    createdAt: new Date(row.created_at),
    updatedAt: new Date(row.updated_at),
  };
}

export class TeammatesRepository {
  constructor(private db: Database) {}

  /**
   * Get all teammates
   */
  getAll(): Teammate[] {
    const rows = this.db.prepare(
      'SELECT * FROM teammates ORDER BY pubg_player_name'
    ).all() as TeammateRow[];
    
    return rows.map(mapRowToTeammate);
  }

  /**
   * Get teammate by ID
   */
  getById(id: number): Teammate | null {
    const row = this.db.prepare('SELECT * FROM teammates WHERE id = ?').get(id) as TeammateRow | undefined;
    return row ? mapRowToTeammate(row) : null;
  }

  /**
   * Get teammate by PUBG player name and platform
   */
  getByNameAndPlatform(playerName: string, platform: Platform): Teammate | null {
    const row = this.db.prepare(
      'SELECT * FROM teammates WHERE pubg_player_name = ? AND platform = ?'
    ).get(playerName, platform) as TeammateRow | undefined;
    return row ? mapRowToTeammate(row) : null;
  }

  /**
   * Get teammate by PUBG account ID
   */
  getByAccountId(accountId: string): Teammate | null {
    const row = this.db.prepare(
      'SELECT * FROM teammates WHERE pubg_account_id = ?'
    ).get(accountId) as TeammateRow | undefined;
    return row ? mapRowToTeammate(row) : null;
  }

  /**
   * Create a new teammate
   */
  create(input: CreateTeammateInput): Teammate {
    const result = this.db.prepare(
      `INSERT INTO teammates 
       (platform, pubg_account_id, pubg_player_name, display_nickname, is_redbag_enabled, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)`
    ).run(
      input.platform,
      input.pubgAccountId,
      input.pubgPlayerName,
      input.displayNickname ?? null,
      input.isRedbagEnabled ? 1 : 0
    );

    const teammate = this.getById(result.lastInsertRowid as number);
    if (!teammate) {
      throw new Error('Failed to create teammate');
    }
    return teammate;
  }

  /**
   * Update a teammate
   */
  update(input: UpdateTeammateInput): Teammate {
    const sets: string[] = [];
    const params: (string | number | null)[] = [];

    if (input.displayNickname !== undefined) {
      sets.push('display_nickname = ?');
      params.push(input.displayNickname);
    }
    if (input.isRedbagEnabled !== undefined) {
      sets.push('is_redbag_enabled = ?');
      params.push(input.isRedbagEnabled ? 1 : 0);
    }

    if (sets.length === 0) {
      const existing = this.getById(input.id);
      if (!existing) throw new Error('Teammate not found');
      return existing;
    }

    sets.push('updated_at = CURRENT_TIMESTAMP');
    params.push(input.id);

    this.db.prepare(
      `UPDATE teammates SET ${sets.join(', ')} WHERE id = ?`
    ).run(...params);

    const teammate = this.getById(input.id);
    if (!teammate) {
      throw new Error('Teammate not found after update');
    }
    return teammate;
  }

  /**
   * Update total redbag cents for a teammate
   */
  updateTotalRedbagCents(id: number, cents: number): void {
    this.db.prepare(
      'UPDATE teammates SET total_redbag_cents = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?'
    ).run(cents, id);
  }

  /**
   * Update last seen timestamp
   */
  updateLastSeen(id: number): void {
    this.db.prepare(
      'UPDATE teammates SET last_seen_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ?'
    ).run(id);
  }

  /**
   * Find or create a teammate by player name and platform
   */
  findOrCreate(playerName: string, platform: Platform, accountId?: string): Teammate {
    let teammate = this.getByNameAndPlatform(playerName, platform);
    
    if (!teammate && accountId) {
      teammate = this.getByAccountId(accountId);
    }
    
    if (!teammate) {
      teammate = this.create({
        platform,
        pubgAccountId: accountId ?? null,
        pubgPlayerName: playerName,
        isRedbagEnabled: true,
      });
    } else if (accountId && !teammate.pubgAccountId) {
      // Update account ID if we now have it
      this.db.prepare(
        'UPDATE teammates SET pubg_account_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?'
      ).run(accountId, teammate.id);
      teammate.pubgAccountId = accountId;
    }
    
    return teammate;
  }
}
