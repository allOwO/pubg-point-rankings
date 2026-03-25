import { getAPI, getRuntimeHost } from './tauri-api';

/**
 * PUBG Point Rankings - Renderer Application
 * Main entry point for the renderer process
 */

declare global {
  interface Window {
    editTeammateNickname?: (id: number) => Promise<void>;
    editRule?: (id: number) => Promise<void>;
    activateRule?: (id: number) => Promise<void>;
    deleteRule?: (id: number) => Promise<void>;
    viewMatchDetail?: (matchId: string) => Promise<void>;
  }
}

// Type definitions
interface Teammate {
  id: number;
  platform: 'steam' | 'xbox' | 'psn' | 'kakao';
  pubgAccountId: string | null;
  pubgPlayerName: string;
  displayNickname: string | null;
  isPointsEnabled: boolean;
  totalPoints: number;
  lastSeenAt: Date | null;
  createdAt: Date;
  updatedAt: Date;
}

interface Match {
  id: number;
  matchId: string;
  platform: 'steam' | 'xbox' | 'psn' | 'kakao';
  mapName: string | null;
  gameMode: string | null;
  playedAt: Date;
  matchStartAt: Date | null;
  matchEndAt: Date | null;
  telemetryUrl: string | null;
  status: 'detected' | 'syncing' | 'ready' | 'failed';
  createdAt: Date;
  updatedAt: Date;
}

interface MatchPlayer {
  id: number;
  matchId: string;
  teammateId: number | null;
  pubgAccountId: string | null;
  pubgPlayerName: string;
  displayNicknameSnapshot: string | null;
  teamId: number | null;
  damage: number;
  kills: number;
  revives: number;
  placement: number | null;
  isSelf: boolean;
  isPointsEnabledSnapshot: boolean;
  points: number;
  createdAt: Date;
}

interface PointRule {
  id: number;
  name: string;
  damagePointsPerDamage: number;
  killPoints: number;
  revivePoints: number;
  isActive: boolean;
  roundingMode: 'floor' | 'round' | 'ceil';
  createdAt: Date;
  updatedAt: Date;
}

interface PointRecord {
  id: number;
  matchId: string;
  matchPlayerId: number;
  teammateId: number | null;
  ruleId: number;
  ruleNameSnapshot: string;
  damagePointsPerDamageSnapshot: number;
  killPointsSnapshot: number;
  revivePointsSnapshot: number;
  roundingModeSnapshot: 'floor' | 'round' | 'ceil';
  points: number;
  note: string | null;
  createdAt: Date;
}

interface SyncStatus {
  isSyncing: boolean;
  lastSyncAt: Date | null;
  currentMatchId: string | null;
  error: string | null;
}

interface AppStatus {
  version: string;
  databasePath: string;
  isDatabaseReady: boolean;
  syncStatus: SyncStatus;
}

interface GameProcessStatus {
  state: 'not_running' | 'running' | 'cooldown_polling';
  lastSeenRunningAtMs: number | null;
  cooldownStartedAtMs: number | null;
  lastProcessCheckAtMs: number | null;
  lastRecentMatchCheckAtMs: number | null;
}

interface CalculatedPoints {
  pubgAccountId: string;
  pubgPlayerName: string;
  damage: number;
  kills: number;
  revives: number;
  damagePoints: number;
  killPoints: number;
  revivePoints: number;
  totalPoints: number;
  isPointsEnabled: boolean;
}

interface PollingSettings {
  autoRecentMatchEnabled: boolean;
  runningProcessCheckIntervalSeconds: number;
  notRunningProcessCheckIntervalSeconds: number;
  runningRecentMatchIntervalSeconds: number;
  cooldownPollingIntervalSeconds: number;
  cooldownWindowMinutes: number;
  recentMatchRetryDelaySeconds: number;
  recentMatchRetryLimit: number;
}

const POLLING_SETTING_KEYS = {
  autoRecentMatchEnabled: 'auto_recent_match_enabled',
  runningProcessCheckIntervalSeconds: 'running_process_check_interval_seconds',
  notRunningProcessCheckIntervalSeconds: 'not_running_process_check_interval_seconds',
  runningRecentMatchIntervalSeconds: 'running_recent_match_interval_seconds',
  cooldownPollingIntervalSeconds: 'cooldown_polling_interval_seconds',
  cooldownWindowMinutes: 'cooldown_window_minutes',
  recentMatchRetryDelaySeconds: 'recent_match_retry_delay_seconds',
  recentMatchRetryLimit: 'recent_match_retry_limit',
} as const;

const DEFAULT_POLLING_SETTINGS: PollingSettings = {
  autoRecentMatchEnabled: true,
  runningProcessCheckIntervalSeconds: 5,
  notRunningProcessCheckIntervalSeconds: 30,
  runningRecentMatchIntervalSeconds: 30,
  cooldownPollingIntervalSeconds: 120,
  cooldownWindowMinutes: 40,
  recentMatchRetryDelaySeconds: 15,
  recentMatchRetryLimit: 2,
};

interface CreateTeammateInput {
  platform: 'steam' | 'xbox' | 'psn' | 'kakao';
  pubgAccountId: string | null;
  pubgPlayerName: string;
  displayNickname: string | null;
  isPointsEnabled?: boolean;
}

interface UpdateTeammateInput {
  id: number;
  displayNickname?: string | null;
  isPointsEnabled?: boolean;
}

interface CreatePointRuleInput {
  name: string;
  damagePointsPerDamage: number;
  killPoints: number;
  revivePoints: number;
  roundingMode?: 'floor' | 'round' | 'ceil';
}

interface UpdatePointRuleInput {
  id: number;
  name?: string;
  damagePointsPerDamage?: number;
  killPoints?: number;
  revivePoints?: number;
  roundingMode?: 'floor' | 'round' | 'ceil';
}

type PlatformValue = CreateTeammateInput['platform'];
type RoundingValue = NonNullable<CreatePointRuleInput['roundingMode']>;

function isPlatformValue(value: string): value is PlatformValue {
  return ['steam', 'xbox', 'psn', 'kakao'].includes(value);
}

