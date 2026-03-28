import assert from 'node:assert/strict';
import test from 'node:test';

test('renders Match Log in English for the match detail log section', async () => {
  const { translate } = await import('./i18n.js');
  assert.equal(translate('en-US', 'detail.activityLog'), 'Match Log');
});

test('renders 比赛日志 in Chinese for the match detail log section', async () => {
  const { translate } = await import('./i18n.js');
  assert.equal(translate('zh-CN', 'detail.activityLog'), '比赛日志');
});
