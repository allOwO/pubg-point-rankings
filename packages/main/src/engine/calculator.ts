/**
 * Points Calculation Engine
 * Calculates point totals based on player stats and rules
 */

import type { 
  RedbagRule, 
  PlayerStats, 
  CalculatedRedbag, 
  RoundingMode 
} from '@pubg-point-rankings/shared';

export interface CalculationInput {
  rule: RedbagRule;
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
 * Calculate red bag for a single player
 */
export function calculatePlayerRedbag(
  player: PlayerStats,
  rule: RedbagRule,
  isEnabled: boolean
): CalculatedRedbag {
  const damageCents = player.damage * rule.damageCentPerPoint;
  const killsCents = player.kills * rule.killCent;
  const revivesCents = player.revives * rule.reviveCent;
  
  const totalCents = isEnabled 
    ? applyRounding(damageCents + killsCents + revivesCents, rule.roundingMode)
    : 0;

  return {
    pubgAccountId: player.pubgAccountId,
    pubgPlayerName: player.pubgPlayerName,
    damage: player.damage,
    kills: player.kills,
    revives: player.revives,
    damageCents: applyRounding(damageCents, rule.roundingMode),
    killsCents,
    revivesCents,
    totalCents,
    isRedbagEnabled: isEnabled,
  };
}

/**
 * Calculate red bags for all players in a match
 */
export function calculateRedbags(input: CalculationInput): CalculatedRedbag[] {
  return input.players.map(player => {
    const isEnabled = input.enabledPlayerIds.has(player.pubgAccountId);
    return calculatePlayerRedbag(player, input.rule, isEnabled);
  });
}

/**
 * Calculate total red bag amount for the match
 */
export function calculateTotalRedbag(
  redbags: CalculatedRedbag[]
): number {
  return redbags.reduce((sum, r) => sum + r.totalCents, 0);
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