function isRoundingValue(value: string): value is RoundingValue {
  return ['floor', 'round', 'ceil'].includes(value);
}

function getFriendIdentifier(teammate: Teammate): string {
  return teammate.pubgAccountId || teammate.pubgPlayerName;
}

function updateTeammateModalMode(mode: 'create' | 'nickname', teammate?: Teammate) {
  const titleEl = document.getElementById('teammate-modal-title');
  const nameInput = document.getElementById('teammate-name') as HTMLInputElement | null;
  const platformSelect = document.getElementById('teammate-platform') as HTMLSelectElement | null;
  const nicknameInput = document.getElementById('teammate-nickname') as HTMLInputElement | null;
  const enabledGroup = document.getElementById('teammate-enabled-group');
  const enabledInput = document.getElementById('teammate-enabled') as HTMLInputElement | null;

  if (mode === 'create') {
    if (titleEl) titleEl.textContent = '添加好友';
    if (nameInput) {
      nameInput.disabled = false;
      nameInput.value = '';
    }
    if (platformSelect) {
      platformSelect.disabled = false;
      platformSelect.value = 'steam';
    }
    if (nicknameInput) nicknameInput.value = '';
    if (enabledInput) enabledInput.checked = true;
    enabledGroup?.classList.remove('hidden');
    return;
  }

  if (!teammate) return;

  if (titleEl) {
    titleEl.textContent = teammate.displayNickname ? '修改昵称' : '添加昵称';
  }
  if (nameInput) {
    nameInput.disabled = true;
    nameInput.value = getFriendIdentifier(teammate);
  }
  if (platformSelect) {
    platformSelect.disabled = true;
    platformSelect.value = teammate.platform;
  }
  if (nicknameInput) nicknameInput.value = teammate.displayNickname || '';
  if (enabledInput) enabledInput.checked = teammate.isPointsEnabled;
  enabledGroup?.classList.add('hidden');
}

function openCreateTeammateModal() {
  const form = document.getElementById('teammate-form') as HTMLFormElement | null;
  const idInput = document.getElementById('teammate-id') as HTMLInputElement | null;

  form?.reset();
  if (idInput) idInput.value = '';
  updateTeammateModalMode('create');
  openModal('modal-teammate');
}

async function handleManualTeammateSync() {
  const syncButton = document.getElementById('btn-new-teammate') as HTMLButtonElement | null;
  const dashboardSyncButton = document.getElementById('btn-add-teammate') as HTMLButtonElement | null;

  if (state.syncStatus?.isSyncing) return;

  try {
    if (syncButton) syncButton.disabled = true;
    if (dashboardSyncButton) dashboardSyncButton.disabled = true;

    const api = getAPI();
    const result = await api.sync.start();

    if (!result.success) {
      showToast(result.error || '手动同步好友失败', 'error');
      return;
    }

    showToast('好友列表同步完成');
    await Promise.all([loadDashboard(), loadTeammates(), loadMatches(), loadPointRecords()]);
  } catch (error) {
    console.error('Failed to sync teammates manually:', error);
    showToast('手动同步好友失败', 'error');
  } finally {
    await loadAppStatus();
    if (syncButton) syncButton.disabled = false;
    if (dashboardSyncButton) dashboardSyncButton.disabled = false;
    setSyncNowButtonState();
  }
}

// State management
export class AppState {
  teammates: Teammate[] = [];
  rules: PointRule[] = [];
  activeRule: PointRule | null = null;
  matches: Match[] = [];
  pointRecords: PointRecord[] = [];
  syncStatus: SyncStatus | null = null;
  appStatus: AppStatus | null = null;
  gameProcessStatus: GameProcessStatus | null = null;
  pollingSettings: PollingSettings = { ...DEFAULT_POLLING_SETTINGS };
  isLoading = false;
}

export const state = new AppState();

// Utility functions
function formatPoints(points: number): string {
  return `${Math.round(points).toLocaleString()} pts`;
}

function formatDate(date: Date | string | null): string {
  if (!date) return 'Never';
  const d = new Date(date);
  return d.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}

function formatDateTime(date: Date | string | null): string {
  if (!date) return 'Never';
  const d = new Date(date);
  return d.toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function truncateMatchId(matchId: string): string {
  return matchId.slice(0, 8) + '...';
}

function parseBooleanSetting(value: string | undefined, fallback: boolean): boolean {
  if (!value) return fallback;
  const normalized = value.trim().toLowerCase();
  if (['1', 'true', 'yes', 'on'].includes(normalized)) return true;
  if (['0', 'false', 'no', 'off'].includes(normalized)) return false;
  return fallback;
}

function parseNumberSetting(value: string | undefined, fallback: number): number {
  if (!value) return fallback;
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) ? parsed : fallback;
}

function setSyncNowButtonState() {
  const syncNowButton = document.getElementById('btn-sync-now') as HTMLButtonElement | null;
  const label = document.getElementById('btn-sync-now-label');
  if (!syncNowButton) return;

  const isSyncing = state.syncStatus?.isSyncing ?? false;
  syncNowButton.disabled = isSyncing;
  if (label) {
    label.textContent = isSyncing ? 'Syncing Latest Match...' : 'Sync Now';
  }
}

// Toast notifications
export function showToast(message: string, type: 'success' | 'error' | 'warning' = 'success') {
  const container = document.getElementById('toast-container');
  if (!container) return;

  const toast = document.createElement('div');
  toast.className = `toast ${type}`;
  toast.innerHTML = `
    <span class="toast-message">${message}</span>
    <button class="toast-close">&times;</button>
  `;

  toast.querySelector('.toast-close')?.addEventListener('click', () => {
    toast.remove();
  });

  container.appendChild(toast);

  setTimeout(() => {
    toast.remove();
  }, 5000);
}

// Loading screen
function showLoadingScreen() {
  document.getElementById('loading-screen')?.classList.remove('hidden');
  document.getElementById('main-app')?.classList.add('hidden');
  document.getElementById('error-screen')?.classList.add('hidden');
}

