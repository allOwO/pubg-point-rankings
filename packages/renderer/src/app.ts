import type { OverwolfStatus } from '@pubg-point-rankings/shared';
import type { ElectronAPI } from './preload/types';

/**
 * PUBG Point Rankings - Renderer Application
 * Main entry point for the renderer process
 */

// Type definitions for the window API
declare global {
  interface Window {
    electronAPI?: ElectronAPI;
    editTeammate?: (id: number) => Promise<void>;
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
  isRedbagEnabled: boolean;
  totalRedbagCents: number;
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
  isRedbagEnabledSnapshot: boolean;
  redbagCents: number;
  createdAt: Date;
}

interface RedbagRule {
  id: number;
  name: string;
  damageCentPerPoint: number;
  killCent: number;
  reviveCent: number;
  isActive: boolean;
  roundingMode: 'floor' | 'round' | 'ceil';
  createdAt: Date;
  updatedAt: Date;
}

interface RedbagRecord {
  id: number;
  matchId: string;
  matchPlayerId: number;
  teammateId: number | null;
  ruleId: number;
  ruleNameSnapshot: string;
  damageCentPerPointSnapshot: number;
  killCentSnapshot: number;
  reviveCentSnapshot: number;
  roundingModeSnapshot: 'floor' | 'round' | 'ceil';
  amountCents: number;
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

interface CalculatedRedbag {
  pubgAccountId: string;
  pubgPlayerName: string;
  damage: number;
  kills: number;
  revives: number;
  damageCents: number;
  killsCents: number;
  revivesCents: number;
  totalCents: number;
  isRedbagEnabled: boolean;
}

interface CreateTeammateInput {
  platform: 'steam' | 'xbox' | 'psn' | 'kakao';
  pubgAccountId: string | null;
  pubgPlayerName: string;
  displayNickname: string | null;
  isRedbagEnabled?: boolean;
}

interface UpdateTeammateInput {
  id: number;
  displayNickname?: string | null;
  isRedbagEnabled?: boolean;
}

interface CreateRedbagRuleInput {
  name: string;
  damageCentPerPoint: number;
  killCent: number;
  reviveCent: number;
  roundingMode?: 'floor' | 'round' | 'ceil';
}

interface UpdateRedbagRuleInput {
  id: number;
  name?: string;
  damageCentPerPoint?: number;
  killCent?: number;
  reviveCent?: number;
  roundingMode?: 'floor' | 'round' | 'ceil';
}

type PlatformValue = CreateTeammateInput['platform'];
type RoundingValue = NonNullable<CreateRedbagRuleInput['roundingMode']>;

function isPlatformValue(value: string): value is PlatformValue {
  return ['steam', 'xbox', 'psn', 'kakao'].includes(value);
}

function isRoundingValue(value: string): value is RoundingValue {
  return ['floor', 'round', 'ceil'].includes(value);
}

// State management
export class AppState {
  teammates: Teammate[] = [];
  rules: RedbagRule[] = [];
  activeRule: RedbagRule | null = null;
  matches: Match[] = [];
  redbags: RedbagRecord[] = [];
  syncStatus: SyncStatus | null = null;
  appStatus: AppStatus | null = null;
  overwolfStatus: OverwolfStatus | null = null;
  isLoading = false;
}

export const state = new AppState();

// API helper
export function getAPI() {
  if (!window.electronAPI) {
    throw new Error('electronAPI is not available');
  }
  return window.electronAPI;
}

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
  if (overlay) {
    overlay.classList.add('hidden');
  }
  modals.forEach(modal => {
    modal.classList.add('hidden');
  });
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
    case 'redbags':
      loadRedbags();
      break;
  }
}

// Data loading functions
async function loadAppStatus() {
  try {
    const api = getAPI();
    state.appStatus = await api.app.getStatus();
    state.overwolfStatus = await api.overwolf.getStatus();
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
    state.appStatus = await api.app.getStatus();
    state.overwolfStatus = await api.overwolf.getStatus();
    state.syncStatus = state.appStatus.syncStatus;
    
    // Update UI
    updateDashboardStatus();
    updateSyncIndicator();
    
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
    
    // Load total redbags
    const redbags = await api.redbags.getAll(1, 0);
    const totalRedbagsEl = document.getElementById('total-redbags');
    if (totalRedbagsEl) {
      totalRedbagsEl.textContent = redbags.length.toString();
    }
  } catch (error) {
    console.error('Failed to load dashboard:', error);
    showToast('Failed to load dashboard data', 'error');
  }
}

