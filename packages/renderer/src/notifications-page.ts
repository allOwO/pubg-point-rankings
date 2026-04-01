import type {
  FailedNotificationSendStatus,
  NotificationEnvStatus,
  NotificationFailedTask,
  NotificationPageStatus,
  NotificationTemplateConfig,
} from '@pubg-point-rankings/shared';

export interface NotificationsPageController {
  load(): Promise<void>;
  refreshStatus(): Promise<void>;
  sendSelected(): Promise<void>;
  deleteTask(taskId: number): Promise<void>;
}

interface FailedTaskState {
  task: NotificationFailedTask;
  selected: boolean;
}

export function shouldShowNotificationLogin(
  status: Pick<NotificationPageStatus, 'envStatus' | 'webUiUrl'> | null,
): boolean {
  return status?.envStatus === 'not_logged_in' && !!status.webUiUrl;
}

export function getCurrentNotificationGroupId(
  status: Pick<NotificationPageStatus, 'groupId'> | null,
): string | null {
  const groupId = status?.groupId.trim();
  return groupId ? groupId : null;
}

type NotificationTemplateLineId = NotificationTemplateConfig['order'][number];
type EditorTemplateLineId = 'header' | 'teammate' | 'battle';

interface EditorTemplateLineConfig {
  id: EditorTemplateLineId;
  prefix: string;
  suffix: string;
}

interface EditorTemplateConfig {
  order: EditorTemplateLineId[];
  lines: Record<EditorTemplateLineId, EditorTemplateLineConfig>;
}

const TEMPLATE_PREVIEW_CONTENT: Record<NotificationTemplateLineId, string> = {
  header: '03-30 06:47｜第4名',
  player1: 'allOwO：1杀 / 143伤 / 0救 / 317分',
  player2: '队友A：0杀 / 88伤 / 1救 / 184分',
  player3: '队友B：2杀 / 201伤 / 0救 / 402分',
  player4: '队友C：0杀 / 0伤 / 0救 / 0分',
  battle: '张三 → 李四 12 分',
};

const TEMPLATE_LINE_LABEL_KEYS: Record<NotificationTemplateLineId, string> = {
  header: 'notifications.templateLine.header',
  player1: 'notifications.templateLine.player1',
  player2: 'notifications.templateLine.player2',
  player3: 'notifications.templateLine.player3',
  player4: 'notifications.templateLine.player4',
  battle: 'notifications.templateLine.battle',
};

const EDITOR_TEMPLATE_LINE_LABEL_KEYS: Record<EditorTemplateLineId, string> = {
  header: 'notifications.templateLine.header',
  teammate: 'notifications.templateLine.teammate',
  battle: 'notifications.templateLine.battle',
};

const TEAMMATE_LINE_IDS: NotificationTemplateLineId[] = ['player1', 'player2', 'player3', 'player4'];

export function buildTemplatePreviewLines(config: NotificationTemplateConfig): string[] {
  return renderConfiguredTemplateLines(config, TEMPLATE_PREVIEW_CONTENT);
}

export function renderConfiguredTemplateLines(
  config: NotificationTemplateConfig,
  content: Record<NotificationTemplateLineId, string>,
): string[] {
  return config.order.flatMap((lineId) => {
    const value = content[lineId]?.trim() ?? '';
    if (!value || value === '-') {
      return [];
    }

    const line = config.lines[lineId];
    return [`${line.prefix}${value}${line.suffix}`];
  });
}

export function collapseTemplateConfigForEditor(config: NotificationTemplateConfig): EditorTemplateConfig {
  const order: EditorTemplateLineId[] = [];
  for (const lineId of config.order) {
    if (TEAMMATE_LINE_IDS.includes(lineId)) {
      if (!order.includes('teammate')) {
        order.push('teammate');
      }
      continue;
    }

    order.push(lineId as EditorTemplateLineId);
  }

  return {
    order,
    lines: {
      header: { ...config.lines.header, id: 'header' },
      teammate: {
        id: 'teammate',
        prefix: config.lines.player1.prefix,
        suffix: config.lines.player1.suffix,
      },
      battle: { ...config.lines.battle, id: 'battle' },
    },
  };
}

