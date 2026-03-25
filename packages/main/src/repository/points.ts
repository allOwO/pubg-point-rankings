/**
 * Point Records Repository
 * Handles CRUD operations for point_records table
 */

import type { Database } from 'better-sqlite3';
import type { PointRecord, RoundingMode } from '@pubg-point-rankings/shared';

interface PointRecordRow {
  id: number;
  match_id: string;
  match_player_id: number;
  teammate_id: number | null;
  rule_id: number;
  rule_name_snapshot: string;
  damage_points_per_damage_snapshot: number;
  kill_points_snapshot: number;
  revive_points_snapshot: number;
  rounding_mode_snapshot: string;
  points: number;
  note: string | null;
  created_at: string;
}

function mapRowToRecord(row: PointRecordRow): PointRecord {
  return {
    id: row.id,
    accountId: 1,
    matchId: row.match_id,
    matchPlayerId: row.match_player_id,
    teammateId: row.teammate_id,
    ruleId: row.rule_id,
    ruleNameSnapshot: row.rule_name_snapshot,
    damagePointsPerDamageSnapshot: row.damage_points_per_damage_snapshot,
    killPointsSnapshot: row.kill_points_snapshot,
    revivePointsSnapshot: row.revive_points_snapshot,
    roundingModeSnapshot: row.rounding_mode_snapshot as RoundingMode,
    points: row.points,
    note: row.note,
    createdAt: new Date(row.created_at),
  };
}

export interface CreatePointRecordInput {
  matchId: string;
  matchPlayerId: number;
  teammateId?: number | null;
  ruleId: number;
  ruleNameSnapshot: string;
  damagePointsPerDamageSnapshot: number;
  killPointsSnapshot: number;
  revivePointsSnapshot: number;
  roundingModeSnapshot: RoundingMode;
  points: number;
  note?: string | null;
}

export class PointRecordsRepository {
  constructor(private db: Database) {}

  getAll(limit: number = 100, offset: number = 0): PointRecord[] {
    const rows = this.db.prepare(
      `SELECT r.* FROM point_records r
       ORDER BY r.created_at DESC
       LIMIT ? OFFSET ?`
    ).all(limit, offset) as PointRecordRow[];

    return rows.map(mapRowToRecord);
  }

  getByMatch(matchId: string): PointRecord[] {
    const rows = this.db.prepare(
      `SELECT * FROM point_records WHERE match_id = ? ORDER BY points DESC`
    ).all(matchId) as PointRecordRow[];

    return rows.map(mapRowToRecord);
  }

  getByTeammate(teammateId: number): PointRecord[] {
    const rows = this.db.prepare(
      `SELECT * FROM point_records WHERE teammate_id = ? ORDER BY created_at DESC`
    ).all(teammateId) as PointRecordRow[];

    return rows.map(mapRowToRecord);
  }

  getById(id: number): PointRecord | null {
    const row = this.db.prepare(
      'SELECT * FROM point_records WHERE id = ?'
    ).get(id) as PointRecordRow | undefined;

    return row ? mapRowToRecord(row) : null;
  }

  create(input: CreatePointRecordInput): PointRecord {
    const result = this.db.prepare(
      `INSERT INTO point_records 
       (match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot,
        damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot,
        rounding_mode_snapshot, points, note, created_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)`
    ).run(
      input.matchId,
      input.matchPlayerId,
      input.teammateId ?? null,
      input.ruleId,
      input.ruleNameSnapshot,
      input.damagePointsPerDamageSnapshot,
      input.killPointsSnapshot,
      input.revivePointsSnapshot,
      input.roundingModeSnapshot,
      input.points,
      input.note ?? null
    );

    const record = this.getById(result.lastInsertRowid as number);
    if (!record) {
      throw new Error('Failed to create point record');
    }
    return record;
  }

  createMany(inputs: CreatePointRecordInput[]): PointRecord[] {
    const stmt = this.db.prepare(
      `INSERT INTO point_records 
       (match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot,
        damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot,
        rounding_mode_snapshot, points, note, created_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)`
    );

    const transaction = this.db.transaction((items: CreatePointRecordInput[]) => {
      const records: PointRecord[] = [];
      for (const input of items) {
        const result = stmt.run(
          input.matchId,
          input.matchPlayerId,
          input.teammateId ?? null,
          input.ruleId,
          input.ruleNameSnapshot,
          input.damagePointsPerDamageSnapshot,
          input.killPointsSnapshot,
          input.revivePointsSnapshot,
          input.roundingModeSnapshot,
          input.points,
          input.note ?? null
        );
        const record = this.getById(result.lastInsertRowid as number);
        if (record) records.push(record);
      }
      return records;
    });

    return transaction(inputs);
  }

  existsForMatch(matchId: string): boolean {
    const result = this.db.prepare(
      'SELECT 1 FROM point_records WHERE match_id = ? LIMIT 1'
    ).get(matchId) as { 1: number } | undefined;
    return !!result;
  }

  getTotalForTeammate(teammateId: number): number {
    const result = this.db.prepare(
      'SELECT COALESCE(SUM(points), 0) as total FROM point_records WHERE teammate_id = ?'
    ).get(teammateId) as { total: number };
    return result.total;
  }

  getGrandTotal(): number {
    const result = this.db.prepare(
      'SELECT COALESCE(SUM(points), 0) as total FROM point_records'
    ).get() as { total: number };
    return result.total;
  }

  updateNote(id: number, note: string | null): void {
    this.db.prepare(
      'UPDATE point_records SET note = ? WHERE id = ?'
    ).run(note, id);
  }

  deleteByMatch(matchId: string): void {
    this.db.prepare('DELETE FROM point_records WHERE match_id = ?').run(matchId);
  }
}