function hideLoadingScreen() {
  document.getElementById('loading-screen')?.classList.add('hidden');
  document.getElementById('main-app')?.classList.remove('hidden');
}

function showErrorScreen(message: string) {
  document.getElementById('loading-screen')?.classList.add('hidden');
  document.getElementById('main-app')?.classList.add('hidden');
  document.getElementById('error-screen')?.classList.remove('hidden');
  const errorMessage = document.getElementById('error-message');
  if (errorMessage) {
    errorMessage.textContent = message;
  }
}

// Modal management
function openModal(modalId: string) {
  const overlay = document.getElementById('modal-overlay');
  const modal = document.getElementById(modalId);
  if (overlay && modal) {
    overlay.classList.remove('hidden');
    modal.classList.remove('hidden');
  }
}

function closeAllModals() {
  const overlay = document.getElementById('modal-overlay');
  const modals = document.querySelectorAll('.modal');
  const teammateIdInput = document.getElementById('teammate-id') as HTMLInputElement | null;
  if (overlay) {
    overlay.classList.add('hidden');
  }
  modals.forEach(modal => {
    modal.classList.add('hidden');
  });
  if (teammateIdInput) teammateIdInput.value = '';
}

// Navigation
export function navigateTo(viewId: string) {
  // Update nav items
  document.querySelectorAll('.nav-item').forEach(item => {
    item.classList.remove('active');
    if (item.getAttribute('data-view') === viewId) {
      item.classList.add('active');
    }
  });

  // Update views
  document.querySelectorAll('.view').forEach(view => {
    view.classList.remove('active');
  });
  const targetView = document.getElementById(`view-${viewId}`);
  if (targetView) {
    targetView.classList.add('active');
  }

  // Load view data
  switch (viewId) {
    case 'dashboard':
      loadDashboard();
      break;
    case 'teammates':
      loadTeammates();
      break;
    case 'rules':
      loadRules();
      break;
    case 'matches':
      loadMatches();
      break;
    case 'points':
      loadPointRecords();
      break;
  }
}

// Data loading functions
async function loadAppStatus() {
  try {
    const api = getAPI();
    const [appStatus, gameProcessStatus] = await Promise.all([
      api.app.getStatus(),
      api.app.getGameProcessStatus(),
    ]);
    state.appStatus = appStatus;
    state.gameProcessStatus = gameProcessStatus;
    state.syncStatus = state.appStatus.syncStatus;
    updateSyncIndicator();
  } catch (error) {
    console.error('Failed to load app status:', error);
  }
}

async function loadDashboard() {
  try {
    const api = getAPI();
    
    // Load status
    const [appStatus, gameProcessStatus] = await Promise.all([
      api.app.getStatus(),
      api.app.getGameProcessStatus(),
    ]);
    state.appStatus = appStatus;
    state.gameProcessStatus = gameProcessStatus;
    state.syncStatus = state.appStatus.syncStatus;
    
    // Update UI
    updateDashboardStatus();
    updateSyncIndicator();
    await loadPollingSettings();
    
    // Load recent matches
    const matches = await api.matches.getAll(5, 0);
    state.matches = matches;
    renderRecentMatches();
    
    // Load teammates
    const teammates = await api.teammates.getAll();
    state.teammates = teammates;
    renderTopTeammates();
    
    // Load active rule
    const activeRule = await api.rules.getActive();
    state.activeRule = activeRule;
    updateActiveRule();
    
    // Load total point records
    const pointRecords = await api.points.getAll(1, 0);
    const totalPointsEl = document.getElementById('total-points');
    if (totalPointsEl) {
      totalPointsEl.textContent = pointRecords.length.toString();
    }
  } catch (error) {
    console.error('Failed to load dashboard:', error);
    showToast('Failed to load dashboard data', 'error');
  }
}

async function loadPollingSettings() {
  try {
    const api = getAPI();
    const settings = await api.settings.getAll();
    const values = new Map(settings.map(setting => [setting.key, setting.value]));

    const pollingSettings: PollingSettings = {
      autoRecentMatchEnabled: parseBooleanSetting(
        values.get(POLLING_SETTING_KEYS.autoRecentMatchEnabled),
        DEFAULT_POLLING_SETTINGS.autoRecentMatchEnabled,
      ),
      runningProcessCheckIntervalSeconds: parseNumberSetting(
        values.get(POLLING_SETTING_KEYS.runningProcessCheckIntervalSeconds),
        DEFAULT_POLLING_SETTINGS.runningProcessCheckIntervalSeconds,
      ),
      notRunningProcessCheckIntervalSeconds: parseNumberSetting(
        values.get(POLLING_SETTING_KEYS.notRunningProcessCheckIntervalSeconds),
        DEFAULT_POLLING_SETTINGS.notRunningProcessCheckIntervalSeconds,
      ),
      runningRecentMatchIntervalSeconds: parseNumberSetting(
        values.get(POLLING_SETTING_KEYS.runningRecentMatchIntervalSeconds),
        DEFAULT_POLLING_SETTINGS.runningRecentMatchIntervalSeconds,
      ),
      cooldownPollingIntervalSeconds: parseNumberSetting(
        values.get(POLLING_SETTING_KEYS.cooldownPollingIntervalSeconds),
        DEFAULT_POLLING_SETTINGS.cooldownPollingIntervalSeconds,
      ),
      cooldownWindowMinutes: parseNumberSetting(
        values.get(POLLING_SETTING_KEYS.cooldownWindowMinutes),
        DEFAULT_POLLING_SETTINGS.cooldownWindowMinutes,
      ),
      recentMatchRetryDelaySeconds: parseNumberSetting(
        values.get(POLLING_SETTING_KEYS.recentMatchRetryDelaySeconds),
        DEFAULT_POLLING_SETTINGS.recentMatchRetryDelaySeconds,
      ),
      recentMatchRetryLimit: parseNumberSetting(
        values.get(POLLING_SETTING_KEYS.recentMatchRetryLimit),
        DEFAULT_POLLING_SETTINGS.recentMatchRetryLimit,
      ),
    };

    state.pollingSettings = pollingSettings;
    applyPollingSettingsToForm();
  } catch (error) {
    console.error('Failed to load polling settings:', error);
  }
}

