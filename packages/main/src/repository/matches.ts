/**
 * Matches Repository
 * Handles CRUD operations for matches and match_players tables
 */

import type { Database } from 'better-sqlite3';
import type { Match, MatchPlayer, MatchStatus, Platform } from '@pubg-point-rankings/shared';

interface MatchRow {
  id: number;
  match_id: string;
  platform: string;
  map_name: string | null;
  game_mode: string | null;
  played_at: string;
  match_start_at: string | null;
  match_end_at: string | null;
  telemetry_url: string | null;
  status: string;
  created_at: string;
  updated_at: string;
}

interface MatchPlayerRow {
  id: number;
  match_id: string;
  teammate_id: number | null;
  pubg_account_id: string | null;
  pubg_player_name: string;
  display_nickname_snapshot: string | null;
  team_id: number | null;
  damage: number;
  kills: number;
  revives: number;
  placement: number | null;
  is_self: number;
  is_redbag_enabled_snapshot: number;
  redbag_cents: number;
  created_at: string;
}

function mapRowToMatch(row: MatchRow): Match {
  return {
    id: row.id,
    matchId: row.match_id,
    platform: row.platform as Platform,
    mapName: row.map_name,
    gameMode: row.game_mode,
    playedAt: new Date(row.played_at),
    matchStartAt: row.match_start_at ? new Date(row.match_start_at) : null,
    matchEndAt: row.match_end_at ? new Date(row.match_end_at) : null,
    telemetryUrl: row.telemetry_url,
    status: row.status as MatchStatus,
    createdAt: new Date(row.created_at),
    updatedAt: new Date(row.updated_at),
  };
}

function mapRowToMatchPlayer(row: MatchPlayerRow): MatchPlayer {
  return {
    id: row.id,
    matchId: row.match_id,
    teammateId: row.teammate_id,
    pubgAccountId: row.pubg_account_id,
    pubgPlayerName: row.pubg_player_name,
    displayNicknameSnapshot: row.display_nickname_snapshot,
    teamId: row.team_id,
    damage: row.damage,
    kills: row.kills,
    revives: row.revives,
    placement: row.placement,
    isSelf: row.is_self === 1,
    isRedbagEnabledSnapshot: row.is_redbag_enabled_snapshot === 1,
    redbagCents: row.redbag_cents,
    createdAt: new Date(row.created_at),
  };
}

export interface CreateMatchInput {
  matchId: string;
  platform: Platform;
  mapName?: string | null;
  gameMode?: string | null;
  playedAt: Date;
  matchStartAt?: Date | null;
  matchEndAt?: Date | null;
  telemetryUrl?: string | null;
}

export interface CreateMatchPlayerInput {
  matchId: string;
  teammateId?: number | null;
  pubgAccountId?: string | null;
  pubgPlayerName: string;
  displayNicknameSnapshot?: string | null;
  teamId?: number | null;
  damage?: number;
  kills?: number;
  revives?: number;
  placement?: number | null;
  isSelf?: boolean;
  isRedbagEnabledSnapshot?: boolean;
  redbagCents?: number;
}

export class MatchesRepository {
  constructor(private db: Database) {}

  /**
   * Get all matches with pagination
   */
  getAll(limit: number = 100, offset: number = 0): Match[] {
    const rows = this.db.prepare(
      'SELECT * FROM matches ORDER BY played_at DESC LIMIT ? OFFSET ?'
    ).all(limit, offset) as MatchRow[];
    
    return rows.map(mapRowToMatch);
  }

  /**
   * Get match by ID
   */
  getById(matchId: string): Match | null {
    const row = this.db.prepare(
      'SELECT * FROM matches WHERE match_id = ?'
    ).get(matchId) as MatchRow | undefined;
    
    return row ? mapRowToMatch(row) : null;
  }

  /**
   * Create a new match
   */
  create(input: CreateMatchInput): Match {
    try {
      this.db.prepare(
        `INSERT INTO matches 
         (match_id, platform, map_name, game_mode, played_at, match_start_at, match_end_at, telemetry_url, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'detected', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)`
      ).run(
        input.matchId,
        input.platform,
        input.mapName ?? null,
        input.gameMode ?? null,
        input.playedAt.toISOString(),
        input.matchStartAt?.toISOString() ?? null,
        input.matchEndAt?.toISOString() ?? null,
        input.telemetryUrl ?? null
      );
    } catch (error) {
      // Handle unique constraint violation
      if (error instanceof Error && error.message.includes('UNIQUE constraint failed')) {
        const existing = this.getById(input.matchId);
        if (existing) return existing;
      }
      throw error;
    }

    const match = this.getById(input.matchId);
    if (!match) {
      throw new Error('Failed to create match');
    }
    return match;
  }

  /**
   * Update match status
   */
  updateStatus(matchId: string, status: MatchStatus): void {
    this.db.prepare(
      'UPDATE matches SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE match_id = ?'
    ).run(status, matchId);
  }

