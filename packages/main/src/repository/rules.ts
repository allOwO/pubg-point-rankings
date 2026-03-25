/**
 * Point Rules Repository
 * Handles CRUD operations for point_rules table
 */

import type { Database } from 'better-sqlite3';
import type { PointRule, RoundingMode, CreatePointRuleInput, UpdatePointRuleInput } from '@pubg-point-rankings/shared';

interface PointRuleRow {
  id: number;
  name: string;
  damage_cent_per_point: number;
  kill_cent: number;
  revive_cent: number;
  is_active: number;
  rounding_mode: string;
  created_at: string;
  updated_at: string;
}

function mapRowToRule(row: PointRuleRow): PointRule {
  return {
    id: row.id,
    name: row.name,
    damagePointsPerDamage: row.damage_cent_per_point,
    killPoints: row.kill_cent,
    revivePoints: row.revive_cent,
    isActive: row.is_active === 1,
    roundingMode: row.rounding_mode as RoundingMode,
    createdAt: new Date(row.created_at),
    updatedAt: new Date(row.updated_at),
  };
}

export class PointRulesRepository {
  constructor(private db: Database) {}

  /**
   * Get all rules
   */
  getAll(): PointRule[] {
    const rows = this.db.prepare(
      'SELECT * FROM point_rules ORDER BY created_at'
    ).all() as PointRuleRow[];
    
    return rows.map(mapRowToRule);
  }

  /**
   * Get active rule
   */
  getActive(): PointRule | null {
    const row = this.db.prepare(
      'SELECT * FROM point_rules WHERE is_active = 1 LIMIT 1'
    ).get() as PointRuleRow | undefined;
    
    return row ? mapRowToRule(row) : null;
  }

  /**
   * Get rule by ID
   */
  getById(id: number): PointRule | null {
    const row = this.db.prepare(
      'SELECT * FROM point_rules WHERE id = ?'
    ).get(id) as PointRuleRow | undefined;
    
    return row ? mapRowToRule(row) : null;
  }

  /**
   * Create a new rule
   */
  create(input: CreatePointRuleInput): PointRule {
    const result = this.db.prepare(
      `INSERT INTO point_rules 
       (name, damage_cent_per_point, kill_cent, revive_cent, rounding_mode, is_active, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)`
    ).run(
      input.name,
      input.damagePointsPerDamage,
      input.killPoints,
      input.revivePoints,
      input.roundingMode
    );

    const rule = this.getById(result.lastInsertRowid as number);
    if (!rule) {
      throw new Error('Failed to create rule');
    }
    return rule;
  }

  /**
   * Update a rule
   */
  update(input: UpdatePointRuleInput): PointRule {
    const sets: string[] = [];
    const params: (string | number)[] = [];

    if (input.name !== undefined) {
      sets.push('name = ?');
      params.push(input.name);
    }
    if (input.damagePointsPerDamage !== undefined) {
      sets.push('damage_cent_per_point = ?');
      params.push(input.damagePointsPerDamage);
    }
    if (input.killPoints !== undefined) {
      sets.push('kill_cent = ?');
      params.push(input.killPoints);
    }
    if (input.revivePoints !== undefined) {
      sets.push('revive_cent = ?');
      params.push(input.revivePoints);
    }
    if (input.roundingMode !== undefined) {
      sets.push('rounding_mode = ?');
      params.push(input.roundingMode);
    }

    if (sets.length === 0) {
      const existing = this.getById(input.id);
      if (!existing) throw new Error('Rule not found');
      return existing;
    }

    sets.push('updated_at = CURRENT_TIMESTAMP');
    params.push(input.id);

    this.db.prepare(
      `UPDATE point_rules SET ${sets.join(', ')} WHERE id = ?`
    ).run(...params);

    const rule = this.getById(input.id);
    if (!rule) {
      throw new Error('Rule not found after update');
    }
    return rule;
  }

  /**
   * Delete a rule (only if not active)
   */
  delete(id: number): void {
    // Check if rule is active
    const rule = this.getById(id);
    if (rule?.isActive) {
      throw new Error('Cannot delete active rule');
    }

    // Check if rule has been used in records
    const count = this.db.prepare(
      'SELECT COUNT(*) as count FROM point_records WHERE rule_id = ?'
    ).get(id) as { count: number };
    
    if (count.count > 0) {
      throw new Error('Cannot delete rule that has been used');
    }

    this.db.prepare('DELETE FROM point_rules WHERE id = ?').run(id);
  }

  /**
   * Activate a rule (deactivates all others)
   */
  activate(id: number): PointRule {
    const transaction = this.db.transaction(() => {
      // Deactivate all rules
      this.db.prepare('UPDATE point_rules SET is_active = 0, updated_at = CURRENT_TIMESTAMP').run();
      
      // Activate the specified rule
      this.db.prepare(
        'UPDATE point_rules SET is_active = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?'
      ).run(id);
    });

    transaction();

    const rule = this.getById(id);
    if (!rule) {
      throw new Error('Rule not found after activation');
    }
    return rule;
  }
}