function applyPollingSettingsToForm() {
  const autoEnabled = document.getElementById('setting-auto-recent-match-enabled') as HTMLInputElement | null;
  const runningProcessCheck = document.getElementById('setting-running-process-check-interval-seconds') as HTMLInputElement | null;
  const notRunningProcessCheck = document.getElementById('setting-not-running-process-check-interval-seconds') as HTMLInputElement | null;
  const runningRecentMatch = document.getElementById('setting-running-recent-match-interval-seconds') as HTMLInputElement | null;
  const cooldownPolling = document.getElementById('setting-cooldown-polling-interval-seconds') as HTMLInputElement | null;
  const cooldownWindow = document.getElementById('setting-cooldown-window-minutes') as HTMLInputElement | null;
  const retryDelay = document.getElementById('setting-recent-match-retry-delay-seconds') as HTMLInputElement | null;
  const retryLimit = document.getElementById('setting-recent-match-retry-limit') as HTMLInputElement | null;

  if (autoEnabled) autoEnabled.checked = state.pollingSettings.autoRecentMatchEnabled;
  if (runningProcessCheck) runningProcessCheck.value = state.pollingSettings.runningProcessCheckIntervalSeconds.toString();
  if (notRunningProcessCheck) notRunningProcessCheck.value = state.pollingSettings.notRunningProcessCheckIntervalSeconds.toString();
  if (runningRecentMatch) runningRecentMatch.value = state.pollingSettings.runningRecentMatchIntervalSeconds.toString();
  if (cooldownPolling) cooldownPolling.value = state.pollingSettings.cooldownPollingIntervalSeconds.toString();
  if (cooldownWindow) cooldownWindow.value = state.pollingSettings.cooldownWindowMinutes.toString();
  if (retryDelay) retryDelay.value = state.pollingSettings.recentMatchRetryDelaySeconds.toString();
  if (retryLimit) retryLimit.value = state.pollingSettings.recentMatchRetryLimit.toString();
}

function updateDashboardStatus() {
  if (!state.appStatus) return;
  
  const dbStatus = document.getElementById('db-status');
  const runtimeStatus = document.getElementById('runtime-status');
  const gameProcessStatus = document.getElementById('game-process-status');
  const currentMatchStatus = document.getElementById('current-match-status');
  const lastSync = document.getElementById('last-sync');
  const systemBadge = document.getElementById('system-status-badge');
  
  if (dbStatus) {
    dbStatus.textContent = state.appStatus.isDatabaseReady ? 'Connected' : 'Error';
    dbStatus.className = 'status-value ' + (state.appStatus.isDatabaseReady ? 'text-success' : 'text-error');
  }
  
  if (lastSync) {
    lastSync.textContent = formatDateTime(state.syncStatus?.lastSyncAt ?? null);
  }

  if (runtimeStatus) {
    runtimeStatus.textContent = getRuntimeHost() === 'tauri' ? 'Tauri 2' : 'Electron';
    runtimeStatus.className = 'status-value text-success';
  }

  if (gameProcessStatus) {
    const processState = state.gameProcessStatus?.state ?? 'not_running';

    if (processState === 'running') {
      gameProcessStatus.textContent = 'Running';
      gameProcessStatus.className = 'status-value text-success';
    } else if (processState === 'cooldown_polling') {
      gameProcessStatus.textContent = 'Cooldown Polling';
      gameProcessStatus.className = 'status-value text-warning';
    } else {
      gameProcessStatus.textContent = 'Not Running';
      gameProcessStatus.className = 'status-value text-muted';
    }
  }

  if (currentMatchStatus) {
    const currentMatchId = state.syncStatus?.currentMatchId;
    currentMatchStatus.textContent = currentMatchId ? truncateMatchId(currentMatchId) : 'Idle';
    currentMatchStatus.className = 'status-value ' + (currentMatchId ? 'text-success' : 'text-muted');
  }

  if (systemBadge) {
    if (state.appStatus.isDatabaseReady && !state.syncStatus?.error) {
      systemBadge.textContent = 'Ready';
      systemBadge.className = 'badge badge-success';
    } else if (state.appStatus.isDatabaseReady) {
      systemBadge.textContent = 'Attention';
      systemBadge.className = 'badge badge-warning';
    } else {
      systemBadge.textContent = 'Error';
      systemBadge.className = 'badge badge-error';
    }
  }
  
  const versionEl = document.getElementById('app-version');
  if (versionEl) {
    versionEl.textContent = state.appStatus.version;
  }
}

function updateSyncIndicator() {
  const indicator = document.getElementById('sync-status-indicator');
  if (!indicator || !state.syncStatus) return;
  
  const dot = indicator.querySelector('.status-dot');
  const text = indicator.querySelector('.status-text');
  
  if (state.syncStatus.isSyncing) {
    dot?.classList.add('syncing');
    dot?.classList.remove('error');
    if (text) text.textContent = 'Syncing...';
  } else if (state.syncStatus.error) {
    dot?.classList.remove('syncing');
    dot?.classList.add('error');
    if (text) text.textContent = 'Sync Error';
  } else {
    dot?.classList.remove('syncing', 'error');
    if (text) text.textContent = 'Ready';
  }

  setSyncNowButtonState();
}

function updateActiveRule() {
  const activeRuleEl = document.getElementById('active-rule');
  if (activeRuleEl) {
    activeRuleEl.textContent = state.activeRule?.name || 'None';
  }
}

function renderRecentMatches() {
  const emptyEl = document.getElementById('recent-matches-empty');
  const tableEl = document.getElementById('recent-matches-table');
  const listEl = document.getElementById('recent-matches-list');
  
  if (!emptyEl || !tableEl || !listEl) return;
  
  if (state.matches.length === 0) {
    emptyEl.classList.remove('hidden');
    tableEl.classList.add('hidden');
    return;
  }
  
  emptyEl.classList.add('hidden');
  tableEl.classList.remove('hidden');
  
  listEl.innerHTML = state.matches.map(match => `
    <tr>
      <td>${truncateMatchId(match.matchId)}</td>
      <td>${match.mapName || 'Unknown'}</td>
      <td>${match.gameMode || 'Unknown'}</td>
      <td>${formatDate(match.playedAt)}</td>
      <td><span class="status-badge status-${match.status}">${match.status}</span></td>
    </tr>
  `).join('');
}