export function expandEditorTemplateConfig(config: EditorTemplateConfig): NotificationTemplateConfig {
  const order: NotificationTemplateLineId[] = [];
  for (const lineId of config.order) {
    if (lineId === 'teammate') {
      order.push('player1', 'player2', 'player3', 'player4');
      continue;
    }
    order.push(lineId);
  }

  return {
    order,
    lines: {
      header: { id: 'header', prefix: config.lines.header.prefix, suffix: config.lines.header.suffix },
      player1: { id: 'player1', prefix: config.lines.teammate.prefix, suffix: config.lines.teammate.suffix },
      player2: { id: 'player2', prefix: config.lines.teammate.prefix, suffix: config.lines.teammate.suffix },
      player3: { id: 'player3', prefix: config.lines.teammate.prefix, suffix: config.lines.teammate.suffix },
      player4: { id: 'player4', prefix: config.lines.teammate.prefix, suffix: config.lines.teammate.suffix },
      battle: { id: 'battle', prefix: config.lines.battle.prefix, suffix: config.lines.battle.suffix },
    },
  };
}

export function moveTemplateLine(
  order: NotificationTemplateConfig['order'],
  lineId: NotificationTemplateLineId,
  direction: -1 | 1,
): NotificationTemplateConfig['order'] {
  const currentIndex = order.indexOf(lineId);
  const targetIndex = currentIndex + direction;
  if (currentIndex < 0 || targetIndex < 0 || targetIndex >= order.length) {
    return [...order];
  }

  const nextOrder = [...order];
  const [item] = nextOrder.splice(currentIndex, 1);
  nextOrder.splice(targetIndex, 0, item);
  return nextOrder;
}

function escapeHtml(value: string): string {
  return value
    .split('&').join('&amp;')
    .split('<').join('&lt;')
    .split('>').join('&gt;')
    .split('"').join('&quot;')
    .split("'").join('&#39;');
}

class NotificationsPageControllerImpl implements NotificationsPageController {
  private status: NotificationPageStatus | null = null;
  private templateConfig: NotificationTemplateConfig | null = null;
  private editorTemplateConfig: EditorTemplateConfig | null = null;
  private failedTasks: Map<number, FailedTaskState> = new Map();
  private selectedTasks: Set<number> = new Set();
  private confirmDeleteTaskId: number | null = null;
  private isLoading = false;

  constructor(private api: NotificationAPI) {}

  async load(): Promise<void> {
    this.isLoading = true;
    this.renderLoadingState();

    try {
      const [status, tasks, templateConfig] = await Promise.all([
        this.api.getStatus(),
        this.api.getFailedTasks(),
        this.api.getTemplateConfig(),
      ]);

      this.status = status;
      this.templateConfig = templateConfig;
      this.editorTemplateConfig = collapseTemplateConfigForEditor(templateConfig);
      this.failedTasks.clear();
      for (const task of tasks) {
        this.failedTasks.set(task.id, {
          task,
          selected: this.selectedTasks.has(task.id),
        });
      }

      this.render();
    } catch (error) {
      console.error('Failed to load notification page:', error);
      this.renderErrorState();
    } finally {
      this.isLoading = false;
    }
  }

  async refreshStatus(): Promise<void> {
    try {
      this.status = await this.api.getStatus();
      this.renderHeaderStatus();
    } catch (error) {
      console.error('Failed to refresh status:', error);
    }
  }

  async sendSelected(): Promise<void> {
    const taskIds = Array.from(this.selectedTasks);
    if (taskIds.length === 0) return;

    try {
      // Mark as sending
      for (const taskId of taskIds) {
        const state = this.failedTasks.get(taskId);
        if (state) {
          state.task.sendStatus = 'sending';
        }
      }
      this.renderFailedTable();

      const result = await this.api.sendSelected(taskIds);

      // Update send status based on result
      for (const taskId of result.sentIds) {
        const state = this.failedTasks.get(taskId);
        if (state) {
          state.task.sendStatus = 'sent';
          state.selected = false;
          this.selectedTasks.delete(taskId);
        }
      }

      for (const taskId of result.failedIds) {
        const state = this.failedTasks.get(taskId);
        if (state) {
          state.task.sendStatus = 'failed';
        }
      }

      this.renderFailedTable();
    } catch (error) {
      console.error('Failed to send selected tasks:', error);
      // Reset to failed state
      for (const taskId of taskIds) {
        const state = this.failedTasks.get(taskId);
        if (state) {
          state.task.sendStatus = 'failed';
        }
      }
      this.renderFailedTable();
    }
  }

