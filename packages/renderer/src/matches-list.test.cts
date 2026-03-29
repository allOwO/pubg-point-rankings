import assert from 'node:assert/strict';
import test from 'node:test';

test('prefers self placement for match list summary', async () => {
  const { getMatchListPlacement } = await import('./matches-list');

  assert.equal(
    getMatchListPlacement([
      { matchPlayerId: 1, isSelf: false, isPointsEnabled: true, placement: 8, points: 100 },
      { matchPlayerId: 2, isSelf: true, isPointsEnabled: true, placement: 3, points: 200 },
    ]),
    3,
  );
});

test('falls back to first known placement when self is unavailable', async () => {
  const { getMatchListPlacement } = await import('./matches-list');

  assert.equal(
    getMatchListPlacement([
      { matchPlayerId: 1, isSelf: false, isPointsEnabled: true, placement: null, points: 100 },
      { matchPlayerId: 2, isSelf: false, isPointsEnabled: true, placement: 5, points: 200 },
    ]),
    5,
  );
});

test('returns null when no placement is available', async () => {
  const { getMatchListPlacement } = await import('./matches-list');

  assert.equal(
    getMatchListPlacement([
      { matchPlayerId: 1, isSelf: false, isPointsEnabled: true, placement: null, points: 100 },
      { matchPlayerId: 2, isSelf: true, isPointsEnabled: true, placement: null, points: 200 },
    ]),
    null,
  );
});

// Tests for getMatchListBattleDelta
test('returns positive delta when self is highest scorer', async () => {
  const { getMatchListBattleDelta } = await import('./matches-list');

  const delta = getMatchListBattleDelta([
    { matchPlayerId: 1, isSelf: true, isPointsEnabled: true, placement: 1, points: 150 },
    { matchPlayerId: 2, isSelf: false, isPointsEnabled: true, placement: 2, points: 100 },
    { matchPlayerId: 3, isSelf: false, isPointsEnabled: true, placement: 3, points: 80 },
  ]);

  assert.equal(delta, 70);
});

test('returns negative delta when self is lowest scorer', async () => {
  const { getMatchListBattleDelta } = await import('./matches-list');

  const delta = getMatchListBattleDelta([
    { matchPlayerId: 1, isSelf: false, isPointsEnabled: true, placement: 1, points: 150 },
    { matchPlayerId: 2, isSelf: false, isPointsEnabled: true, placement: 2, points: 100 },
    { matchPlayerId: 3, isSelf: true, isPointsEnabled: true, placement: 3, points: 50 },
  ]);

  assert.equal(delta, -100);
});

test('returns zero delta when self is neither highest nor lowest', async () => {
  const { getMatchListBattleDelta } = await import('./matches-list');

  const delta = getMatchListBattleDelta([
    { matchPlayerId: 1, isSelf: false, isPointsEnabled: true, placement: 1, points: 150 },
    { matchPlayerId: 2, isSelf: true, isPointsEnabled: true, placement: 2, points: 100 },
    { matchPlayerId: 3, isSelf: false, isPointsEnabled: true, placement: 3, points: 50 },
  ]);

  assert.equal(delta, 0);
});

test('returns zero delta when fewer than 2 enabled players', async () => {
  const { getMatchListBattleDelta } = await import('./matches-list');

  const delta = getMatchListBattleDelta([
    { matchPlayerId: 1, isSelf: true, isPointsEnabled: true, placement: 1, points: 100 },
  ]);

  assert.equal(delta, 0);
});

test('returns zero delta when all players have same score', async () => {
  const { getMatchListBattleDelta } = await import('./matches-list');

  const delta = getMatchListBattleDelta([
    { matchPlayerId: 1, isSelf: false, isPointsEnabled: true, placement: 1, points: 100 },
    { matchPlayerId: 2, isSelf: true, isPointsEnabled: true, placement: 2, points: 100 },
    { matchPlayerId: 3, isSelf: false, isPointsEnabled: true, placement: 3, points: 100 },
  ]);

  assert.equal(delta, 0);
});

test('ignores disabled players when computing self delta', async () => {
  const { getMatchListBattleDelta } = await import('./matches-list');

  const delta = getMatchListBattleDelta([
    { matchPlayerId: 1, isSelf: false, isPointsEnabled: true, placement: 1, points: 150 },
    { matchPlayerId: 2, isSelf: true, isPointsEnabled: true, placement: 2, points: 50 },
    { matchPlayerId: 3, isSelf: false, isPointsEnabled: false, placement: 3, points: 999 },
  ]);

  assert.equal(delta, -100);
});