function renderTopTeammates() {
  const emptyEl = document.getElementById('top-teammates-empty');
  const listEl = document.getElementById('top-teammates-list');
  
  if (!emptyEl || !listEl) return;
  
  const enabledTeammates = state.teammates
    .filter(t => t.isPointsEnabled)
    .sort((a, b) => b.totalPoints - a.totalPoints)
    .slice(0, 4);
  
  if (enabledTeammates.length === 0) {
    emptyEl.classList.remove('hidden');
    listEl.classList.add('hidden');
    return;
  }
  
  emptyEl.classList.add('hidden');
  listEl.classList.remove('hidden');
  
  listEl.innerHTML = enabledTeammates.map(teammate => `
    <div class="teammate-card ${teammate.isPointsEnabled ? 'enabled' : 'disabled'}">
      <div class="card-title">${teammate.displayNickname || teammate.pubgPlayerName}</div>
      <div class="card-subtitle">${teammate.pubgPlayerName}</div>
      <div class="card-stats">
        <div class="card-stat">
          <div class="card-stat-value">${formatPoints(teammate.totalPoints)}</div>
          <div class="card-stat-label">Total</div>
        </div>
      </div>
    </div>
  `).join('');
}

// Teammates view
async function loadTeammates() {
  try {
    const api = getAPI();
    const teammates = await api.teammates.getAll();
    state.teammates = teammates;
    renderTeammatesList();
  } catch (error) {
    console.error('Failed to load teammates:', error);
    showToast('Failed to load teammates', 'error');
  }
}

function renderTeammatesList() {
  const emptyEl = document.getElementById('teammates-empty');
  const listEl = document.getElementById('teammates-list');
  
  if (!emptyEl || !listEl) return;
  
  if (state.teammates.length === 0) {
    emptyEl.classList.remove('hidden');
    listEl.classList.add('hidden');
    return;
  }
  
  emptyEl.classList.add('hidden');
  listEl.classList.remove('hidden');
  
  listEl.innerHTML = state.teammates.map(teammate => `
    <div class="friend-row ${teammate.isPointsEnabled ? 'enabled' : 'disabled'}">
      <div class="friend-row-main">
        <div class="friend-row-label">${teammate.pubgAccountId ? '用户 ID' : '用户标识'}</div>
        <div class="friend-row-value">${getFriendIdentifier(teammate)}</div>
      </div>
      <div class="friend-row-main">
        <div class="friend-row-label">保存昵称</div>
        <div class="friend-row-value ${teammate.displayNickname ? '' : 'muted'}">${teammate.displayNickname || '未设置'}</div>
      </div>
      <div class="card-actions friend-row-actions">
        <button class="btn btn-secondary" onclick="editTeammateNickname(${teammate.id})">${teammate.displayNickname ? '修改昵称' : '添加昵称'}</button>
      </div>
    </div>
  `).join('');
}

// Rules view
async function loadRules() {
  try {
    const api = getAPI();
    const [rules, activeRule] = await Promise.all([
      api.rules.getAll(),
      api.rules.getActive(),
    ]);
    state.rules = rules;
    state.activeRule = activeRule;
    renderRulesList();
  } catch (error) {
    console.error('Failed to load rules:', error);
    showToast('Failed to load rules', 'error');
  }
}

function renderRulesList() {
  const emptyEl = document.getElementById('rules-empty');
  const listEl = document.getElementById('rules-list');
  
  if (!emptyEl || !listEl) return;
  
  if (state.rules.length === 0) {
    emptyEl.classList.remove('hidden');
    listEl.classList.add('hidden');
    return;
  }
  
  emptyEl.classList.add('hidden');
  listEl.classList.remove('hidden');
  
  listEl.innerHTML = state.rules.map(rule => `
    <div class="rule-card ${rule.isActive ? 'active' : ''}">
      <div class="card-title">${rule.name}</div>
      ${rule.isActive ? '<span class="badge badge-success">Active</span>' : ''}
      <div class="card-stats">
        <div class="card-stat">
          <div class="card-stat-value">${rule.damagePointsPerDamage}</div>
          <div class="card-stat-label">pts/DMG</div>
        </div>
        <div class="card-stat">
          <div class="card-stat-value">${rule.killPoints}</div>
          <div class="card-stat-label">pts/Kill</div>
        </div>
        <div class="card-stat">
          <div class="card-stat-value">${rule.revivePoints}</div>
          <div class="card-stat-label">pts/Revive</div>
        </div>
      </div>
      <div class="card-actions">
        ${!rule.isActive ? `<button class="btn btn-secondary" onclick="activateRule(${rule.id})">Activate</button>` : ''}
        <button class="btn btn-secondary" onclick="editRule(${rule.id})">Edit</button>
        <button class="btn btn-danger" onclick="deleteRule(${rule.id})">Delete</button>
      </div>
    </div>
  `).join('');
}

// Matches view
async function loadMatches() {
  try {
    const api = getAPI();
    const matches = await api.matches.getAll(20, 0);
    state.matches = matches;
    renderMatchesList();
  } catch (error) {
    console.error('Failed to load matches:', error);
    showToast('Failed to load matches', 'error');
  }
}

function renderMatchesList() {
  const emptyEl = document.getElementById('matches-empty');
  const containerEl = document.getElementById('matches-container');
  const listEl = document.getElementById('matches-list');
  
  if (!emptyEl || !containerEl || !listEl) return;
  
  if (state.matches.length === 0) {
    emptyEl.classList.remove('hidden');
    containerEl.classList.add('hidden');
    return;
  }
  
  emptyEl.classList.add('hidden');
  containerEl.classList.remove('hidden');
  
  listEl.innerHTML = state.matches.map(match => `
    <tr>
      <td>${truncateMatchId(match.matchId)}</td>
      <td>${match.mapName || 'Unknown'}</td>
      <td>${match.gameMode || 'Unknown'}</td>
      <td>${formatDateTime(match.playedAt)}</td>
      <td><span class="status-badge status-${match.status}">${match.status}</span></td>
      <td>
        <button class="btn btn-secondary" onclick="viewMatchDetail('${match.matchId}')">View</button>
      </td>
    </tr>
  `).join('');
}