  async deleteTask(taskId: number): Promise<void> {
    if (this.confirmDeleteTaskId !== taskId) {
      this.confirmDeleteTaskId = taskId;
      this.renderFailedTable();
      return;
    }

    try {
      await this.api.deleteFailedTask(taskId);
      this.failedTasks.delete(taskId);
      this.selectedTasks.delete(taskId);
      this.confirmDeleteTaskId = null;
      this.renderFailedTable();
    } catch (error) {
      console.error('Failed to delete task:', error);
      this.confirmDeleteTaskId = null;
      this.renderFailedTable();
    }
  }

  private toggleTaskSelection(taskId: number): void {
    const state = this.failedTasks.get(taskId);
    if (!state) return;

    state.selected = !state.selected;
    if (state.selected) {
      this.selectedTasks.add(taskId);
    } else {
      this.selectedTasks.delete(taskId);
    }

    this.renderFailedTable();
  }

  private selectAllTasks(): void {
    let allSelected = true;
    for (const state of this.failedTasks.values()) {
      if (!state.selected) {
        allSelected = false;
        break;
      }
    }

    const newSelected = !allSelected;
    for (const [taskId, state] of this.failedTasks) {
      state.selected = newSelected;
      if (newSelected) {
        this.selectedTasks.add(taskId);
      } else {
        this.selectedTasks.delete(taskId);
      }
    }

    this.renderFailedTable();
  }

  private render(): void {
    this.renderHeaderStatus();
    this.renderRuntimeCard();
    this.renderLoginCard();
    this.renderConfigCard();
    this.renderFailedTable();
    this.renderTemplateEditor();
  }

  private renderHeaderStatus(): void {
    const headerStatus = document.getElementById('notification-header-status');
    const readyBadge = document.getElementById('notification-header-ready-badge');
    const runtimeSpan = document.getElementById('notification-header-runtime');

    if (!headerStatus || !this.status) return;

    const isReady = this.status.envStatus === 'ready';
    const isError = this.status.envStatus === 'unsupported_os' || this.status.lastError;

    // Update ready badge styling and text
    if (readyBadge) {
      readyBadge.className = 'notification-header-ready-badge';
      if (isError) {
        readyBadge.classList.add('error');
      } else if (!isReady) {
        readyBadge.classList.add('not-ready');
      }

      const readyText = readyBadge.querySelector('.ready-text');
      if (readyText) {
        readyText.textContent = this.getStatusText(this.status.envStatus);
      }
    }

    // Update runtime version
    if (runtimeSpan) {
      runtimeSpan.textContent = this.status.runtimeVersion
        ? `Runtime: ${this.status.runtimeVersion}`
        : '';
    }
  }

  private renderRuntimeCard(): void {
    const card = document.getElementById('notifications-runtime-card');
    if (!card) return;

    const isUnsupported = this.status?.envStatus === 'unsupported_os';
    const isMissing = this.status?.envStatus === 'missing_runtime';
    const canStart = this.status?.envStatus === 'runtime_not_running';
    const canInstall = isMissing && !!this.status?.canInstallRuntime;
    const isRunning = this.status?.envStatus === 'not_logged_in' ||
                      this.status?.envStatus === 'missing_group_id' ||
                      this.status?.envStatus === 'ready';

    card.innerHTML = `
      <div class="card-header">
        <h3 data-i18n="notifications.runtimeTitle">Runtime</h3>
      </div>
      <div class="card-body">
        <div class="notification-runtime-controls">
          ${canInstall ? `
            <button type="button" class="btn btn-primary" id="btn-install-runtime" data-i18n="notifications.installRuntime">
              Download & Install
            </button>
          ` : ''}
          ${canStart ? `
            <button type="button" class="btn btn-primary" id="btn-start-runtime" data-i18n="notifications.startRuntime">
              Start Runtime
            </button>
          ` : ''}
          ${isRunning ? `
            <div class="notification-runtime-actions">
              <button type="button" class="btn btn-secondary" id="btn-restart-runtime" data-i18n="notifications.restartRuntime">
                Restart Runtime
              </button>
              <button type="button" class="btn btn-secondary" id="btn-stop-runtime" data-i18n="notifications.stopRuntime">
                Stop Runtime
              </button>
            </div>
          ` : ''}
        </div>
      </div>
    `;

    // Attach event listeners
    const installBtn = card.querySelector('#btn-install-runtime');
    if (installBtn) {
      installBtn.addEventListener('click', () => this.handleInstallRuntime());
    }

    const stopBtn = card.querySelector('#btn-stop-runtime');
    if (stopBtn) {
      stopBtn.addEventListener('click', () => this.handleStopRuntime());
    }

    const startBtn = card.querySelector('#btn-start-runtime');
    if (startBtn) {
      startBtn.addEventListener('click', () => this.handleStartRuntime());
    }

    const restartBtn = card.querySelector('#btn-restart-runtime');
    if (restartBtn) {
      restartBtn.addEventListener('click', () => this.handleRestartRuntime());
    }
  }

