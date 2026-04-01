import type { LogEntry, LogLevel, LogStatus } from '@pubg-point-rankings/shared';

export interface LogsPageController {
  load(): Promise<void>;
}

interface LogsPageAPI {
  getStatus(): Promise<LogStatus>;
  getRecent(limit?: number): Promise<LogEntry[]>;
  openDirectory(): Promise<void>;
}

interface LogsPageControllerOptions extends LogsPageAPI {
  translate: (key: string) => string;
  formatDateTime: (value: Date) => string;
  showToast: (message: string, type?: 'success' | 'error' | 'warning') => void;
}

type LogLevelFilter = 'ALL' | LogLevel;

function escapeHtml(value: string): string {
  return value
    .split('&').join('&amp;')
    .split('<').join('&lt;')
    .split('>').join('&gt;')
    .split('"').join('&quot;')
    .split("'").join('&#39;');
}

function formatLogLine(entry: LogEntry, formatDateTime: (value: Date) => string): string {
  return `${formatDateTime(entry.timestamp)} [${entry.level}] [${entry.source}] ${entry.message}`;
}

function getLevelBadgeClass(level: LogLevel): string {
  switch (level) {
    case 'ERROR':
      return 'badge-error';
    case 'WARN':
      return 'badge-warning';
    case 'INFO':
      return 'badge-info';
    default:
      return 'badge-success';
  }
}

export function filterLogEntries(entries: LogEntry[], searchTerm: string, level: LogLevelFilter): LogEntry[] {
  const query = searchTerm.trim().toLowerCase();

  return entries.filter((entry) => {
    if (level !== 'ALL' && entry.level !== level) {
      return false;
    }

    if (!query) {
      return true;
    }

    return [entry.source, entry.message, entry.level]
      .some((field) => field.toLowerCase().includes(query));
  });
}

class LogsPageControllerImpl implements LogsPageController {
  private status: LogStatus | null = null;
  private entries: LogEntry[] = [];
  private isBound = false;
  private searchTerm = '';
  private levelFilter: LogLevelFilter = 'ALL';

  constructor(private options: LogsPageControllerOptions) {}

  async load(): Promise<void> {
    this.bindEvents();

    try {
      const [status, entries] = await Promise.all([
        this.options.getStatus(),
        this.options.getRecent(500),
      ]);

      this.status = status;
      this.entries = entries;
      this.render();
    } catch (error) {
      console.error('Failed to load logs page:', error);
      this.renderErrorState();
      this.options.showToast(this.options.translate('toast.logsLoadFailed'), 'error');
    }
  }

  private bindEvents() {
    if (this.isBound) {
      return;
    }

    document.getElementById('btn-logs-refresh')?.addEventListener('click', () => {
      void this.load();
    });

    document.getElementById('btn-logs-open-directory')?.addEventListener('click', async () => {
      try {
        await this.options.openDirectory();
      } catch (error) {
        console.error('Failed to open logs directory:', error);
        this.options.showToast(this.options.translate('toast.logsOpenDirectoryFailed'), 'error');
      }
    });

    document.getElementById('btn-logs-copy')?.addEventListener('click', async () => {
      try {
        const filteredEntries = filterLogEntries(this.entries, this.searchTerm, this.levelFilter);
        const content = filteredEntries
          .map((entry) => formatLogLine(entry, this.options.formatDateTime))
          .join('\n');
        await navigator.clipboard.writeText(content);
        this.options.showToast(this.options.translate('toast.logsCopied'));
      } catch (error) {
        console.error('Failed to copy logs:', error);
        this.options.showToast(this.options.translate('toast.logsCopyFailed'), 'error');
      }
    });

    document.getElementById('logs-search')?.addEventListener('input', (event) => {
      this.searchTerm = (event.target as HTMLInputElement).value;
      this.renderEntries();
    });

    document.getElementById('logs-level-filter')?.addEventListener('change', (event) => {
      this.levelFilter = (event.target as HTMLSelectElement).value as LogLevelFilter;
      this.renderEntries();
    });

    this.isBound = true;
  }

  private render() {
    this.renderHeader();
    this.renderEntries();
  }

  private renderHeader() {
    const statusBadge = document.getElementById('logs-status-badge');
    const directoryEl = document.getElementById('logs-directory-value');
    const fileEl = document.getElementById('logs-file-value');

    if (statusBadge) {
      statusBadge.textContent = this.status?.enabled
        ? this.options.translate('logs.enabled')
        : this.options.translate('logs.disabled');
      statusBadge.className = `badge ${this.status?.enabled ? 'badge-success' : 'badge-warning'}`;
    }

    if (directoryEl) {
      directoryEl.textContent = this.status?.directory ?? '--';
    }

    if (fileEl) {
      fileEl.textContent = this.status?.logFilePath ?? this.options.translate('logs.noFileYet');
    }
  }

  private renderEntries() {
    const tableWrapper = document.getElementById('logs-table-wrapper');
    const tbody = document.getElementById('logs-table-body');
    const emptyState = document.getElementById('logs-empty');
    const countEl = document.getElementById('logs-count');
    if (!tableWrapper || !tbody || !emptyState || !countEl) {
      return;
    }

    const filteredEntries = filterLogEntries(this.entries, this.searchTerm, this.levelFilter);
    countEl.textContent = `${filteredEntries.length}`;

    if (filteredEntries.length === 0) {
      tableWrapper.classList.add('hidden');
      emptyState.classList.remove('hidden');
      tbody.innerHTML = '';
      return;
    }

    tbody.innerHTML = filteredEntries.map((entry) => `
      <tr>
        <td class="logs-table-timestamp">${escapeHtml(this.options.formatDateTime(entry.timestamp))}</td>
        <td><span class="badge ${getLevelBadgeClass(entry.level)}">${escapeHtml(entry.level)}</span></td>
        <td class="logs-table-source">${escapeHtml(entry.source)}</td>
        <td class="logs-table-message">${escapeHtml(entry.message)}</td>
      </tr>
    `).join('');

    emptyState.classList.add('hidden');
    tableWrapper.classList.remove('hidden');
  }

  private renderErrorState() {
    const tableWrapper = document.getElementById('logs-table-wrapper');
    const emptyState = document.getElementById('logs-empty');
    const countEl = document.getElementById('logs-count');
    const emptyTitle = emptyState?.querySelector<HTMLElement>('h3');
    const emptyHint = emptyState?.querySelector<HTMLElement>('p');

    if (countEl) {
      countEl.textContent = '0';
    }
    tableWrapper?.classList.add('hidden');
    emptyState?.classList.remove('hidden');
    if (emptyTitle) {
      emptyTitle.textContent = this.options.translate('logs.loadFailedTitle');
    }
    if (emptyHint) {
      emptyHint.textContent = this.options.translate('logs.loadFailedHint');
    }
  }
}

export function createLogsPageController(options: LogsPageControllerOptions): LogsPageController {
  return new LogsPageControllerImpl(options);
}