// Point records view
async function loadPointRecords() {
  try {
    const api = getAPI();
    const pointRecords = await api.points.getAll(50, 0);
    state.pointRecords = pointRecords;
    renderPointRecordsList();
  } catch (error) {
    console.error('Failed to load point records:', error);
    showToast('Failed to load point history', 'error');
  }
}

function renderPointRecordsList() {
  const emptyEl = document.getElementById('points-empty');
  const containerEl = document.getElementById('points-container');
  const listEl = document.getElementById('points-list');
  const statsEl = document.getElementById('points-stats');
  
  if (!emptyEl || !containerEl || !listEl) return;
  
  if (state.pointRecords.length === 0) {
    emptyEl.classList.remove('hidden');
    containerEl.classList.add('hidden');
    return;
  }
  
  emptyEl.classList.add('hidden');
  containerEl.classList.remove('hidden');
  
  // Calculate stats
  const totalAmount = state.pointRecords.reduce((sum, record) => sum + record.points, 0);
  
  if (statsEl) {
    statsEl.innerHTML = `
      <div class="stat-item">
        <div class="stat-value">${state.pointRecords.length}</div>
        <div class="stat-label">Total Records</div>
      </div>
      <div class="stat-item">
        <div class="stat-value">${formatPoints(totalAmount)}</div>
        <div class="stat-label">Total Points</div>
      </div>
    `;
  }
  
  listEl.innerHTML = state.pointRecords.map(pointRecord => `
    <tr>
      <td>${truncateMatchId(pointRecord.matchId)}</td>
      <td>${truncateMatchId(pointRecord.matchId)}</td>
      <td>${pointRecord.ruleNameSnapshot}</td>
      <td class="text-success">${formatPoints(pointRecord.points)}</td>
      <td>${formatDateTime(pointRecord.createdAt)}</td>
    </tr>
  `).join('');
}

// Form handlers
async function handleTeammateSubmit(e: Event) {
  e.preventDefault();
  
  const form = e.target as HTMLFormElement;
  const idInput = document.getElementById('teammate-id') as HTMLInputElement;
  const nameInput = document.getElementById('teammate-name') as HTMLInputElement;
  const platformInput = document.getElementById('teammate-platform') as HTMLSelectElement;
  const nicknameInput = document.getElementById('teammate-nickname') as HTMLInputElement;
  const enabledInput = document.getElementById('teammate-enabled') as HTMLInputElement;
  const isFirstTeammate = state.teammates.length === 0 && !idInput.value;
  
  try {
    const api = getAPI();
    
    if (idInput.value) {
      // Update existing
      await api.teammates.update({
        id: parseInt(idInput.value),
        displayNickname: nicknameInput.value || null,
        isPointsEnabled: enabledInput.checked,
      });
      showToast('昵称保存成功');
    } else {
      // Create new
      if (!isPlatformValue(platformInput.value)) {
        throw new Error('Invalid platform');
      }

      await api.teammates.create({
        platform: platformInput.value,
        pubgAccountId: null,
        pubgPlayerName: nameInput.value,
        displayNickname: nicknameInput.value || null,
        isPointsEnabled: enabledInput.checked,
      });

      if (isFirstTeammate) {
        const syncResult = await api.sync.start();
        if (!syncResult.success) {
          showToast(syncResult.error || '首次添加后自动同步失败', 'warning');
        } else {
          showToast('好友已添加，并自动同步好友列表');
        }
      } else {
        showToast('好友添加成功');
      }
    }
    
    closeAllModals();
    form.reset();
    await Promise.all([
      loadTeammates(),
      loadDashboard(),
      ...(isFirstTeammate ? [loadMatches(), loadPointRecords()] : []),
    ]);
  } catch (error) {
    console.error('Failed to save teammate:', error);
    showToast('保存好友失败', 'error');
  }
}

async function handleRuleSubmit(e: Event) {
  e.preventDefault();
  
  const form = e.target as HTMLFormElement;
  const idInput = document.getElementById('rule-id') as HTMLInputElement;
  const nameInput = document.getElementById('rule-name') as HTMLInputElement;
  const damageInput = document.getElementById('rule-damage') as HTMLInputElement;
  const killInput = document.getElementById('rule-kill') as HTMLInputElement;
  const reviveInput = document.getElementById('rule-revive') as HTMLInputElement;
  const roundingInput = document.getElementById('rule-rounding') as HTMLSelectElement;
  
  try {
    const api = getAPI();
    const roundingMode = isRoundingValue(roundingInput.value) ? roundingInput.value : 'round';
    
    if (idInput.value) {
      // Update existing
      await api.rules.update({
        id: parseInt(idInput.value),
        name: nameInput.value,
        damagePointsPerDamage: parseInt(damageInput.value) || 0,
        killPoints: parseInt(killInput.value) || 0,
        revivePoints: parseInt(reviveInput.value) || 0,
        roundingMode,
      });
      showToast('Rule updated successfully');
    } else {
      // Create new
      await api.rules.create({
        name: nameInput.value,
        damagePointsPerDamage: parseInt(damageInput.value) || 0,
        killPoints: parseInt(killInput.value) || 0,
        revivePoints: parseInt(reviveInput.value) || 0,
        roundingMode,
      });
      showToast('Rule created successfully');
    }
    
    closeAllModals();
    form.reset();
    loadRules();
    loadDashboard();
  } catch (error) {
    console.error('Failed to save rule:', error);
    showToast('Failed to save rule', 'error');
  }
}