  private renderLoginCard(): void {
    const card = document.getElementById('notifications-login-card');
    if (!card || !this.status) return;

    const showLoginCard = shouldShowNotificationLogin(this.status);
    card.hidden = !showLoginCard;

    if (!showLoginCard) {
      card.innerHTML = '';
      return;
    }

    card.innerHTML = `
      <div class="card-header">
        <h3 data-i18n="notifications.loginTitle">QQ Login</h3>
      </div>
      <div class="card-body">
        ${this.status.webUiUrl ? `
          <div class="notification-webui-container">
            <iframe 
              src="${this.status.webUiUrl}" 
              class="notification-webui-frame"
              sandbox="allow-same-origin allow-scripts allow-forms"
              title="NapCat WebUI"
            ></iframe>
          </div>
        ` : `
          <div class="notification-webui-placeholder">
            <p data-i18n="notifications.webUiNotAvailable">WebUI not available. Start the runtime first.</p>
          </div>
        `}
      </div>
    `;
  }

  private renderConfigCard(): void {
    const card = document.getElementById('notifications-config-card');
    if (!card) return;

    card.hidden = false;
    const canConfigure = this.status?.envStatus === 'not_logged_in' ||
                        this.status?.envStatus === 'missing_group_id' ||
                        this.status?.envStatus === 'ready';
    const currentGroupId = getCurrentNotificationGroupId(this.status);

    card.innerHTML = `
      <div class="card-header">
        <h3 data-i18n="notifications.configTitle">Configuration</h3>
      </div>
      <div class="card-body">
        ${canConfigure ? `
          <div class="notification-config-form">
            ${currentGroupId ? `
              <p class="notification-current-group">
                ${this.api.translate('notifications.currentGroup')}: <strong>${escapeHtml(currentGroupId)}</strong>
              </p>
            ` : ''}
            <div class="form-group">
              <label for="notification-group-id" data-i18n="notifications.groupId">Group ID</label>
              <input type="text" id="notification-group-id" class="form-input" 
                value="${escapeHtml(this.status?.groupId || '')}" 
                placeholder="Enter QQ group ID">
            </div>
            <div class="form-actions">
              <button type="button" class="btn btn-primary" id="btn-save-group-id" data-i18n="common.save">
                Save
              </button>
              <button type="button" class="btn btn-secondary" id="btn-send-test" data-i18n="notifications.sendTest" ${this.status?.envStatus === 'ready' ? '' : 'disabled'}>
                Send Test
              </button>
            </div>
          </div>
        ` : `
          <div class="notification-config-placeholder">
            <p data-i18n="notifications.configNotAvailable">Configuration available after login.</p>
          </div>
        `}
      </div>
    `;

    const saveBtn = card.querySelector('#btn-save-group-id');
    if (saveBtn) {
      saveBtn.addEventListener('click', () => this.handleSaveGroupId());
    }

    const sendTestBtn = card.querySelector('#btn-send-test');
    if (sendTestBtn) {
      sendTestBtn.addEventListener('click', () => this.handleSendTest());
    }
  }

