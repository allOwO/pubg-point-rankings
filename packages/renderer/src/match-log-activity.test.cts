import assert from 'node:assert/strict';
import test from 'node:test';

test('formats Chinese knock sentence with weapon', async () => {
  const { formatActivitySentence } = await import('./match-log-activity.js');

  assert.equal(
    formatActivitySentence({
      locale: 'zh-CN',
      type: 'knock',
      actionText: '击倒',
      targetName: '李四',
      extra: 'ACE32',
    }),
    '用 ACE32 击倒了 李四',
  );
});

test('formats Chinese elimination sentence without weapon', async () => {
  const { formatActivitySentence } = await import('./match-log-activity.js');

  assert.equal(
    formatActivitySentence({
      locale: 'zh-CN',
      type: 'kill',
      actionText: '淘汰',
      targetName: '李四',
      extra: null,
    }),
    '淘汰了 李四',
  );
});

test('formats English revive sentence', async () => {
  const { formatActivitySentence } = await import('./match-log-activity.js');

  assert.equal(
    formatActivitySentence({
      locale: 'en-US',
      type: 'revive',
      actionText: 'revived',
      targetName: 'Lisi',
      extra: null,
    }),
    'revived Lisi',
  );
});

test('formats English elimination sentence with weapon', async () => {
  const { formatActivitySentence } = await import('./match-log-activity.js');

  assert.equal(
    formatActivitySentence({
      locale: 'en-US',
      type: 'kill',
      actionText: 'eliminated',
      targetName: 'Lisi',
      extra: 'ACE32',
    }),
    'eliminated Lisi with ACE32',
  );
});
