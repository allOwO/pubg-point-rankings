/**
 * Redbag Records Repository
 * Handles CRUD operations for redbag_records table
 */

import type { Database } from 'better-sqlite3';
import type { RedbagRecord, RoundingMode } from '@pubg-point-rankings/shared';

interface RedbagRecordRow {
  id: number;
  match_id: string;
  match_player_id: number;
  teammate_id: number | null;
  rule_id: number;
  rule_name_snapshot: string;
  damage_cent_per_point_snapshot: number;
  kill_cent_snapshot: number;
  revive_cent_snapshot: number;
  rounding_mode_snapshot: string;
  amount_cents: number;
  note: string | null;
  created_at: string;
}

function mapRowToRecord(row: RedbagRecordRow): RedbagRecord {
  return {
    id: row.id,
    matchId: row.match_id,
    matchPlayerId: row.match_player_id,
    teammateId: row.teammate_id,
    ruleId: row.rule_id,
    ruleNameSnapshot: row.rule_name_snapshot,
    damageCentPerPointSnapshot: row.damage_cent_per_point_snapshot,
    killCentSnapshot: row.kill_cent_snapshot,
    reviveCentSnapshot: row.revive_cent_snapshot,
    roundingModeSnapshot: row.rounding_mode_snapshot as RoundingMode,
    amountCents: row.amount_cents,
    note: row.note,
    createdAt: new Date(row.created_at),
  };
}

export interface CreateRedbagRecordInput {
  matchId: string;
  matchPlayerId: number;
  teammateId?: number | null;
  ruleId: number;
  ruleNameSnapshot: string;
  damageCentPerPointSnapshot: number;
  killCentSnapshot: number;
  reviveCentSnapshot: number;
  roundingModeSnapshot: RoundingMode;
  amountCents: number;
  note?: string | null;
}

export class RedbagRecordsRepository {
  constructor(private db: Database) {}

  /**
   * Get all redbag records with pagination
   */
  getAll(limit: number = 100, offset: number = 0): RedbagRecord[] {
    const rows = this.db.prepare(
      `SELECT r.* FROM redbag_records r
       ORDER BY r.created_at DESC
       LIMIT ? OFFSET ?`
    ).all(limit, offset) as RedbagRecordRow[];
    
    return rows.map(mapRowToRecord);
  }

  /**
   * Get records by match ID
   */
  getByMatch(matchId: string): RedbagRecord[] {
    const rows = this.db.prepare(
      `SELECT * FROM redbag_records WHERE match_id = ? ORDER BY amount_cents DESC`
    ).all(matchId) as RedbagRecordRow[];
    
    return rows.map(mapRowToRecord);
  }

  /**
   * Get records by teammate ID
   */
  getByTeammate(teammateId: number): RedbagRecord[] {
    const rows = this.db.prepare(
      `SELECT * FROM redbag_records WHERE teammate_id = ? ORDER BY created_at DESC`
    ).all(teammateId) as RedbagRecordRow[];
    
    return rows.map(mapRowToRecord);
  }

  /**
   * Get a single record by ID
   */
  getById(id: number): RedbagRecord | null {
    const row = this.db.prepare(
      'SELECT * FROM redbag_records WHERE id = ?'
    ).get(id) as RedbagRecordRow | undefined;
    
    return row ? mapRowToRecord(row) : null;
  }

  /**
   * Create a new redbag record
   */
  create(input: CreateRedbagRecordInput): RedbagRecord {
    const result = this.db.prepare(
      `INSERT INTO redbag_records 
       (match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot,
        damage_cent_per_point_snapshot, kill_cent_snapshot, revive_cent_snapshot,
        rounding_mode_snapshot, amount_cents, note, created_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)`
    ).run(
      input.matchId,
      input.matchPlayerId,
      input.teammateId ?? null,
      input.ruleId,
      input.ruleNameSnapshot,
      input.damageCentPerPointSnapshot,
      input.killCentSnapshot,
      input.reviveCentSnapshot,
      input.roundingModeSnapshot,
      input.amountCents,
      input.note ?? null
    );

    const record = this.getById(result.lastInsertRowid as number);
    if (!record) {
      throw new Error('Failed to create redbag record');
    }
    return record;
  }

  /**
   * Create multiple records in a transaction
   */
  createMany(inputs: CreateRedbagRecordInput[]): RedbagRecord[] {
    const stmt = this.db.prepare(
      `INSERT INTO redbag_records 
       (match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot,
        damage_cent_per_point_snapshot, kill_cent_snapshot, revive_cent_snapshot,
        rounding_mode_snapshot, amount_cents, note, created_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)`
    );

    const transaction = this.db.transaction((items: CreateRedbagRecordInput[]) => {
      const records: RedbagRecord[] = [];
      for (const input of items) {
        const result = stmt.run(
          input.matchId,
          input.matchPlayerId,
          input.teammateId ?? null,
          input.ruleId,
          input.ruleNameSnapshot,
          input.damageCentPerPointSnapshot,
          input.killCentSnapshot,
          input.reviveCentSnapshot,
          input.roundingModeSnapshot,
          input.amountCents,
          input.note ?? null
        );
        const record = this.getById(result.lastInsertRowid as number);
        if (record) records.push(record);
      }
      return records;
    });

    return transaction(inputs);
  }

  /**
   * Check if records exist for a match
   */
  existsForMatch(matchId: string): boolean {
    const result = this.db.prepare(
      'SELECT 1 FROM redbag_records WHERE match_id = ? LIMIT 1'
    ).get(matchId) as { 1: number } | undefined;
    return !!result;
  }

  /**
   * Get total redbag amount for a teammate
   */
  getTotalForTeammate(teammateId: number): number {
    const result = this.db.prepare(
      'SELECT COALESCE(SUM(amount_cents), 0) as total FROM redbag_records WHERE teammate_id = ?'
    ).get(teammateId) as { total: number };
    return result.total;
  }

  /**
   * Get total redbag amount across all records
   */
  getGrandTotal(): number {
    const result = this.db.prepare(
      'SELECT COALESCE(SUM(amount_cents), 0) as total FROM redbag_records'
    ).get() as { total: number };
    return result.total;
  }

  /**
   * Update record note
   */
  updateNote(id: number, note: string | null): void {
    this.db.prepare(
      'UPDATE redbag_records SET note = ? WHERE id = ?'
    ).run(note, id);
  }

  /**
   * Delete records for a match
   */
  deleteByMatch(matchId: string): void {
    this.db.prepare('DELETE FROM redbag_records WHERE match_id = ?').run(matchId);
  }
}