  private renderFailedTable(): void {
    const card = document.getElementById('notifications-failed-card');
    if (!card) return;

    const hasTasks = this.failedTasks.size > 0;
    const selectedCount = this.selectedTasks.size;

    card.innerHTML = `
      <div class="card-header">
        <h3 data-i18n="notifications.failedTitle">Failed Notifications</h3>
        ${hasTasks ? `<span class="notification-failed-count">${this.failedTasks.size}</span>` : ''}
      </div>
      <div class="card-body">
        ${!hasTasks ? `
          <div class="notification-failed-empty">
            <p data-i18n="notifications.noFailedTasks">No failed notifications.</p>
          </div>
        ` : `
          <div class="notification-failed-toolbar">
            <button type="button" class="btn btn-secondary btn-sm" id="btn-select-all-failed" data-i18n="notifications.selectAll">
              Select All
            </button>
            <button type="button" class="btn btn-primary btn-sm" id="btn-send-selected" ${selectedCount === 0 ? 'disabled' : ''}>
              <span data-i18n="notifications.sendSelected">Send Selected</span>
              ${selectedCount > 0 ? ` (${selectedCount})` : ''}
            </button>
          </div>
          <div class="table-wrapper notification-failed-table-wrapper">
            <table class="data-table notification-failed-table">
              <thead>
                <tr>
                  <th class="checkbox-cell">
                    <input type="checkbox" id="select-all-failed-checkbox" ${this.areAllSelected() ? 'checked' : ''}>
                  </th>
                  <th data-i18n="notifications.matchTime">Match Time</th>
                  <th data-i18n="notifications.placement">Rank</th>
                  <th data-i18n="notifications.battleSummary">Battle Summary</th>
                  <th data-i18n="notifications.lastError">Error</th>
                  <th data-i18n="notifications.sendStatus">Status</th>
                  <th data-i18n="notifications.actions">Actions</th>
                </tr>
              </thead>
              <tbody>
                ${Array.from(this.failedTasks.values()).map(({ task, selected }) => `
                  <tr data-task-id="${task.id}" class="${task.sendStatus === 'sending' ? 'sending' : ''}">
                    <td class="checkbox-cell">
                      <input type="checkbox" class="task-select-checkbox" data-task-id="${task.id}" ${selected ? 'checked' : ''} ${task.sendStatus === 'sending' ? 'disabled' : ''}>
                    </td>
                    <td class="match-time-cell">${this.formatDateTime(task.matchTime)}</td>
                    <td class="placement-cell">${task.placement ? `#${task.placement}` : '-'}</td>
                    <td class="battle-summary-cell" title="${task.battleSummary}">${task.battleSummary}</td>
                    <td class="error-cell" title="${task.lastError || ''}">${task.lastError || '-'}</td>
                    <td class="send-status-cell">
                      <span class="notification-send-state ${task.sendStatus}">${this.getSendStatusText(task.sendStatus)}</span>
                    </td>
                    <td class="actions-cell">
                      ${this.confirmDeleteTaskId === task.id ? `
                       <div class="delete-confirm">
                         <span class="delete-confirm-text" data-i18n="notifications.confirmDelete">Confirm?</span>
                          <button type="button" class="btn btn-danger btn-xs" data-action="confirm-delete" data-task-id="${task.id}">${this.api.translate('modal.confirm')}</button>
                          <button type="button" class="btn btn-secondary btn-xs" data-action="cancel-delete" data-task-id="${task.id}">${this.api.translate('modal.cancel')}</button>
                        </div>
                      ` : `
                        <button type="button" class="btn btn-danger btn-xs btn-delete" data-action="delete" data-task-id="${task.id}" data-i18n="notifications.delete" ${task.sendStatus === 'sending' ? 'disabled' : ''}>Delete</button>
                      `}
                    </td>
                  </tr>
                `).join('')}
              </tbody>
            </table>
          </div>
        `}
      </div>
    `;

    this.attachFailedTableListeners();
  }

  private renderTemplateEditor(): void {
    const card = document.getElementById('notifications-template-card');
    if (!card) return;

    if (!this.editorTemplateConfig) {
      card.innerHTML = `
        <div class="card-header">
          <h3 data-i18n="notifications.templateTitle">Message Template</h3>
        </div>
        <div class="card-body">
          <div class="template-line-placeholder">
            <p>${this.api.translate('notifications.loading')}</p>
          </div>
        </div>
      `;
      return;
    }

    const preview = escapeHtml(buildTemplatePreviewLines(this.templateConfig ?? expandEditorTemplateConfig(this.editorTemplateConfig)).join('\n'));

    card.innerHTML = `
      <div class="card-header">
        <h3 data-i18n="notifications.templateTitle">Message Template</h3>
      </div>
      <div class="card-body">
        <div class="notification-template-editor">
          <div class="template-editor-hint">
            <p data-i18n="notifications.templateHint">Configure the order and prefix/suffix for each line of the notification message.</p>
          </div>
          <div class="template-lines" id="template-lines-container">
            ${this.editorTemplateConfig.order.map((lineId, index) => {
              const line = this.editorTemplateConfig?.lines[lineId];
              if (!line) return '';
              return `
                <div class="template-line-card" data-line-id="${lineId}">
                  <div class="template-line-header-row">
                    <div>
                      <strong>${this.api.translate(EDITOR_TEMPLATE_LINE_LABEL_KEYS[lineId])}</strong>
                      <p class="template-line-example">${escapeHtml(lineId === 'teammate' ? TEMPLATE_PREVIEW_CONTENT.player1 : TEMPLATE_PREVIEW_CONTENT[lineId])}</p>
                    </div>
                    <div class="template-line-order-actions">
                      <button type="button" class="btn btn-secondary btn-sm" data-action="move-line" data-line-id="${lineId}" data-direction="-1" ${index === 0 ? 'disabled' : ''}>↑</button>
                      <button type="button" class="btn btn-secondary btn-sm" data-action="move-line" data-line-id="${lineId}" data-direction="1" ${index === this.editorTemplateConfig!.order.length - 1 ? 'disabled' : ''}>↓</button>
                    </div>
                  </div>
                  <div class="template-line-fields">
                    <label>
                      <span>${this.api.translate('notifications.templatePrefix')}</span>
                      <input type="text" class="form-input" data-action="template-prefix" data-line-id="${lineId}" value="${escapeHtml(line.prefix)}">
                    </label>
                    <label>
                      <span>${this.api.translate('notifications.templateSuffix')}</span>
                      <input type="text" class="form-input" data-action="template-suffix" data-line-id="${lineId}" value="${escapeHtml(line.suffix)}">
                    </label>
                  </div>
                </div>
              `;
            }).join('')}
          </div>
          <div class="notification-template-preview">
            <h4>${this.api.translate('notifications.templatePreview')}</h4>
            <pre id="notification-template-preview" class="notification-template-preview-text">${preview}</pre>
          </div>
          <div class="form-actions">
            <button type="button" class="btn btn-primary" id="btn-save-template-config" data-i18n="common.save">Save</button>
          </div>
        </div>
      </div>
    `;

    this.attachTemplateEditorListeners(card);
  }

  private attachTemplateEditorListeners(card: HTMLElement): void {
    const moveButtons = card.querySelectorAll('[data-action="move-line"]');
    moveButtons.forEach((button) => {
      button.addEventListener('click', (event) => {
        const target = event.currentTarget as HTMLButtonElement;
        const lineId = target.dataset.lineId as EditorTemplateLineId;
        const direction = Number(target.dataset.direction) as -1 | 1;
        if (!this.editorTemplateConfig) return;
        this.editorTemplateConfig = {
          ...this.editorTemplateConfig,
          order: moveTemplateLine(this.editorTemplateConfig.order as unknown as NotificationTemplateConfig['order'], lineId as NotificationTemplateLineId, direction) as unknown as EditorTemplateLineId[],
        };
        this.renderTemplateEditor();
      });
    });

    const prefixInputs = card.querySelectorAll('[data-action="template-prefix"]');
    prefixInputs.forEach((input) => {
      input.addEventListener('input', (event) => {
        const target = event.currentTarget as HTMLInputElement;
        this.updateTemplateLineAffix(target.dataset.lineId as EditorTemplateLineId, 'prefix', target.value);
      });
    });

    const suffixInputs = card.querySelectorAll('[data-action="template-suffix"]');
    suffixInputs.forEach((input) => {
      input.addEventListener('input', (event) => {
        const target = event.currentTarget as HTMLInputElement;
        this.updateTemplateLineAffix(target.dataset.lineId as EditorTemplateLineId, 'suffix', target.value);
      });
    });

    const saveButton = card.querySelector('#btn-save-template-config');
    if (saveButton) {
      saveButton.addEventListener('click', () => this.handleSaveTemplateConfig());
    }
  }

  private updateTemplateLineAffix(
    lineId: EditorTemplateLineId,
    field: 'prefix' | 'suffix',
    value: string,
  ): void {
    if (!this.editorTemplateConfig) return;
    this.editorTemplateConfig = {
      ...this.editorTemplateConfig,
      lines: {
        ...this.editorTemplateConfig.lines,
        [lineId]: {
          ...this.editorTemplateConfig.lines[lineId],
          [field]: value,
        },
      },
    };
    this.templateConfig = expandEditorTemplateConfig(this.editorTemplateConfig);

    const preview = document.getElementById('notification-template-preview');
    if (preview && this.templateConfig) {
      preview.textContent = buildTemplatePreviewLines(this.templateConfig).join('\n');
    }
  }

  private renderLoadingState(): void {
    const sections = [
      'notifications-status-card',
      'notifications-runtime-card',
      'notifications-login-card',
      'notifications-config-card',
      'notifications-failed-card',
      'notifications-template-card',
    ];

    for (const id of sections) {
      const el = document.getElementById(id);
      if (el) {
        el.innerHTML = `
          <div class="card-body">
              <div class="loading-spinner">
                <div class="spinner"></div>
                <span>${this.api.translate('notifications.loading')}</span>
              </div>
            </div>
          `;
      }
    }
  }

  private renderErrorState(): void {
    const sections = [
      'notifications-status-card',
      'notifications-runtime-card',
      'notifications-login-card',
      'notifications-config-card',
      'notifications-failed-card',
      'notifications-template-card',
    ];

    for (const id of sections) {
      const el = document.getElementById(id);
      if (el) {
        el.innerHTML = `
          <div class="card-body">
            <div class="error-message">
              <span>${this.api.translate('notifications.loadFailed')}</span>
            </div>
          </div>
        `;
      }
    }
  }

  private attachFailedTableListeners(): void {
    const card = document.getElementById('notifications-failed-card');
    if (!card) return;

    // Select all checkbox
    const selectAllCheckbox = card.querySelector('#select-all-failed-checkbox') as HTMLInputElement | null;
    if (selectAllCheckbox) {
      selectAllCheckbox.addEventListener('change', () => {
        this.selectAllTasks();
      });
    }

    // Individual checkboxes
    const checkboxes = card.querySelectorAll('.task-select-checkbox');
    checkboxes.forEach((checkbox) => {
      checkbox.addEventListener('change', (e) => {
        const target = e.target as HTMLInputElement;
        const taskId = Number(target.dataset.taskId);
        this.toggleTaskSelection(taskId);
      });
    });

    // Select all button
    const selectAllBtn = card.querySelector('#btn-select-all-failed');
    if (selectAllBtn) {
      selectAllBtn.addEventListener('click', () => this.selectAllTasks());
    }

    // Send selected button
    const sendSelectedBtn = card.querySelector('#btn-send-selected');
    if (sendSelectedBtn) {
      sendSelectedBtn.addEventListener('click', () => this.sendSelected());
    }

    // Delete buttons
    const deleteBtns = card.querySelectorAll('[data-action="delete"]');
    deleteBtns.forEach((btn) => {
      btn.addEventListener('click', (e) => {
        const target = e.currentTarget as HTMLElement;
        const taskId = Number(target.dataset.taskId);
        this.deleteTask(taskId);
      });
    });

    // Confirm delete buttons
    const confirmDeleteBtns = card.querySelectorAll('[data-action="confirm-delete"]');
    confirmDeleteBtns.forEach((btn) => {
      btn.addEventListener('click', (e) => {
        const target = e.currentTarget as HTMLElement;
        const taskId = Number(target.dataset.taskId);
        this.confirmDeleteTaskId = taskId;
        this.deleteTask(taskId);
      });
    });

    // Cancel delete buttons
    const cancelDeleteBtns = card.querySelectorAll('[data-action="cancel-delete"]');
    cancelDeleteBtns.forEach((btn) => {
      btn.addEventListener('click', (e) => {
        this.confirmDeleteTaskId = null;
        this.renderFailedTable();
      });
    });
  }

  private areAllSelected(): boolean {
    if (this.failedTasks.size === 0) return false;
    for (const state of this.failedTasks.values()) {
      if (!state.selected) return false;
    }
    return true;
  }

  private getStatusClass(status: NotificationEnvStatus): string {
    const statusClasses: Record<NotificationEnvStatus, string> = {
      unsupported_os: 'error',
      missing_runtime: 'warning',
      runtime_not_running: 'warning',
      not_logged_in: 'info',
      missing_group_id: 'info',
      ready: 'success',
    };
    return statusClasses[status] || 'default';
  }

  private getStatusText(status: NotificationEnvStatus): string {
    const statusTexts: Record<NotificationEnvStatus, string> = {
      unsupported_os: 'Unsupported OS',
      missing_runtime: this.api.translate('notifications.state.missing_runtime'),
      runtime_not_running: this.api.translate('notifications.state.runtime_not_running'),
      not_logged_in: this.api.translate('notifications.state.not_logged_in'),
      missing_group_id: this.api.translate('notifications.state.missing_group_id'),
      ready: this.api.translate('notifications.state.ready'),
    };
    return status === 'unsupported_os'
      ? this.api.translate('notifications.state.unsupported_os')
      : statusTexts[status] || status;
  }

  private getSendStatusText(status: FailedNotificationSendStatus): string {
    const statusTexts: Record<FailedNotificationSendStatus, string> = {
      sending: this.api.translate('notifications.sendState.sending'),
      sent: this.api.translate('notifications.sendState.sent'),
      failed: this.api.translate('notifications.sendState.failed'),
    };
    return statusTexts[status] || status;
  }

  private formatDateTime(date: Date): string {
    const locale = document.documentElement.lang || undefined;
    return date.toLocaleString(locale, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  private async handleInstallRuntime(): Promise<void> {
    try {
      this.status = await this.api.installRuntime();
      this.render();
    } catch (error) {
      console.error('Failed to install runtime:', error);
    }
  }

  private async handleStopRuntime(): Promise<void> {
    try {
      await this.api.stopRuntime();
      await this.refreshStatus();
    } catch (error) {
      console.error('Failed to stop runtime:', error);
    }
  }

  private async handleStartRuntime(): Promise<void> {
    try {
      this.status = await this.api.startRuntime();
      this.render();
    } catch (error) {
      console.error('Failed to start runtime:', error);
    }
  }

  private async handleRestartRuntime(): Promise<void> {
    try {
      this.status = await this.api.restartRuntime();
      this.render();
    } catch (error) {
      console.error('Failed to restart runtime:', error);
    }
  }

  private async handleSaveGroupId(): Promise<void> {
    const input = document.getElementById('notification-group-id') as HTMLInputElement | null;
    if (!input) return;

    const groupId = input.value.trim();
    if (!groupId) return;

    try {
      this.status = await this.api.saveGroupId(groupId);
      this.render();
    } catch (error) {
      console.error('Failed to save group ID:', error);
    }
  }

  private async handleSaveTemplateConfig(): Promise<void> {
    if (!this.editorTemplateConfig) return;

    try {
      this.templateConfig = expandEditorTemplateConfig(this.editorTemplateConfig);
      this.templateConfig = await this.api.saveTemplateConfig(this.templateConfig);
      this.editorTemplateConfig = collapseTemplateConfigForEditor(this.templateConfig);
      this.renderTemplateEditor();
    } catch (error) {
      console.error('Failed to save notification template config:', error);
    }
  }

  private async handleSendTest(): Promise<void> {
    try {
      await this.api.sendTest();
    } catch (error) {
      console.error('Failed to send notification test:', error);
    }
  }
}

interface NotificationAPI {
  getStatus(): Promise<NotificationPageStatus>;
  getFailedTasks(): Promise<NotificationFailedTask[]>;
  getTemplateConfig(): Promise<NotificationTemplateConfig>;
  sendSelected(taskIds: number[]): Promise<{ sentIds: number[]; failedIds: number[] }>;
  deleteFailedTask(taskId: number): Promise<void>;
  installRuntime(): Promise<NotificationPageStatus>;
  startRuntime(): Promise<NotificationPageStatus>;
  stopRuntime(): Promise<void>;
  restartRuntime(): Promise<NotificationPageStatus>;
  sendTest(): Promise<void>;
  saveGroupId(groupId: string): Promise<NotificationPageStatus>;
  saveTemplateConfig(config: NotificationTemplateConfig): Promise<NotificationTemplateConfig>;
  translate(key: string): string;
}

export function createNotificationsPageController(api: NotificationAPI): NotificationsPageController {
  return new NotificationsPageControllerImpl(api);
}
