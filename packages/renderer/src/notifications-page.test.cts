import assert from 'node:assert/strict';
import test from 'node:test';

test('returns success tone for a completed manual sync', async () => {
  const { getManualSyncStatusTone } = await import('./notifications-page');

  assert.equal(
    getManualSyncStatusTone({
      state: 'success',
      startedAt: new Date('2026-04-01T08:00:00Z'),
      finishedAt: new Date('2026-04-01T08:00:05Z'),
      errorMessage: null,
      trigger: 'manual',
    }),
    'success',
  );
});

test('returns error tone for a failed manual sync', async () => {
  const { getManualSyncStatusTone } = await import('./notifications-page');

  assert.equal(
    getManualSyncStatusTone({
      state: 'failed',
      startedAt: new Date('2026-04-01T08:00:00Z'),
      finishedAt: new Date('2026-04-01T08:00:05Z'),
      errorMessage: 'boom',
      trigger: 'manual',
    }),
    'error',
  );
});