test('returns zero when no self player found', async () => {
  const { getMatchListBattleDelta } = await import('./matches-list');

  const delta = getMatchListBattleDelta([
    { matchPlayerId: 1, isSelf: false, isPointsEnabled: true, placement: 1, points: 150 },
    { matchPlayerId: 2, isSelf: false, isPointsEnabled: true, placement: 2, points: 100 },
  ]);

  assert.equal(delta, 0);
});

// Tests for getMatchBattleDeltas

test('getMatchBattleDeltas returns correct deltas for highest and lowest scorers', async () => {
  const { getMatchBattleDeltas } = await import('./matches-list');

  const result = getMatchBattleDeltas([
    { matchPlayerId: 1, displayName: 'Player1', isPointsEnabled: true, points: 150 },
    { matchPlayerId: 2, displayName: 'Player2', isPointsEnabled: true, points: 100 },
    { matchPlayerId: 3, displayName: 'Player3', isPointsEnabled: true, points: 80 },
  ]);

  // Highest (150) gets +70, Lowest (80) gets -70, Middle gets 0
  assert.equal(result.find(r => r.matchPlayerId === 1)?.delta, 70);
  assert.equal(result.find(r => r.matchPlayerId === 2)?.delta, 0);
  assert.equal(result.find(r => r.matchPlayerId === 3)?.delta, -70);
});

test('getMatchBattleDeltas returns zero for all when all players have same score', async () => {
  const { getMatchBattleDeltas } = await import('./matches-list');

  const result = getMatchBattleDeltas([
    { matchPlayerId: 1, displayName: 'Player1', isPointsEnabled: true, points: 100 },
    { matchPlayerId: 2, displayName: 'Player2', isPointsEnabled: true, points: 100 },
    { matchPlayerId: 3, displayName: 'Player3', isPointsEnabled: true, points: 100 },
  ]);

  assert.equal(result.every(r => r.delta === 0), true);
});

test('getNonZeroMatchBattleDeltas omits zero-delta players', async () => {
  const { getNonZeroMatchBattleDeltas } = await import('./matches-list');

  const result = getNonZeroMatchBattleDeltas([
    { matchPlayerId: 1, displayName: 'Player1', isPointsEnabled: true, points: 150 },
    { matchPlayerId: 2, displayName: 'Player2', isPointsEnabled: true, points: 100 },
    { matchPlayerId: 3, displayName: 'Player3', isPointsEnabled: true, points: 80 },
  ]);

  assert.deepEqual(result, [
    { matchPlayerId: 1, displayName: 'Player1', delta: 70 },
    { matchPlayerId: 3, displayName: 'Player3', delta: -70 },
  ]);
});

test('getNonZeroMatchBattleDeltas returns empty list when no player has non-zero delta', async () => {
  const { getNonZeroMatchBattleDeltas } = await import('./matches-list');

  const result = getNonZeroMatchBattleDeltas([
    { matchPlayerId: 1, displayName: 'Player1', isPointsEnabled: true, points: 100 },
    { matchPlayerId: 2, displayName: 'Player2', isPointsEnabled: true, points: 100 },
    { matchPlayerId: 3, displayName: 'Player3', isPointsEnabled: true, points: 100 },
  ]);

  assert.deepEqual(result, []);
});

test('getMatchBattleDeltas ignores disabled players', async () => {
  const { getMatchBattleDeltas } = await import('./matches-list');

  const result = getMatchBattleDeltas([
    { matchPlayerId: 1, displayName: 'Player1', isPointsEnabled: true, points: 150 },
    { matchPlayerId: 2, displayName: 'Player2', isPointsEnabled: false, points: 999 }, // disabled, ignored
    { matchPlayerId: 3, displayName: 'Player3', isPointsEnabled: true, points: 100 },
  ]);

  // Player 2 is disabled so ignored; Player 1 is highest, Player 3 is lowest
  assert.equal(result.find(r => r.matchPlayerId === 1)?.delta, 50);
  assert.equal(result.find(r => r.matchPlayerId === 2)?.delta, 0); // disabled player gets 0
  assert.equal(result.find(r => r.matchPlayerId === 3)?.delta, -50);
});

test('getMatchBattleDeltas returns zero for all when fewer than 2 enabled players', async () => {
  const { getMatchBattleDeltas } = await import('./matches-list');

  const result = getMatchBattleDeltas([
    { matchPlayerId: 1, displayName: 'Player1', isPointsEnabled: true, points: 150 },
    { matchPlayerId: 2, displayName: 'Player2', isPointsEnabled: false, points: 100 },
  ]);

  // Only 1 enabled player, so everyone gets 0
  assert.equal(result.every(r => r.delta === 0), true);
});
