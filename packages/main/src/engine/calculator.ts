/**
 * Points Calculation Engine
 * Calculates point totals based on player stats and rules
 */

import type { 
  PointRule, 
  PlayerStats, 
  CalculatedPoints, 
  RoundingMode 
} from '@pubg-point-rankings/shared';

export interface CalculationInput {
  rule: PointRule;
  players: PlayerStats[];
  enabledPlayerIds: Set<string>;
}

/**
 * Apply rounding based on the specified mode
 */
export function applyRounding(value: number, mode: RoundingMode): number {
  switch (mode) {
    case 'floor':
      return Math.floor(value);
    case 'ceil':
      return Math.ceil(value);
    case 'round':
    default:
      return Math.round(value);
  }
}

/**
 * Calculate points for a single player
 */
export function calculatePlayerPoints(
  player: PlayerStats,
  rule: PointRule,
  isEnabled: boolean
): CalculatedPoints {
  const damagePoints = player.damage * rule.damagePointsPerDamage;
  const killPoints = player.kills * rule.killPoints;
  const revivePoints = player.revives * rule.revivePoints;
  
  const totalPoints = isEnabled 
    ? applyRounding(damagePoints + killPoints + revivePoints, rule.roundingMode)
    : 0;

  return {
    pubgAccountId: player.pubgAccountId,
    pubgPlayerName: player.pubgPlayerName,
    damage: player.damage,
    kills: player.kills,
    assists: player.assists,
    revives: player.revives,
    damagePoints: applyRounding(damagePoints, rule.roundingMode),
    killPoints,
    revivePoints,
    totalPoints,
    isPointsEnabled: isEnabled,
  };
}

/**
 * Calculate points for all players in a match
 */
export function calculatePoints(input: CalculationInput): CalculatedPoints[] {
  return input.players.map(player => {
    const isEnabled = input.enabledPlayerIds.has(player.pubgAccountId);
    return calculatePlayerPoints(player, input.rule, isEnabled);
  });
}

/**
 * Calculate total points for the match
 */
export function calculateTotalPoints(
  points: CalculatedPoints[]
): number {
  return points.reduce((sum, record) => sum + record.totalPoints, 0);
}

/**
 * Format integer points for display
 */
export function formatPoints(points: number): string {
  return `${Math.round(points).toLocaleString()} pts`;
}

/**
 * Convert point input string to integer points
 */
export function parsePoints(pointsStr: string): number {
  const cleaned = pointsStr.replace(/[,\s]|pts/gi, '');
  const points = parseFloat(cleaned);
  if (Number.isNaN(points)) return 0;
  return Math.round(points);
}