function updateDashboardStatus() {
  if (!state.appStatus) return;
  
  const dbStatus = document.getElementById('db-status');
  const overwolfStatus = document.getElementById('overwolf-status');
  const pubgStatus = document.getElementById('pubg-status');
  const lastSync = document.getElementById('last-sync');
  const systemBadge = document.getElementById('system-status-badge');
  
  if (dbStatus) {
    dbStatus.textContent = state.appStatus.isDatabaseReady ? 'Connected' : 'Error';
    dbStatus.className = 'status-value ' + (state.appStatus.isDatabaseReady ? 'text-success' : 'text-error');
  }
  
  if (lastSync) {
    lastSync.textContent = formatDateTime(state.syncStatus?.lastSyncAt ?? null);
  }

  if (overwolfStatus) {
    const isReady = Boolean(state.overwolfStatus?.isRunning && state.overwolfStatus?.isGEPAvailable);
    overwolfStatus.textContent = isReady ? 'Connected' : 'Unavailable';
    overwolfStatus.className = 'status-value ' + (isReady ? 'text-success' : 'text-warning');
  }

  if (pubgStatus) {
    const isPubgRunning = Boolean(state.overwolfStatus?.isPUBGRunning);
    pubgStatus.textContent = isPubgRunning ? 'Detected' : 'Idle';
    pubgStatus.className = 'status-value ' + (isPubgRunning ? 'text-success' : 'text-muted');
  }

  if (systemBadge) {
    if (state.appStatus.isDatabaseReady && !state.syncStatus?.error && !state.overwolfStatus?.lastError) {
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
    .filter(t => t.isRedbagEnabled)
    .sort((a, b) => b.totalRedbagCents - a.totalRedbagCents)
    .slice(0, 4);
  
  if (enabledTeammates.length === 0) {
    emptyEl.classList.remove('hidden');
    listEl.classList.add('hidden');
    return;
  }
  
  emptyEl.classList.add('hidden');
  listEl.classList.remove('hidden');
  
  listEl.innerHTML = enabledTeammates.map(teammate => `
    <div class="teammate-card ${teammate.isRedbagEnabled ? 'enabled' : 'disabled'}">
      <div class="card-title">${teammate.displayNickname || teammate.pubgPlayerName}</div>
      <div class="card-subtitle">${teammate.pubgPlayerName}</div>
      <div class="card-stats">
        <div class="card-stat">
          <div class="card-stat-value">${formatPoints(teammate.totalRedbagCents)}</div>
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
    <div class="teammate-card ${teammate.isRedbagEnabled ? 'enabled' : 'disabled'}">
      <div class="card-title">${teammate.displayNickname || teammate.pubgPlayerName}</div>
      <div class="card-subtitle">
        <span class="platform-badge">${teammate.platform}</span>
        ${teammate.pubgPlayerName}
      </div>
      <div class="card-stats">
        <div class="card-stat">
          <div class="card-stat-value">${formatPoints(teammate.totalRedbagCents)}</div>
          <div class="card-stat-label">Total Points</div>
        </div>
        <div class="card-stat">
          <div class="card-stat-value">${teammate.lastSeenAt ? formatDate(teammate.lastSeenAt) : 'Never'}</div>
          <div class="card-stat-label">Last Seen</div>
        </div>
      </div>
      <div class="card-actions">
        <button class="btn btn-secondary" onclick="editTeammate(${teammate.id})">Edit</button>
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
          <div class="card-stat-value">${rule.damageCentPerPoint}</div>
          <div class="card-stat-label">pts/DMG</div>
        </div>
        <div class="card-stat">
          <div class="card-stat-value">${rule.killCent}</div>
          <div class="card-stat-label">pts/Kill</div>
        </div>
        <div class="card-stat">
          <div class="card-stat-value">${rule.reviveCent}</div>
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

// Redbags view
async function loadRedbags() {
  try {
    const api = getAPI();
    const redbags = await api.redbags.getAll(50, 0);
    state.redbags = redbags;
    renderRedbagsList();
  } catch (error) {
    console.error('Failed to load redbags:', error);
    showToast('Failed to load point history', 'error');
  }
}

function renderRedbagsList() {
  const emptyEl = document.getElementById('redbags-empty');
  const containerEl = document.getElementById('redbags-container');
  const listEl = document.getElementById('redbags-list');
  const statsEl = document.getElementById('redbags-stats');
  
  if (!emptyEl || !containerEl || !listEl) return;
  
  if (state.redbags.length === 0) {
    emptyEl.classList.remove('hidden');
    containerEl.classList.add('hidden');
    return;
  }
  
  emptyEl.classList.add('hidden');
  containerEl.classList.remove('hidden');
  
  // Calculate stats
  const totalAmount = state.redbags.reduce((sum, r) => sum + r.amountCents, 0);
  
  if (statsEl) {
    statsEl.innerHTML = `
      <div class="stat-item">
        <div class="stat-value">${state.redbags.length}</div>
        <div class="stat-label">Total Records</div>
      </div>
      <div class="stat-item">
        <div class="stat-value">${formatPoints(totalAmount)}</div>
        <div class="stat-label">Total Points</div>
      </div>
    `;
  }
  
  listEl.innerHTML = state.redbags.map(redbag => `
    <tr>
      <td>${truncateMatchId(redbag.matchId)}</td>
      <td>${truncateMatchId(redbag.matchId)}</td>
      <td>${redbag.ruleNameSnapshot}</td>
      <td class="text-success">${formatPoints(redbag.amountCents)}</td>
      <td>${formatDateTime(redbag.createdAt)}</td>
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
  
  try {
    const api = getAPI();
    
    if (idInput.value) {
      // Update existing
      await api.teammates.update({
        id: parseInt(idInput.value),
        displayNickname: nicknameInput.value || null,
        isRedbagEnabled: enabledInput.checked,
      });
      showToast('Teammate updated successfully');
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
        isRedbagEnabled: enabledInput.checked,
      });
      showToast('Teammate added successfully');
    }
    
    closeAllModals();
    form.reset();
    loadTeammates();
    loadDashboard();
  } catch (error) {
    console.error('Failed to save teammate:', error);
    showToast('Failed to save teammate', 'error');
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
        damageCentPerPoint: parseInt(damageInput.value) || 0,
        killCent: parseInt(killInput.value) || 0,
        reviveCent: parseInt(reviveInput.value) || 0,
        roundingMode,
      });
      showToast('Rule updated successfully');
    } else {
      // Create new
      await api.rules.create({
        name: nameInput.value,
        damageCentPerPoint: parseInt(damageInput.value) || 0,
        killCent: parseInt(killInput.value) || 0,
        reviveCent: parseInt(reviveInput.value) || 0,
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
      loadRedbags();
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

// Global functions for inline handlers
window.editTeammate = async (id: number) => {
  try {
    const api = getAPI();
    const teammate = await api.teammates.getById(id);
    if (!teammate) return;
    
    const idInput = document.getElementById('teammate-id') as HTMLInputElement;
    const nameInput = document.getElementById('teammate-name') as HTMLInputElement;
    const nicknameInput = document.getElementById('teammate-nickname') as HTMLInputElement;
    const enabledInput = document.getElementById('teammate-enabled') as HTMLInputElement;
    const titleEl = document.getElementById('teammate-modal-title');
    
    if (idInput) idInput.value = teammate.id.toString();
    if (nameInput) {
      nameInput.value = teammate.pubgPlayerName;
      nameInput.disabled = true;
    }
    const platformSelect = document.getElementById('teammate-platform') as HTMLSelectElement;
    if (platformSelect) platformSelect.disabled = true;
    if (nicknameInput) nicknameInput.value = teammate.displayNickname || '';
    if (enabledInput) enabledInput.checked = teammate.isRedbagEnabled;
    if (titleEl) titleEl.textContent = 'Edit Teammate';
    
    openModal('modal-teammate');
  } catch (error) {
    console.error('Failed to load teammate:', error);
    showToast('Failed to load teammate', 'error');
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
    if (damageInput) damageInput.value = rule.damageCentPerPoint.toString();
    if (killInput) killInput.value = rule.killCent.toString();
    if (reviveInput) reviveInput.value = rule.reviveCent.toString();
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
                  <td class="text-success">${formatPoints(p.redbagCents)}</td>
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
    // Check API availability
    if (!window.electronAPI) {
      showErrorScreen('Electron API is not available. Make sure the preload script is loaded correctly.');
      return;
    }
    
    // Initial data load
    await loadAppStatus();

    const unsubscribeOverwolfStatus = window.electronAPI.overwolf.onStatusChange((status) => {
      state.overwolfStatus = status;
      updateDashboardStatus();
    });

    window.addEventListener('beforeunload', () => {
      unsubscribeOverwolfStatus();
    });
    
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
      openModal('modal-sync');
    });
    
    document.getElementById('btn-add-teammate')?.addEventListener('click', () => {
      // Reset form
      const form = document.getElementById('teammate-form') as HTMLFormElement;
      form?.reset();
      const idInput = document.getElementById('teammate-id') as HTMLInputElement;
      const nameInput = document.getElementById('teammate-name') as HTMLInputElement;
      const platformSelect = document.getElementById('teammate-platform') as HTMLSelectElement;
      const titleEl = document.getElementById('teammate-modal-title');
      
      if (idInput) idInput.value = '';
      if (nameInput) nameInput.disabled = false;
      if (platformSelect) platformSelect.disabled = false;
      if (titleEl) titleEl.textContent = 'Add Teammate';
      
      openModal('modal-teammate');
    });
    
    document.getElementById('btn-new-teammate')?.addEventListener('click', () => {
      document.getElementById('btn-add-teammate')?.click();
    });
    
    document.getElementById('btn-empty-add-teammate')?.addEventListener('click', () => {
      document.getElementById('btn-add-teammate')?.click();
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
    
  } catch (error) {
    console.error('Failed to initialize app:', error);
    showErrorScreen('Failed to initialize the application. Please try again.');
  }
});

console.log('PUBG Point Rankings - Renderer App Loaded');
