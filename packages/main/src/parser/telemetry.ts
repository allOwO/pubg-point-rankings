/**
 * PUBG Telemetry Parser
 * Parses PUBG telemetry data to extract player statistics
 */

import type { PlayerStats } from '@pubg-point-rankings/shared';

// Telemetry event types based on PUBG API documentation
export interface TelemetryEvent {
  _T: string;
  _D: string; // ISO timestamp
}

export interface LogPlayerTakeDamage extends TelemetryEvent {
  _T: 'LogPlayerTakeDamage';
  attacker?: {
    accountId: string;
    name: string;
  };
  victim?: {
    accountId: string;
    name: string;
  };
  damage: number;
  damageTypeCategory: string;
}

export interface LogPlayerKillV2 extends TelemetryEvent {
  _T: 'LogPlayerKillV2';
  killer?: {
    accountId: string;
    name: string;
  };
  victim?: {
    accountId: string;
    name: string;
  };
}

export interface LogPlayerRevive extends TelemetryEvent {
  _T: 'LogPlayerRevive';
  reviver?: {
    accountId: string;
    name: string;
  };
  victim?: {
    accountId: string;
    name: string;
  };
}

export interface LogMatchDefinition extends TelemetryEvent {
  _T: 'LogMatchDefinition';
  MatchId: string;
}

export interface LogMatchStart extends TelemetryEvent {
  _T: 'LogMatchStart';
  characters: Array<{
    accountId: string;
    name: string;
    teamId: number;
  }>;
}

export interface LogMatchEnd extends TelemetryEvent {
  _T: 'LogMatchEnd';
  characters: Array<{
    accountId: string;
    name: string;
    teamId: number;
    ranking: number;
  }>;
}

export type TelemetryData = 
  | LogPlayerTakeDamage 
  | LogPlayerKillV2 
  | LogPlayerRevive 
  | LogMatchDefinition 
  | LogMatchStart 
  | LogMatchEnd 
  | TelemetryEvent;

/**
 * Intermediate stats accumulator
 */
interface PlayerStatsAccumulator {
  accountId: string;
  name: string;
  teamId: number | null;
  damage: number;
  kills: number;
  revives: number;
  placement: number | null;
}

/**
 * Parse telemetry JSON data
 */
export function parseTelemetry(jsonData: string): TelemetryData[] {
  try {
    return JSON.parse(jsonData) as TelemetryData[];
  } catch (error) {
    throw new Error(`Failed to parse telemetry data: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Aggregate player statistics from telemetry events
 */
export function aggregatePlayerStats(events: TelemetryData[]): PlayerStats[] {
  const statsMap = new Map<string, PlayerStatsAccumulator>();

  // First pass: collect player info from match start
  for (const event of events) {
    if (event._T === 'LogMatchStart') {
      const startEvent = event as LogMatchStart;
      for (const char of startEvent.characters) {
        if (!statsMap.has(char.accountId)) {
          statsMap.set(char.accountId, {
            accountId: char.accountId,
            name: char.name,
            teamId: char.teamId,
            damage: 0,
            kills: 0,
            revives: 0,
            placement: null,
          });
        }
      }
    }
  }

  // Second pass: aggregate stats
  for (const event of events) {
    switch (event._T) {
      case 'LogPlayerTakeDamage': {
        const damageEvent = event as LogPlayerTakeDamage;
        if (damageEvent.attacker?.accountId) {
          const attackerId = damageEvent.attacker.accountId;
          let stats = statsMap.get(attackerId);
          if (!stats) {
            stats = {
              accountId: attackerId,
              name: damageEvent.attacker.name,
              teamId: null,
              damage: 0,
              kills: 0,
              revives: 0,
              placement: null,
            };
            statsMap.set(attackerId, stats);
          }
          // Only count damage to other players (not self-damage, not environmental)
          if (damageEvent.victim?.accountId !== attackerId) {
            stats.damage += damageEvent.damage;
          }
        }
        break;
      }

      case 'LogPlayerKillV2': {
        const killEvent = event as LogPlayerKillV2;
        if (killEvent.killer?.accountId) {
          const killerId = killEvent.killer.accountId;
          let stats = statsMap.get(killerId);
          if (!stats) {
            stats = {
              accountId: killerId,
              name: killEvent.killer.name,
              teamId: null,
              damage: 0,
              kills: 0,
              revives: 0,
              placement: null,
            };
            statsMap.set(killerId, stats);
          }
          stats.kills += 1;
        }
        break;
      }

      case 'LogPlayerRevive': {
        const reviveEvent = event as LogPlayerRevive;
        if (reviveEvent.reviver?.accountId) {
          const reviverId = reviveEvent.reviver.accountId;
          let stats = statsMap.get(reviverId);
          if (!stats) {
            stats = {
              accountId: reviverId,
              name: reviveEvent.reviver.name,
              teamId: null,
              damage: 0,
              kills: 0,
              revives: 0,
              placement: null,
            };
            statsMap.set(reviverId, stats);
          }
          stats.revives += 1;
        }
        break;
      }

      case 'LogMatchEnd': {
        const endEvent = event as LogMatchEnd;
        for (const char of endEvent.characters) {
          const stats = statsMap.get(char.accountId);
          if (stats) {
            stats.placement = char.ranking;
          }
        }
        break;
      }
    }
  }

  // Convert map to array
  return Array.from(statsMap.values()).map(acc => ({
    pubgAccountId: acc.accountId,
    pubgPlayerName: acc.name,
    damage: Math.round(acc.damage * 10) / 10, // Round to 1 decimal place
    kills: acc.kills,
    revives: acc.revives,
    teamId: acc.teamId,
    placement: acc.placement,
  }));
}

/**
 * Get teammates for a specific player (same team)
 */
export function getTeammates(
  stats: PlayerStats[],
  playerAccountId: string
): PlayerStats[] {
  const player = stats.find(s => s.pubgAccountId === playerAccountId);
  if (!player?.teamId) return [];
  
  return stats.filter(s => s.teamId === player.teamId);
}

/**
 * Filter out AI/bot players (optional)
 * PUBG account IDs for real players typically follow a pattern
 */
export function filterRealPlayers(stats: PlayerStats[]): PlayerStats[] {
  // Real players have account IDs that don't start with 'ai.' or contain 'bot'
  return stats.filter(s => {
    const id = s.pubgAccountId.toLowerCase();
    return !id.startsWith('ai.') && !id.includes('bot');
  });
}

/**
 * Find player by name (case-insensitive)
 */
export function findPlayerByName(
  stats: PlayerStats[],
  name: string
): PlayerStats | undefined {
  const lowerName = name.toLowerCase();
  return stats.find(s => s.pubgPlayerName.toLowerCase() === lowerName);
}

/**
 * Find player by account ID
 */
export function findPlayerByAccountId(
  stats: PlayerStats[],
  accountId: string
): PlayerStats | undefined {
  return stats.find(s => s.pubgAccountId === accountId);
}
