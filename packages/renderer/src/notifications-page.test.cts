import assert from 'node:assert/strict';
import test from 'node:test';

test('shows QQ login only while not logged in', async () => {
  const { shouldShowNotificationLogin } = await import('./notifications-page.js');

  assert.equal(shouldShowNotificationLogin({ envStatus: 'not_logged_in', webUiUrl: 'http://127.0.0.1:6099' }), true);
  assert.equal(shouldShowNotificationLogin({ envStatus: 'missing_group_id', webUiUrl: 'http://127.0.0.1:6099' }), false);
  assert.equal(shouldShowNotificationLogin({ envStatus: 'ready', webUiUrl: 'http://127.0.0.1:6099' }), false);
});

test('returns current QQ group id text when a group is configured', async () => {
  const { getCurrentNotificationGroupId } = await import('./notifications-page.js');

  assert.equal(getCurrentNotificationGroupId({ groupId: '123456' }), '123456');
  assert.equal(getCurrentNotificationGroupId({ groupId: '   ' }), null);
  assert.equal(getCurrentNotificationGroupId(null), null);
});

test('builds template preview lines in configured order with affixes', async () => {
  const { buildTemplatePreviewLines } = await import('./notifications-page.js');

  const lines = buildTemplatePreviewLines({
    order: ['battle', 'header', 'player1', 'player2', 'player3', 'player4'],
    lines: {
      header: { id: 'header', prefix: '[', suffix: ']' },
      player1: { id: 'player1', prefix: '', suffix: ' <-1' },
      player2: { id: 'player2', prefix: '', suffix: '' },
      player3: { id: 'player3', prefix: '', suffix: '' },
      player4: { id: 'player4', prefix: '', suffix: '' },
      battle: { id: 'battle', prefix: 'RESULT: ', suffix: '' },
    },
  });

  assert.equal(lines[0], 'RESULT: 张三 → 李四 12 分');
  assert.equal(lines[1], '[03-30 06:47｜第4名]');
  assert.equal(lines[2], 'allOwO：1杀 / 143伤 / 0救 / 317分 <-1');
});

test('moves template line within fixed order', async () => {
  const { moveTemplateLine } = await import('./notifications-page.js');

  assert.deepEqual(
    moveTemplateLine(['header', 'player1', 'player2', 'player3', 'player4', 'battle'], 'player2', -1),
    ['header', 'player2', 'player1', 'player3', 'player4', 'battle'],
  );
  assert.deepEqual(
    moveTemplateLine(['header', 'player1', 'player2', 'player3', 'player4', 'battle'], 'header', -1),
    ['header', 'player1', 'player2', 'player3', 'player4', 'battle'],
  );
});

test('omits empty player lines from rendered template output', async () => {
  const { renderConfiguredTemplateLines } = await import('./notifications-page.js');

  assert.deepEqual(
    renderConfiguredTemplateLines(
      {
        order: ['header', 'player1', 'player2', 'player3', 'player4', 'battle'],
        lines: {
          header: { id: 'header', prefix: '', suffix: '' },
          player1: { id: 'player1', prefix: '', suffix: '' },
          player2: { id: 'player2', prefix: '', suffix: '' },
          player3: { id: 'player3', prefix: '', suffix: '' },
          player4: { id: 'player4', prefix: '', suffix: '' },
          battle: { id: 'battle', prefix: '', suffix: '' },
        },
      },
      {
        header: '03-30 15:59｜第4名',
        player1: 'allOwO：3杀 / 284伤 / 0救 / 659分',
        player2: 'JiNiTaiMei202301：2杀 / 493伤 / 0救 / 1048分',
        player3: 'qwer1122vv：0杀 / 180伤 / 0救 / 360分',
        player4: '',
        battle: 'qwer1122vv → JiNiTaiMei202301 688 分',
      },
    ),
    [
      '03-30 15:59｜第4名',
      'allOwO：3杀 / 284伤 / 0救 / 659分',
      'JiNiTaiMei202301：2杀 / 493伤 / 0救 / 1048分',
      'qwer1122vv：0杀 / 180伤 / 0救 / 360分',
      'qwer1122vv → JiNiTaiMei202301 688 分',
    ],
  );
});

test('collapses four teammate template rows into one shared editor row', async () => {
  const { collapseTemplateConfigForEditor } = await import('./notifications-page.js');

  const editorConfig = collapseTemplateConfigForEditor({
    order: ['battle', 'player1', 'player2', 'player3', 'player4', 'header'],
    lines: {
      header: { id: 'header', prefix: '[', suffix: ']' },
      player1: { id: 'player1', prefix: 'P:', suffix: ':S' },
      player2: { id: 'player2', prefix: 'ignored', suffix: 'ignored' },
      player3: { id: 'player3', prefix: 'ignored', suffix: 'ignored' },
      player4: { id: 'player4', prefix: 'ignored', suffix: 'ignored' },
      battle: { id: 'battle', prefix: '', suffix: '!' },
    },
  });

  assert.deepEqual(editorConfig.order, ['battle', 'teammate', 'header']);
  assert.deepEqual(editorConfig.lines.teammate, { id: 'teammate', prefix: 'P:', suffix: ':S' });
});

test('expands shared teammate editor row back into four teammate template rows', async () => {
  const { expandEditorTemplateConfig } = await import('./notifications-page.js');

  const templateConfig = expandEditorTemplateConfig({
    order: ['header', 'teammate', 'battle'],
    lines: {
      header: { id: 'header', prefix: '[', suffix: ']' },
      teammate: { id: 'teammate', prefix: 'TEAM:', suffix: ':END' },
      battle: { id: 'battle', prefix: 'RESULT: ', suffix: '' },
    },
  });

  assert.deepEqual(templateConfig.order, ['header', 'player1', 'player2', 'player3', 'player4', 'battle']);
  assert.deepEqual(templateConfig.lines.player1, { id: 'player1', prefix: 'TEAM:', suffix: ':END' });
  assert.deepEqual(templateConfig.lines.player4, { id: 'player4', prefix: 'TEAM:', suffix: ':END' });
});

test('returns success tone for a completed manual sync', async () => {
  const { getManualSyncStatusTone } = await import('./notifications-page.js');

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
  const { getManualSyncStatusTone } = await import('./notifications-page.js');

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
