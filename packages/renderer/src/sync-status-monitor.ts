import type { ManualSyncTaskStatus } from '@pubg-point-rankings/shared';

export function isSyncButtonLocked(input: {
  isSyncing: boolean;
  cooldownUntilMs: number | null;
  nowMs: number;
}): boolean {
  if (input.isSyncing) return true;
  return input.cooldownUntilMs !== null && input.nowMs < input.cooldownUntilMs;
}

export function getManualSyncToastEvent(
  previous: ManualSyncTaskStatus | null,
  next: ManualSyncTaskStatus | null,
): 'success' | 'failed' | null {
  if (!previous || !next) return null;
  if (previous.state === next.state) return null;
  if (next.state === 'success') return 'success';
  if (next.state === 'failed') return 'failed';
  return null;
}

export function shouldPollManualSyncStatus(status: ManualSyncTaskStatus | null): boolean {
  return status?.state === 'syncing';
}
