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

class NotificationsPageControllerImpl implements NotificationsPageController {
  private status: NotificationPageStatus | null = null;
  private failedTasks: Map<number, FailedTaskState> = new Map();
  private selectedTasks: Set<number> = new Set();
  private confirmDeleteTaskId: number | null = null;
  private isLoading = false;

  constructor(private api: NotificationAPI) {}

  async load(): Promise<void> {
    this.isLoading = true;
    this.renderLoadingState();

    try {
      const [status, tasks] = await Promise.all([
        this.api.getStatus(),
        this.api.getFailedTasks(),
      ]);

      this.status = status;
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
      this.renderStatusCard();
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
    this.renderStatusCard();
    this.renderRuntimeCard();
    this.renderLoginCard();
    this.renderConfigCard();
    this.renderFailedTable();
    this.renderTemplateEditor();
  }

  private renderStatusCard(): void {
    const card = document.getElementById('notifications-status-card');
    if (!card || !this.status) return;

    const statusClass = this.getStatusClass(this.status.envStatus);
    const statusText = this.getStatusText(this.status.envStatus);

    card.innerHTML = `
      <div class="card-header">
        <h3 data-i18n="notifications.statusTitle">Status</h3>
        <span class="notification-status-pill ${statusClass}">${statusText}</span>
      </div>
      <div class="card-body">
        <div class="notification-status-details">
          ${this.status.runtimeVersion ? `<p>Runtime: ${this.status.runtimeVersion}</p>` : ''}
          ${this.status.lastError ? `<p class="error-text">Error: ${this.status.lastError}</p>` : ''}
        </div>
      </div>
    `;
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

    const showWebUi = this.status.webUiUrl && this.status.envStatus !== 'unsupported_os';

    card.innerHTML = `
      <div class="card-header">
        <h3 data-i18n="notifications.loginTitle">QQ Login</h3>
      </div>
      <div class="card-body">
        ${showWebUi ? `
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

    const canConfigure = this.status?.envStatus === 'not_logged_in' ||
                        this.status?.envStatus === 'missing_group_id' ||
                        this.status?.envStatus === 'ready';

    card.innerHTML = `
      <div class="card-header">
        <h3 data-i18n="notifications.configTitle">Configuration</h3>
      </div>
      <div class="card-body">
        ${canConfigure ? `
          <div class="notification-config-form">
            <div class="form-group">
              <label for="notification-group-id" data-i18n="notifications.groupId">Group ID</label>
              <input type="text" id="notification-group-id" class="form-input" 
                value="${this.status?.groupId || ''}" 
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
            <div class="template-line-placeholder">
              <p data-i18n="notifications.templateNotAvailable">Template editor will be available after the backend implementation.</p>
            </div>
          </div>
        </div>
      </div>
    `;
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
  sendSelected(taskIds: number[]): Promise<{ sentIds: number[]; failedIds: number[] }>;
  deleteFailedTask(taskId: number): Promise<void>;
  installRuntime(): Promise<NotificationPageStatus>;
  startRuntime(): Promise<NotificationPageStatus>;
  stopRuntime(): Promise<void>;
  restartRuntime(): Promise<NotificationPageStatus>;
  sendTest(): Promise<void>;
  saveGroupId(groupId: string): Promise<NotificationPageStatus>;
  translate(key: string): string;
}

export function createNotificationsPageController(api: NotificationAPI): NotificationsPageController {
  return new NotificationsPageControllerImpl(api);
}
