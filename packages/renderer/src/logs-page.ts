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

function getLevelClass(level: LogLevel): string {
  switch (level) {
    case 'ERROR':
      return 'logs-level-error';
    case 'WARN':
      return 'logs-level-warn';
    case 'INFO':
      return 'logs-level-info';
    default:
      return 'logs-level-debug';
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

    if (statusBadge) {
      const isEnabled = this.status?.enabled ?? false;
      statusBadge.className = `logs-status-badge ${isEnabled ? 'is-enabled' : 'is-disabled'}`;
      
      const readyText = statusBadge.querySelector('.ready-text');
      if (readyText) {
        readyText.textContent = isEnabled
          ? this.options.translate('logs.enabled')
          : this.options.translate('logs.disabled');
      }

      const readyDot = statusBadge.querySelector('.ready-dot');
      if (readyDot) {
        readyDot.className = 'ready-dot';
      }
    }
  }

  private renderEntries() {
    const terminal = document.getElementById('logs-terminal');
    const emptyState = document.getElementById('logs-empty');
    if (!terminal || !emptyState) {
      return;
    }

    const filteredEntries = filterLogEntries(this.entries, this.searchTerm, this.levelFilter);

    if (filteredEntries.length === 0) {
      terminal.classList.add('hidden');
      emptyState.classList.remove('hidden');
      terminal.innerHTML = '';
      return;
    }

    terminal.innerHTML = filteredEntries.map((entry) => {
      const timestamp = escapeHtml(this.options.formatDateTime(entry.timestamp));
      const level = escapeHtml(entry.level);
      const source = escapeHtml(entry.source);
      const message = escapeHtml(entry.message);
      const levelClass = getLevelClass(entry.level);

      return `
        <div class="logs-terminal-line">
          <span class="logs-terminal-timestamp">${timestamp}</span>
          <span class="logs-terminal-level ${levelClass}">${level}</span>
          <span class="logs-terminal-source">[${source}]</span>
          <span class="logs-terminal-message">${message}</span>
        </div>
      `;
    }).join('');

    emptyState.classList.add('hidden');
    terminal.classList.remove('hidden');
  }

  private renderErrorState() {
    const terminal = document.getElementById('logs-terminal');
    const emptyState = document.getElementById('logs-empty');
    const emptyTitle = emptyState?.querySelector<HTMLElement>('h3');
    const emptyHint = emptyState?.querySelector<HTMLElement>('p');

    terminal?.classList.add('hidden');
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