async function handleSyncSubmit(e: Event) {
  e.preventDefault();
  
  const matchIdInput = document.getElementById('sync-match-id') as HTMLInputElement;
  const platformInput = document.getElementById('sync-platform') as HTMLSelectElement;
  const submitBtn = (e.target as HTMLFormElement).querySelector('button[type="submit"]') as HTMLButtonElement | null;
  const btnText = submitBtn?.querySelector('.btn-text');
  const btnSpinner = submitBtn?.querySelector('.btn-spinner');
  
  try {
    // Show loading state
    if (submitBtn) submitBtn.disabled = true;
    if (btnText) btnText.classList.add('hidden');
    if (btnSpinner) btnSpinner.classList.remove('hidden');
    
    const api = getAPI();
    const result = await api.sync.startMatch(matchIdInput.value, platformInput.value);
    
    if (result.success) {
      showToast('Match synced successfully');
      closeAllModals();
      (e.target as HTMLFormElement).reset();
      loadMatches();
      loadPointRecords();
      loadDashboard();
    } else {
      showToast(result.error || 'Failed to sync match', 'error');
    }
  } catch (error) {
    console.error('Failed to sync match:', error);
    showToast('Failed to sync match', 'error');
  } finally {
    // Reset loading state
    if (submitBtn) submitBtn.disabled = false;
    if (btnText) btnText.classList.remove('hidden');
    if (btnSpinner) btnSpinner.classList.add('hidden');
  }
}

async function handlePollingSettingsSubmit(e: Event) {
  e.preventDefault();

  const form = e.target as HTMLFormElement;
  const submitBtn = form.querySelector('button[type="submit"]') as HTMLButtonElement | null;

  const autoEnabled = document.getElementById('setting-auto-recent-match-enabled') as HTMLInputElement | null;
  const runningProcessCheck = document.getElementById('setting-running-process-check-interval-seconds') as HTMLInputElement | null;
  const notRunningProcessCheck = document.getElementById('setting-not-running-process-check-interval-seconds') as HTMLInputElement | null;
  const runningRecentMatch = document.getElementById('setting-running-recent-match-interval-seconds') as HTMLInputElement | null;
  const cooldownPolling = document.getElementById('setting-cooldown-polling-interval-seconds') as HTMLInputElement | null;
  const cooldownWindow = document.getElementById('setting-cooldown-window-minutes') as HTMLInputElement | null;
  const retryDelay = document.getElementById('setting-recent-match-retry-delay-seconds') as HTMLInputElement | null;
  const retryLimit = document.getElementById('setting-recent-match-retry-limit') as HTMLInputElement | null;

  if (!autoEnabled || !runningProcessCheck || !notRunningProcessCheck || !runningRecentMatch || !cooldownPolling || !cooldownWindow || !retryDelay || !retryLimit) {
    showToast('Polling settings form is not available.', 'error');
    return;
  }

  try {
    if (submitBtn) submitBtn.disabled = true;

    const api = getAPI();
    const entries: Array<[string, string]> = [
      [POLLING_SETTING_KEYS.autoRecentMatchEnabled, autoEnabled.checked ? '1' : '0'],
      [POLLING_SETTING_KEYS.runningProcessCheckIntervalSeconds, runningProcessCheck.value],
      [POLLING_SETTING_KEYS.notRunningProcessCheckIntervalSeconds, notRunningProcessCheck.value],
      [POLLING_SETTING_KEYS.runningRecentMatchIntervalSeconds, runningRecentMatch.value],
      [POLLING_SETTING_KEYS.cooldownPollingIntervalSeconds, cooldownPolling.value],
      [POLLING_SETTING_KEYS.cooldownWindowMinutes, cooldownWindow.value],
      [POLLING_SETTING_KEYS.recentMatchRetryDelaySeconds, retryDelay.value],
      [POLLING_SETTING_KEYS.recentMatchRetryLimit, retryLimit.value],
    ];

    await Promise.all(entries.map(([key, value]) => api.settings.set(key, value)));

    await loadPollingSettings();
    showToast('Polling settings saved successfully');
  } catch (error) {
    console.error('Failed to save polling settings:', error);
    showToast('Failed to save polling settings', 'error');
  } finally {
    if (submitBtn) submitBtn.disabled = false;
  }
}

async function handleImmediateRecentMatchCheck() {
  const syncNowButton = document.getElementById('btn-sync-now') as HTMLButtonElement | null;
  if (state.syncStatus?.isSyncing) return;

  try {
    if (syncNowButton) syncNowButton.disabled = true;

    const api = getAPI();
    const result = await api.sync.start();

    if (!result.success) {
      showToast(result.error || 'Failed to check recent match', 'error');
      return;
    }

    showToast('Recent match check completed');
    await Promise.all([loadDashboard(), loadMatches(), loadPointRecords()]);
  } catch (error) {
    console.error('Failed to check recent match:', error);
    showToast('Failed to check recent match', 'error');
  } finally {
    await loadAppStatus();
    setSyncNowButtonState();
  }
}

// Global functions for inline handlers
window.editTeammateNickname = async (id: number) => {
  try {
    const api = getAPI();
    const teammate = await api.teammates.getById(id);
    if (!teammate) return;
    
    const idInput = document.getElementById('teammate-id') as HTMLInputElement | null;
    
    if (idInput) idInput.value = teammate.id.toString();
    updateTeammateModalMode('nickname', teammate);
    openModal('modal-teammate');
  } catch (error) {
    console.error('Failed to load teammate:', error);
    showToast('加载好友信息失败', 'error');
  }
};

window.editRule = async (id: number) => {
  try {
    const rule = state.rules.find(r => r.id === id);
    if (!rule) return;
    
    const idInput = document.getElementById('rule-id') as HTMLInputElement;
    const nameInput = document.getElementById('rule-name') as HTMLInputElement;
    const damageInput = document.getElementById('rule-damage') as HTMLInputElement;
    const killInput = document.getElementById('rule-kill') as HTMLInputElement;
    const reviveInput = document.getElementById('rule-revive') as HTMLInputElement;
    const roundingInput = document.getElementById('rule-rounding') as HTMLSelectElement;
    const titleEl = document.getElementById('rule-modal-title');
    
    if (idInput) idInput.value = rule.id.toString();
    if (nameInput) nameInput.value = rule.name;
    if (damageInput) damageInput.value = rule.damagePointsPerDamage.toString();
    if (killInput) killInput.value = rule.killPoints.toString();
    if (reviveInput) reviveInput.value = rule.revivePoints.toString();
    if (roundingInput) roundingInput.value = rule.roundingMode;
    if (titleEl) titleEl.textContent = 'Edit Rule';
    
    openModal('modal-rule');
  } catch (error) {
    console.error('Failed to load rule:', error);
    showToast('Failed to load rule', 'error');
  }
};

