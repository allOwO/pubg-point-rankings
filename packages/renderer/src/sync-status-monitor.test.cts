import assert from 'node:assert/strict';
import test from 'node:test';

test('disables the sync button while cooldown is active', async () => {
  const { isSyncButtonLocked } = await import('./sync-status-monitor');

  assert.equal(
    isSyncButtonLocked({ isSyncing: false, cooldownUntilMs: 5_000, nowMs: 4_000 }),
    true,
  );
  assert.equal(
    isSyncButtonLocked({ isSyncing: false, cooldownUntilMs: 5_000, nowMs: 5_000 }),
    false,
  );
  assert.equal(
    isSyncButtonLocked({ isSyncing: true, cooldownUntilMs: null, nowMs: 5_000 }),
    true,
  );
});

test('returns a success toast only when manual sync transitions to success', async () => {
  const { getManualSyncToastEvent } = await import('./sync-status-monitor');

  assert.equal(
    getManualSyncToastEvent(
      { state: 'syncing', startedAt: null, finishedAt: null, errorMessage: null, trigger: 'manual' },
      { state: 'success', startedAt: null, finishedAt: null, errorMessage: null, trigger: 'manual' },
    ),
    'success',
  );

  assert.equal(
    getManualSyncToastEvent(
      { state: 'success', startedAt: null, finishedAt: null, errorMessage: null, trigger: 'manual' },
      { state: 'success', startedAt: null, finishedAt: null, errorMessage: null, trigger: 'manual' },
    ),
    null,
  );

  assert.equal(
    getManualSyncToastEvent(
      { state: 'syncing', startedAt: null, finishedAt: null, errorMessage: null, trigger: 'manual' },
      { state: 'failed', startedAt: null, finishedAt: null, errorMessage: 'boom', trigger: 'manual' },
    ),
    'failed',
  );
});

test('polls manual sync status only while the manual task is syncing', async () => {
  const { shouldPollManualSyncStatus } = await import('./sync-status-monitor');

  assert.equal(shouldPollManualSyncStatus(null), false);
  assert.equal(
    shouldPollManualSyncStatus({ state: 'success', startedAt: null, finishedAt: null, errorMessage: null, trigger: 'manual' }),
    false,
  );
  assert.equal(
    shouldPollManualSyncStatus({ state: 'syncing', startedAt: null, finishedAt: null, errorMessage: null, trigger: 'manual' }),
    true,
  );
});