  /**
   * Update match telemetry URL
   */
  updateTelemetryUrl(matchId: string, telemetryUrl: string): void {
    this.db.prepare(
      'UPDATE matches SET telemetry_url = ?, updated_at = CURRENT_TIMESTAMP WHERE match_id = ?'
    ).run(telemetryUrl, matchId);
  }

  /**
   * Get matches by status
   */
  getByStatus(status: MatchStatus): Match[] {
    const rows = this.db.prepare(
      'SELECT * FROM matches WHERE status = ? ORDER BY played_at DESC'
    ).all(status) as MatchRow[];
    
    return rows.map(mapRowToMatch);
  }

  /**
   * Get matches pending sync
   */
  getPendingSync(): Match[] {
    const rows = this.db.prepare(
      "SELECT * FROM matches WHERE status IN ('detected', 'failed') ORDER BY played_at DESC"
    ).all() as MatchRow[];
    
    return rows.map(mapRowToMatch);
  }

  /**
   * Recover in-progress sync records left by an interrupted app session.
   * Recent syncs are retried, stale syncs are marked failed.
   */
  resetSyncingMatches(timeoutMs: number, now: Date = new Date()): { retried: number; failed: number } {
    const syncingMatches = this.getByStatus('syncing');
    const cutoffTime = now.getTime() - timeoutMs;
    let retried = 0;
    let failed = 0;

    this.db.transaction(() => {
      for (const match of syncingMatches) {
        if (match.updatedAt.getTime() >= cutoffTime) {
          this.updateStatus(match.matchId, 'detected');
          retried += 1;
        } else {
          this.updateStatus(match.matchId, 'failed');
          failed += 1;
        }
      }
    })();

    return { retried, failed };
  }

  // Match Players operations

  /**
   * Get all players for a match
   */
  getPlayers(matchId: string): MatchPlayer[] {
    const rows = this.db.prepare(
      'SELECT * FROM match_players WHERE match_id = ? ORDER BY damage DESC'
    ).all(matchId) as MatchPlayerRow[];
    
    return rows.map(mapRowToMatchPlayer);
  }

  /**
   * Get a specific match player by ID
   */
  getPlayerById(playerId: number): MatchPlayer | null {
    const row = this.db.prepare(
      'SELECT * FROM match_players WHERE id = ?'
    ).get(playerId) as MatchPlayerRow | undefined;
    
    return row ? mapRowToMatchPlayer(row) : null;
  }

  /**
   * Create a match player
   */
  createPlayer(input: CreateMatchPlayerInput): MatchPlayer {
    const result = this.db.prepare(
      `INSERT INTO match_players 
       (match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot, team_id, 
        damage, kills, revives, placement, is_self, is_redbag_enabled_snapshot, redbag_cents, created_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)`
    ).run(
      input.matchId,
      input.teammateId ?? null,
      input.pubgAccountId ?? null,
      input.pubgPlayerName,
      input.displayNicknameSnapshot ?? null,
      input.teamId ?? null,
      input.damage ?? 0,
      input.kills ?? 0,
      input.revives ?? 0,
      input.placement ?? null,
      input.isSelf ? 1 : 0,
      input.isRedbagEnabledSnapshot ?? true ? 1 : 0,
      input.redbagCents ?? 0
    );

    const player = this.getPlayerById(result.lastInsertRowid as number);
    if (!player) {
      throw new Error('Failed to create match player');
    }
    return player;
  }

  /**
   * Update match player stats
   */
  updatePlayerStats(playerId: number, stats: Partial<Pick<MatchPlayer, 'damage' | 'kills' | 'revives' | 'placement' | 'redbagCents'>>): void {
    const sets: string[] = [];
    const params: (string | number | null)[] = [];

    if (stats.damage !== undefined) {
      sets.push('damage = ?');
      params.push(stats.damage);
    }
    if (stats.kills !== undefined) {
      sets.push('kills = ?');
      params.push(stats.kills);
    }
    if (stats.revives !== undefined) {
      sets.push('revives = ?');
      params.push(stats.revives);
    }
    if (stats.placement !== undefined) {
      sets.push('placement = ?');
      params.push(stats.placement);
    }
    if (stats.redbagCents !== undefined) {
      sets.push('redbag_cents = ?');
      params.push(stats.redbagCents);
    }

    if (sets.length === 0) return;

    params.push(playerId);
    this.db.prepare(
      `UPDATE match_players SET ${sets.join(', ')} WHERE id = ?`
    ).run(...params);
  }

  /**
   * Link a match player to a teammate
   */
  linkPlayerToTeammate(playerId: number, teammateId: number): void {
    this.db.prepare(
      'UPDATE match_players SET teammate_id = ? WHERE id = ?'
    ).run(teammateId, playerId);
  }

  /**
   * Check if match exists
   */
  exists(matchId: string): boolean {
    const result = this.db.prepare(
      'SELECT 1 FROM matches WHERE match_id = ?'
    ).get(matchId) as { 1: number } | undefined;
    return !!result;
  }

  /**
   * Delete a match and all related data
   */
  delete(matchId: string): void {
    this.db.prepare('DELETE FROM matches WHERE match_id = ?').run(matchId);
  }
}
