export interface MatchListPlacementCandidate {
  matchPlayerId: number;
  isSelf: boolean;
  isPointsEnabled: boolean;
  placement: number | null;
  points: number;
}

export interface MatchBattleDeltaCandidate {
  matchPlayerId: number;
  displayName: string;
  isPointsEnabled: boolean;
  points: number;
}

export interface MatchBattleDeltaResult {
  matchPlayerId: number;
  displayName: string;
  delta: number;
}

export interface MatchListResult {
  placement: number | null;
  delta: number;
}

export function getMatchListPlacement(players: MatchListPlacementCandidate[]): number | null {
  const selfPlacement = players.find((player) => player.isSelf && player.placement !== null)?.placement;
  if (selfPlacement !== undefined) {
    return selfPlacement;
  }

  return players.find((player) => player.placement !== null)?.placement ?? null;
}

export function getMatchBattleDeltas(players: MatchBattleDeltaCandidate[]): MatchBattleDeltaResult[] {
  const zeroDeltas = players.map((player) => ({
    matchPlayerId: player.matchPlayerId,
    displayName: player.displayName,
    delta: 0,
  }));

  const participants = players.filter((player) => player.isPointsEnabled);
  if (participants.length < 2) {
    return zeroDeltas;
  }

  let highest = participants[0];
  let lowest = participants[0];

  for (const participant of participants.slice(1)) {
    if (
      participant.points > highest.points
      || (participant.points === highest.points && participant.matchPlayerId < highest.matchPlayerId)
    ) {
      highest = participant;
    }

    if (
      participant.points < lowest.points
      || (participant.points === lowest.points && participant.matchPlayerId < lowest.matchPlayerId)
    ) {
      lowest = participant;
    }
  }

  const gap = highest.points - lowest.points;
  if (gap === 0) {
    return zeroDeltas;
  }

  return players.map((player) => {
    let delta = 0;
    if (player.matchPlayerId === highest.matchPlayerId) {
      delta = gap;
    } else if (player.matchPlayerId === lowest.matchPlayerId) {
      delta = -gap;
    }

    return {
      matchPlayerId: player.matchPlayerId,
      displayName: player.displayName,
      delta,
    };
  });
}

/**
 * Calculate the battle delta for the current player in a match.
 * Uses battle-delta semantics:
 * - If self is the highest scorer among enabled players: return positive gap (highest - second)
 * - If self is the lowest scorer among enabled players: return negative gap (second lowest - lowest)
 * - Otherwise: return 0
 * - If fewer than 2 enabled players or all have same score: return 0
 */
export function getMatchListBattleDelta(players: MatchListPlacementCandidate[]): number {
  const self = players.find((player) => player.isSelf);
  if (!self) {
    return 0;
  }

  const deltas = getMatchBattleDeltas(players.map((player) => ({
    matchPlayerId: player.matchPlayerId,
    displayName: '',
    isPointsEnabled: player.isPointsEnabled,
    points: player.points,
  })));

  return deltas.find((delta) => delta.matchPlayerId === self.matchPlayerId)?.delta ?? 0;
}