window.activateRule = async (id: number) => {
  try {
    const api = getAPI();
    await api.rules.activate(id);
    showToast('Rule activated');
    loadRules();
    loadDashboard();
  } catch (error) {
    console.error('Failed to activate rule:', error);
    showToast('Failed to activate rule', 'error');
  }
};

window.deleteRule = async (id: number) => {
  if (!confirm('Are you sure you want to delete this rule?')) return;
  
  try {
    const api = getAPI();
    await api.rules.delete(id);
    showToast('Rule deleted');
    loadRules();
    loadDashboard();
  } catch (error) {
    console.error('Failed to delete rule:', error);
    showToast('Failed to delete rule', 'error');
  }
};

window.viewMatchDetail = async (matchId: string) => {
  try {
    const api = getAPI();
    const [match, players] = await Promise.all([
      api.matches.getById(matchId),
      api.matches.getPlayers(matchId),
    ]);
    
    if (!match) return;
    
    const contentEl = document.getElementById('match-detail-content');
    if (!contentEl) return;
    
    contentEl.innerHTML = `
      <div class="status-grid" style="margin-bottom: 1.5rem;">
        <div class="status-item">
          <span class="status-label">Match ID</span>
          <span class="status-value">${match.matchId}</span>
        </div>
        <div class="status-item">
          <span class="status-label">Map</span>
          <span class="status-value">${match.mapName || 'Unknown'}</span>
        </div>
        <div class="status-item">
          <span class="status-label">Mode</span>
          <span class="status-value">${match.gameMode || 'Unknown'}</span>
        </div>
        <div class="status-item">
          <span class="status-label">Status</span>
          <span class="status-value"><span class="status-badge status-${match.status}">${match.status}</span></span>
        </div>
      </div>
      
      ${players.length > 0 ? `
        <h4 style="margin-bottom: 1rem;">Players</h4>
        <div class="table-wrapper">
          <table class="data-table">
            <thead>
              <tr>
                <th>Player</th>
                <th>Damage</th>
                <th>Kills</th>
                <th>Revives</th>
                 <th>Points</th>
              </tr>
            </thead>
            <tbody>
              ${players.map(p => `
                <tr>
                  <td>${p.displayNicknameSnapshot || p.pubgPlayerName}</td>
                  <td>${p.damage.toLocaleString()}</td>
                  <td>${p.kills}</td>
                  <td>${p.revives}</td>
                  <td class="text-success">${formatPoints(p.points)}</td>
                </tr>
              `).join('')}
            </tbody>
          </table>
        </div>
      ` : '<p class="text-muted">No player data available</p>'}
    `;
    
    openModal('modal-match-detail');
  } catch (error) {
    console.error('Failed to load match details:', error);
    showToast('Failed to load match details', 'error');
  }
};

// Event listeners
document.addEventListener('DOMContentLoaded', async () => {
  try {
    // Initial data load
    await loadAppStatus();
    
    // Hide loading screen after a brief delay
    setTimeout(() => {
      hideLoadingScreen();
      navigateTo('dashboard');
    }, 1000);
    
    // Navigation
    document.querySelectorAll('.nav-item').forEach(item => {
      item.addEventListener('click', (e) => {
        e.preventDefault();
        const viewId = item.getAttribute('data-view');
        if (viewId) navigateTo(viewId);
      });
    });
    
    // Modal close buttons
    document.querySelectorAll('[data-close-modal]').forEach(btn => {
      btn.addEventListener('click', closeAllModals);
    });
    
    // Modal overlay click
    document.getElementById('modal-overlay')?.addEventListener('click', (e) => {
      if (e.target === e.currentTarget) {
        closeAllModals();
      }
    });
    
    // Quick action buttons
    document.getElementById('btn-sync-now')?.addEventListener('click', () => {
      void handleImmediateRecentMatchCheck();
    });
    
    document.getElementById('btn-add-teammate')?.addEventListener('click', () => {
      void handleManualTeammateSync();
    });
    
    document.getElementById('btn-new-teammate')?.addEventListener('click', () => {
      void handleManualTeammateSync();
    });
    
    document.getElementById('btn-empty-add-teammate')?.addEventListener('click', () => {
      openCreateTeammateModal();
    });
    
    document.getElementById('btn-new-rule')?.addEventListener('click', () => {
      // Reset form
      const form = document.getElementById('rule-form') as HTMLFormElement;
      form?.reset();
      const idInput = document.getElementById('rule-id') as HTMLInputElement;
      const titleEl = document.getElementById('rule-modal-title');
      
      if (idInput) idInput.value = '';
      if (titleEl) titleEl.textContent = 'Create Rule';
      
      openModal('modal-rule');
    });
    
    document.getElementById('btn-empty-create-rule')?.addEventListener('click', () => {
      document.getElementById('btn-new-rule')?.click();
    });
    
    document.getElementById('btn-empty-sync-matches')?.addEventListener('click', () => {
      openModal('modal-sync');
    });
    
    // Form submissions
    document.getElementById('teammate-form')?.addEventListener('submit', handleTeammateSubmit);
    document.getElementById('rule-form')?.addEventListener('submit', handleRuleSubmit);
    document.getElementById('sync-form')?.addEventListener('submit', handleSyncSubmit);
    document.getElementById('polling-settings-form')?.addEventListener('submit', handlePollingSettingsSubmit);
    
  } catch (error) {
    console.error('Failed to initialize app:', error);
    showErrorScreen('Failed to initialize the application. Please try again.');
  }
});

console.log('PUBG Point Rankings - Renderer App Loaded');
