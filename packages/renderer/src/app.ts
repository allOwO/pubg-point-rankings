import { getAPI, getRuntimeHost, type Account, type MatchDetail, type PointHistoryListItem, type UnsettledBattleSummary } from './tauri-api';
import {
  APP_LANGUAGE_SETTING_KEY,
  DEFAULT_LOCALE,
  normalizeLocale,
  translate,
  type Locale,
  type TranslationKey,
} from './i18n';
import { formatActivitySentenceParts } from './match-log-activity';
import { getMatchListBattleDelta, getMatchListPlacement, getNonZeroMatchBattleDeltas } from './matches-list';
import { createNotificationsPageController, type NotificationsPageController } from './notifications-page';

/**
 * PUBG Point Rankings - Renderer Application
 * Main entry point for the renderer process
 */

declare global {
  interface Window {
    editTeammateNickname?: (id: number) => Promise<void>;
    deleteTeammate?: (id: number) => Promise<void>;
    editRule?: (id: number) => Promise<void>;
    activateRule?: (id: number) => Promise<void>;
    deleteRule?: (id: number) => Promise<void>;
  }
}

// Type definitions
interface Teammate {
  id: number;
  platform: 'steam' | 'xbox' | 'psn' | 'kakao';
  pubgAccountId: string | null;
  pubgPlayerName: string;
  displayNickname: string | null;
  isFriend: boolean;
  isPointsEnabled: boolean;
  totalPoints: number;
  lastSeenAt: Date | null;
  createdAt: Date;
  updatedAt: Date;
}

interface RecentTeammateCandidate {
  platform: 'steam' | 'xbox' | 'psn' | 'kakao';
  pubgAccountId: string | null;
  pubgPlayerName: string;
  lastTeammateAt: Date;
  isFriend: boolean;
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
  assists: number;
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

interface MatchWithPlayers {
  match: Match;
  players: MatchPlayer[];
}

interface DashboardRecentMatchPlayerRow {
  matchPlayerId: number;
  displayName: string;
  kills: number;
  damage: number;
  assists: number;
  revives: number;
  isSelf: boolean;
  points: number;
  isPointsEnabled: boolean;
}

interface DashboardRecentMatchRow {
  matchId: string;
  mapName: string;
  gameMode: string;
  playedAt: Date;
  status: Match['status'];
  selfPlayer: DashboardRecentMatchPlayerRow;
  squad: DashboardRecentMatchPlayerRow[];
  battleDeltas: Array<{
    matchPlayerId: number;
    displayName: string;
    delta: number;
  }>;
}

interface LanguageOption {
  value: Locale;
  labelKey: TranslationKey;
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

const LANGUAGE_OPTIONS: LanguageOption[] = [
  { value: 'en-US', labelKey: 'settings.languageEnglish' },
  { value: 'zh-CN', labelKey: 'settings.languageChinese' },
];

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

function getTeammateLastPlayedMeta(teammate: Teammate): { label: string; sortTime: number } {
  if (!teammate.lastSeenAt) {
    return {
      label: t('friends.lastPlayedNever'),
      sortTime: Number.NEGATIVE_INFINITY,
    };
  }

  return {
    label: t('friends.lastTeammateAt', { date: formatDateTime(teammate.lastSeenAt) }),
    sortTime: new Date(teammate.lastSeenAt).getTime(),
  };
}

function isRecentlyActiveTeammate(teammate: Teammate): boolean {
  if (!teammate.lastSeenAt) {
    return false;
  }

  const thirtyDaysMs = 30 * 24 * 60 * 60 * 1000;
  return Date.now() - new Date(teammate.lastSeenAt).getTime() < thirtyDaysMs;
}

function updateTeammateModalMode(teammate?: Teammate) {
  const titleEl = document.getElementById('teammate-modal-title');
  const nicknameInput = document.getElementById('teammate-nickname') as HTMLInputElement | null;

  if (!teammate) return;

  if (titleEl) {
    titleEl.textContent = teammate.displayNickname ? t('friends.editNickname') : t('friends.addNickname');
  }
  if (nicknameInput) nicknameInput.value = teammate.displayNickname || '';
}

async function openRecentTeammatesModal() {
  const triggerButtons = [
    document.getElementById('btn-new-teammate') as HTMLButtonElement | null,
    document.getElementById('btn-sync-friends-manual') as HTMLButtonElement | null,
  ];
  const listEl = document.getElementById('recent-teammates-list');
  const emptyEl = document.getElementById('recent-teammates-empty');

  if (state.syncStatus?.isSyncing) return;

  try {
    triggerButtons.forEach((button) => {
      if (button) button.disabled = true;
    });

    if (listEl) {
      listEl.innerHTML = '';
      listEl.classList.add('hidden');
    }
    if (emptyEl) {
      emptyEl.classList.remove('hidden');
      emptyEl.textContent = t('friends.loadingRecentTeammates');
    }

    const api = getAPI();
    const candidates = await api.teammates.getRecentCandidates();
    state.recentTeammateCandidates = candidates;

    if (!listEl || !emptyEl) {
      return;
    }

    if (candidates.length === 0) {
      emptyEl.classList.remove('hidden');
      emptyEl.textContent = t('friends.noRecentTeammates');
      openModal('modal-recent-teammates');
      return;
    }

    listEl.innerHTML = candidates.map((candidate, index) => `
      <div class="recent-teammate-row">
        <div class="recent-teammate-name">${escapeHtml(candidate.pubgPlayerName)}</div>
        <button
          type="button"
          class="btn btn-primary"
          data-recent-teammate-index="${index}"
        >
          ${t('friends.addRecentTeammate')}
        </button>
      </div>
    `).join('');
    emptyEl.classList.add('hidden');
    listEl.classList.remove('hidden');
    openModal('modal-recent-teammates');
  } catch (error) {
    console.error('Failed to load recent teammates:', error);
    showToast(t('toast.recentTeammatesLoadFailed'), 'error');
  } finally {
    triggerButtons.forEach((button) => {
      if (button) button.disabled = false;
    });
  }
}

function openManualFriendModal() {
  const form = document.getElementById('manual-friend-form') as HTMLFormElement | null;
  form?.reset();
  openModal('modal-manual-friend');
}

async function loadSettings() {
  try {
    const api = getAPI();
    const activeAccount = await api.accounts.getActive();
    
    const nameInput = document.getElementById('account-name') as HTMLInputElement | null;
    const playerNameInput = document.getElementById('account-player-name') as HTMLInputElement | null;
    const platformSelect = document.getElementById('account-platform') as HTMLSelectElement | null;
    const apiKeyInput = document.getElementById('account-api-key') as HTMLInputElement | null;
    const languageSelect = document.getElementById('app-language') as HTMLSelectElement | null;

    if (languageSelect) {
      languageSelect.value = state.locale;
    }
    
    if (activeAccount) {
      if (nameInput) nameInput.value = activeAccount.accountName || '';
      if (playerNameInput) playerNameInput.value = activeAccount.selfPlayerName || '';
      if (platformSelect) platformSelect.value = activeAccount.selfPlatform;
      if (apiKeyInput) apiKeyInput.value = activeAccount.pubgApiKey || '';
    } else {
      if (nameInput) nameInput.value = '';
      if (playerNameInput) playerNameInput.value = '';
      if (platformSelect) platformSelect.value = 'steam';
      if (apiKeyInput) apiKeyInput.value = '';
    }

    // Load polling settings as part of settings
    await loadPollingSettings();
  } catch (error) {
    console.error('Failed to load settings:', error);
    showToast(t('toast.accountLoadFailed'), 'error');
  }
}

async function loadNotifications() {
  try {
    const api = getAPI();

    // Create or reuse the notifications controller
    if (!state.notificationsController) {
      state.notificationsController = createNotificationsPageController({
        getStatus: () => api.notifications.getStatus(),
        getFailedTasks: () => api.notifications.getFailedTasks(),
        sendSelected: (taskIds: number[]) => api.notifications.sendSelected(taskIds),
        deleteFailedTask: (taskId: number) => api.notifications.deleteFailedTask(taskId),
        installRuntime: () => api.notifications.installRuntime(),
        startRuntime: () => api.notifications.startRuntime(),
        stopRuntime: () => api.notifications.stopRuntime(),
        restartRuntime: () => api.notifications.restartRuntime(),
        sendTest: () => api.notifications.sendTest(),
        saveGroupId: (groupId: string) => api.notifications.saveGroupId(groupId),
        translate: (key: string) => t(key as TranslationKey),
      });
    }

    await state.notificationsController.load();
    applyStaticTranslations();
  } catch (error) {
    console.error('Failed to load notifications:', error);
    showToast(t('toast.notificationsLoadFailed'), 'error');
  }
}

async function handleLanguageSubmit(e: Event) {
  e.preventDefault();

  const form = e.target as HTMLFormElement;
  const submitButton = form.querySelector('button[type="submit"]') as HTMLButtonElement | null;
  const languageSelect = document.getElementById('app-language') as HTMLSelectElement | null;

  if (!languageSelect) {
    return;
  }

  try {
    if (submitButton) submitButton.disabled = true;

    await setLanguage(normalizeLocale(languageSelect.value));
    showToast(t('toast.languageSaved'));
  } catch (error) {
    console.error('Failed to save language preference:', error);
    showToast(t('toast.languageSaveFailed'), 'error');
  } finally {
    if (submitButton) submitButton.disabled = false;
  }
}

async function handleAccountSettingsSubmit(e: Event) {
  e.preventDefault();
  
  const nameInput = document.getElementById('account-name') as HTMLInputElement | null;
  const playerNameInput = document.getElementById('account-player-name') as HTMLInputElement | null;
  const platformSelect = document.getElementById('account-platform') as HTMLSelectElement | null;
  const submitButton = (e.target as HTMLFormElement).querySelector('button[type="submit"]') as HTMLButtonElement | null;
  
  if (!playerNameInput?.value || !platformSelect?.value) {
    showToast(t('toast.accountRequiredFields'), 'error');
    return;
  }
  
  try {
    if (submitButton) submitButton.disabled = true;
    
    const api = getAPI();
    const platform = platformSelect.value as Account['selfPlatform'];
    const activeAccount = await api.accounts.getActive();

    if (activeAccount) {
      await api.accounts.updateActive({
        accountName: nameInput?.value || activeAccount.accountName,
        selfPlayerName: playerNameInput.value,
        selfPlatform: platform,
        pubgApiKey: activeAccount.pubgApiKey,
      });
    } else {
      // Create new account with empty API key (user will add it via separate API key form)
      await api.accounts.create({
        accountName: nameInput?.value || playerNameInput.value,
        selfPlayerName: playerNameInput.value,
        selfPlatform: platform,
        pubgApiKey: '',
        setActive: true,
      });
    }
    
    await Promise.all([loadSettings(), loadAppStatus(), loadDashboard()]);
    showToast(t('toast.accountSaved'));
  } catch (error) {
    console.error('Failed to save account settings:', error);
    showToast(t('toast.accountSaveFailed'), 'error');
  } finally {
    if (submitButton) submitButton.disabled = false;
  }
}

async function handleApiKeySettingsSubmit(e: Event) {
  e.preventDefault();
  
  const apiKeyInput = document.getElementById('account-api-key') as HTMLInputElement | null;
  const submitButton = (e.target as HTMLFormElement).querySelector('button[type="submit"]') as HTMLButtonElement | null;
  
  if (!apiKeyInput?.value) {
    showToast(t('toast.accountRequiredFields'), 'error');
    return;
  }
  
  try {
    if (submitButton) submitButton.disabled = true;
    
    const api = getAPI();
    const activeAccount = await api.accounts.getActive();

    if (activeAccount) {
      await api.accounts.updateActive({
        ...activeAccount,
        pubgApiKey: apiKeyInput.value,
      });
    } else {
      showToast(t('toast.accountRequiredFields'), 'error');
      return;
    }
    
    await Promise.all([loadSettings(), loadAppStatus(), loadDashboard()]);
    showToast(t('toast.accountSaved'));
  } catch (error) {
    console.error('Failed to save API key:', error);
    showToast(t('toast.accountSaveFailed'), 'error');
  } finally {
    if (submitButton) submitButton.disabled = false;
  }
}

async function handleLogout() {
  openConfirmModal(
    t('confirm.logout.title'),
    t('confirm.logout.message'),
    t('confirm.logout.confirm'),
    async () => {
      try {
        const api = getAPI();
        await api.accounts.logout();
        showToast(t('toast.logoutSuccess'));
        // Refresh the page to reset state
        location.reload();
      } catch (error) {
        console.error('Failed to logout:', error);
        showToast(t('toast.logoutFailed'), 'error');
      }
    },
    'btn-danger',
    'danger'
  );
}

// State management
export class AppState {
  teammates: Teammate[] = [];
  recentTeammateCandidates: RecentTeammateCandidate[] = [];
  rules: PointRule[] = [];
  activeRule: PointRule | null = null;
  matches: Match[] = [];
  matchPlacements = new Map<string, number | null>();
  matchBattleDeltas = new Map<string, number | null>();
  pointRecords: PointRecord[] = [];
  pointHistory: PointHistoryListItem[] = [];
  unsettledSummary: UnsettledBattleSummary | null = null;
  selectedUnsettledRuleId: number | null = null;
  dashboardRecentMatches: DashboardRecentMatchRow[] = [];
  expandedDashboardMatchId: string | null = null;
  pendingSettleMatchId: string | null = null;
  pendingSettleTimerId: number | null = null;
  expandedMatchLogId: string | null = null;
  loadingMatchLogId: string | null = null;
  matchLogDetails = new Map<string, MatchDetail>();
  matchLogErrors = new Map<string, string>();
  syncStatus: SyncStatus | null = null;
  appStatus: AppStatus | null = null;
  gameProcessStatus: GameProcessStatus | null = null;
  pollingSettings: PollingSettings = { ...DEFAULT_POLLING_SETTINGS };
  locale: Locale = DEFAULT_LOCALE;
  hasConfiguredApiKey = false;
  isLoading = false;
  notificationsController: NotificationsPageController | null = null;
}

export const state = new AppState();

// Utility functions
function t(key: TranslationKey, params?: Record<string, string | number>): string {
  return translate(state.locale, key, params);
}

function getActiveViewId(): string {
  return document.querySelector('.view.active')?.id.replace('view-', '') || 'dashboard';
}

function translateMatchStatus(status: Match['status']): string {
  return t(`status.match.${status}` as TranslationKey);
}

function buildDashboardRecentMatchRow(detail: MatchWithPlayers): DashboardRecentMatchRow | null {
  const selfPlayer = detail.players.find((player) => player.isSelf);
  if (!selfPlayer) {
    return null;
  }

  const toRow = (player: MatchPlayer): DashboardRecentMatchPlayerRow => ({
    matchPlayerId: player.id,
    displayName: player.displayNicknameSnapshot || player.pubgPlayerName,
    kills: player.kills,
    damage: player.damage,
    assists: player.assists,
    revives: player.revives,
    isSelf: player.isSelf,
    points: player.points,
    isPointsEnabled: player.isPointsEnabledSnapshot,
  });

  const squadPlayers = detail.players
    .filter((player) => player.teamId === selfPlayer.teamId)
    .sort((left, right) => {
      if (left.isSelf !== right.isSelf) return left.isSelf ? -1 : 1;
      return right.kills - left.kills || right.damage - left.damage || right.assists - left.assists;
    });

  const squad = squadPlayers.map(toRow);
  const battleDeltas = getNonZeroMatchBattleDeltas(squad.map((player) => ({
    matchPlayerId: player.matchPlayerId,
    displayName: player.displayName,
    isPointsEnabled: player.isPointsEnabled,
    points: player.points,
  })));

  return {
    matchId: detail.match.matchId,
    mapName: detail.match.mapName || t('common.unknown'),
    gameMode: detail.match.gameMode || t('common.unknown'),
    playedAt: detail.match.matchEndAt || detail.match.playedAt,
    status: detail.match.status,
    selfPlayer: toRow(selfPlayer),
    squad,
    battleDeltas,
  };
}

function applyStaticTranslations() {
  document.documentElement.lang = state.locale;
  document.title = t('app.title');

  document.querySelectorAll<HTMLElement>('[data-i18n]').forEach((element) => {
    const key = element.dataset.i18n as TranslationKey | undefined;
    if (key) {
      element.textContent = t(key);
    }
  });

  document.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>('[data-i18n-placeholder]').forEach((element) => {
    const key = element.dataset.i18nPlaceholder as TranslationKey | undefined;
    if (key) {
      element.placeholder = t(key);
    }
  });

  const languageSelect = document.getElementById('app-language') as HTMLSelectElement | null;
  if (languageSelect) {
    languageSelect.innerHTML = LANGUAGE_OPTIONS.map((option) => {
      const selected = option.value === state.locale ? ' selected' : '';
      return `<option value="${option.value}"${selected}>${t(option.labelKey)}</option>`;
    }).join('');
  }

  const syncLabel = document.getElementById('btn-sync-now-label');
  if (syncLabel && !state.syncStatus?.isSyncing) {
    syncLabel.textContent = t('dashboard.syncNow');
  }

  const confirmCancel = document.getElementById('confirm-modal-cancel');
  if (confirmCancel) {
    confirmCancel.textContent = t('modal.cancel');
  }
}

async function loadLanguagePreference() {
  try {
    const api = getAPI();
    const languageSetting = await api.settings.get(APP_LANGUAGE_SETTING_KEY);
    state.locale = normalizeLocale(languageSetting?.value);
  } catch (error) {
    console.error('Failed to load language preference:', error);
    state.locale = DEFAULT_LOCALE;
  }

  applyStaticTranslations();
}

async function refreshLocalizedContent() {
  updateSyncIndicator();
  updateDashboardStatus();
  updateActiveRule();

  // New dashboard renderers
  renderDashboardUnsettledSummary();
  renderDashboardRecentMatches();
  
  // Keep existing renderers for other views
  renderTeammatesList();
  renderRulesList();
  renderMatchesList();
  renderPointRecordsList();

  if (getActiveViewId() === 'settings') {
    await loadSettings();
  }
}

async function setLanguage(locale: Locale) {
  const normalizedLocale = normalizeLocale(locale);
  const api = getAPI();
  await api.settings.set(APP_LANGUAGE_SETTING_KEY, normalizedLocale);
  state.locale = normalizedLocale;
  applyStaticTranslations();
  await refreshLocalizedContent();
}

async function hasValidApiKey(): Promise<boolean> {
  const api = getAPI();
  const activeAccount = await api.accounts.getActive();
  
  if (!activeAccount || !activeAccount.pubgApiKey?.trim()) {
    openConfirmModal(
      t('confirm.apiKeyMissing.title'),
      t('confirm.apiKeyMissing.message'),
      t('confirm.apiKeyMissing.confirm'),
      () => {
        navigateTo('settings');
      },
      'btn-primary',
      'warning'
    );
    return false;
  }
  
  return true;
}

function formatPoints(points: number): string {
  return `${Math.round(points).toLocaleString()} pts`;
}

function formatInteger(value: number): string {
  return Math.round(value).toLocaleString(state.locale);
}

function formatSignedInteger(value: number): string {
  const rounded = Math.round(value);
  return `${rounded > 0 ? '+' : ''}${rounded.toLocaleString(state.locale)}`;
}

function syncSelectedUnsettledRuleId() {
  const availableRuleIds = new Set(state.rules.map((rule) => rule.id));
  const preferredRuleId = state.unsettledSummary?.ruleId ?? state.activeRule?.id ?? null;

  if (preferredRuleId !== null && availableRuleIds.has(preferredRuleId)) {
    state.selectedUnsettledRuleId = preferredRuleId;
    return;
  }

  if (
    state.selectedUnsettledRuleId !== null
    && availableRuleIds.has(state.selectedUnsettledRuleId)
  ) {
    return;
  }

  state.selectedUnsettledRuleId = state.rules[0]?.id ?? null;
}

function formatDate(date: Date | string | null): string {
  if (!date) return t('common.never');
  const d = new Date(date);
  return d.toLocaleDateString(state.locale, {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}

function formatDateTime(date: Date | string | null): string {
  if (!date) return t('common.never');
  const d = new Date(date);
  return d.toLocaleString(state.locale, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function truncateMatchId(matchId: string): string {
  return matchId.slice(0, 8) + '...';
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function clearPendingSettleState() {
  if (state.pendingSettleTimerId !== null) {
    window.clearTimeout(state.pendingSettleTimerId);
  }

  state.pendingSettleTimerId = null;
  state.pendingSettleMatchId = null;
}

function getHistoryMatchGroup(matchId: string) {
  return state.pointHistory.find(
    (item): item is Extract<PointHistoryListItem, { type: 'match_group' }> =>
      item.type === 'match_group' && item.matchId === matchId,
  );
}

function getMatchLogPlayerKey(pubgAccountId: string | null, pubgPlayerName: string): string {
  const trimmedAccountId = pubgAccountId?.trim();
  if (trimmedAccountId) {
    return `account:${trimmedAccountId}`;
  }

  return `name:${pubgPlayerName.trim().toLowerCase()}`;
}

function formatMatchDuration(startAt: Date | null, endAt: Date | null): string {
  if (!startAt || !endAt) {
    return '--';
  }

  const diffMs = endAt.getTime() - startAt.getTime();
  if (!Number.isFinite(diffMs) || diffMs <= 0) {
    return '--';
  }

  const totalMinutes = Math.floor(diffMs / 60000);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

function getTeamPlayersForMatchLog(detail: MatchDetail): MatchPlayer[] {
  const selfPlayer = detail.players.find((player) => player.isSelf);
  const teamId = selfPlayer?.teamId ?? null;
  const players = teamId !== null
    ? detail.players.filter((player) => player.teamId === teamId)
    : [...detail.players];

  return [...players]
    .sort((left, right) => {
      if (left.isSelf !== right.isSelf) {
        return left.isSelf ? -1 : 1;
      }

      return right.kills - left.kills
        || right.assists - left.assists
        || right.damage - left.damage
        || (left.placement ?? Number.MAX_SAFE_INTEGER) - (right.placement ?? Number.MAX_SAFE_INTEGER);
    })
    .slice(0, 4);
}

function buildReviveSummaryMap(detail: MatchDetail): Map<string, Array<{ victimLabel: string; count: number }>> {
  const summaryMap = new Map<string, Map<string, { victimLabel: string; count: number }>>();

  detail.reviveEvents.forEach((event) => {
    const reviverKey = getMatchLogPlayerKey(event.reviverAccountId, event.reviverName || t('common.unknown'));
    const victimKey = getMatchLogPlayerKey(event.victimAccountId, event.victimName || t('common.unknown'));
    const reviverBucket = summaryMap.get(reviverKey) || new Map<string, { victimLabel: string; count: number }>();
    const current = reviverBucket.get(victimKey);
    if (current) {
      current.count += 1;
    } else {
      reviverBucket.set(victimKey, {
        victimLabel: event.victimName || t('common.unknown'),
        count: 1,
      });
    }
    summaryMap.set(reviverKey, reviverBucket);
  });

  return new Map(
    Array.from(summaryMap.entries()).map(([key, bucket]) => [
      key,
      Array.from(bucket.values()).sort((left, right) => right.count - left.count || left.victimLabel.localeCompare(right.victimLabel, state.locale)),
    ]),
  );
}

function renderMatchLogPanel(matchId: string): string {
  if (state.expandedMatchLogId !== matchId) return '';

  if (state.loadingMatchLogId === matchId && !state.matchLogDetails.has(matchId)) {
    return `<section class="match-log-panel loading"><div class="match-log-loading">${escapeHtml(t('matches.loadingLog'))}</div></section>`;
  }

  const error = state.matchLogErrors.get(matchId);
  if (error) {
    return `<section class="match-log-panel error"><div class="match-log-empty">${escapeHtml(error)}</div></section>`;
  }

  const detail = state.matchLogDetails.get(matchId);
  if (!detail) {
    return `<section class="match-log-panel empty"><div class="match-log-empty">${escapeHtml(t('matches.logUnavailable'))}</div></section>`;
  }

  const squadPlayers = getTeamPlayersForMatchLog(detail);
  const reviveMap = buildReviveSummaryMap(detail);
  const weaponsByPlayer = new Map<string, typeof detail.weaponStats>();
  detail.weaponStats.forEach((s) => {
    const k = getMatchLogPlayerKey(s.pubgAccountId, s.pubgPlayerName);
    weaponsByPlayer.set(k, [...(weaponsByPlayer.get(k) || []), s].sort((a, b) => b.totalDamage - a.totalDamage));
  });

  const formatPlayer = (p: MatchPlayer | null) => {
    if (!p) return `<article class="match-log-squad-card empty"><div class="match-log-squad-empty">${escapeHtml(t('detail.emptySlot'))}</div></article>`;
    const key = getMatchLogPlayerKey(p.pubgAccountId, p.pubgPlayerName);
    const weapons = weaponsByPlayer.get(key) || [];
    const revives = reviveMap.get(key) || [];
    const reviveHtml = p.revives > 0 ? `<details class="match-log-revive-details"><summary>${escapeHtml(t('detail.reviveDetails'))}</summary><ul>${revives.map((r) => `<li>${escapeHtml(r.victimLabel)} × ${formatInteger(r.count)}</li>`).join('')}</ul></details>` : '';
    const selfTag = p.isSelf ? `<span class="points-self-tag">${escapeHtml(t('common.you'))}</span>` : '';
    const weaponHtml = weapons.length ? `<ul class="match-log-weapon-list">${weapons.map((w) => `<li><span>${escapeHtml(w.weaponName)}</span><strong>${formatInteger(w.totalDamage)}</strong></li>`).join('')}</ul>` : `<div class="match-log-empty-hint">${escapeHtml(t('detail.noWeaponDamage'))}</div>`;
    return `
      <article class="match-log-squad-card${p.isSelf ? ' self' : ''}">
        <div class="match-log-squad-name">${escapeHtml(p.displayNicknameSnapshot || p.pubgPlayerName)}${selfTag}</div>
        <dl class="match-log-squad-stats">
          <div><dt>${escapeHtml(t('detail.kills'))}</dt><dd>${formatInteger(p.kills)}</dd></div>
          <div><dt>${escapeHtml(t('detail.damage'))}</dt><dd>${formatInteger(p.damage)}</dd></div>
          <div><dt>${escapeHtml(t('detail.assists'))}</dt><dd>${formatInteger(p.assists)}</dd></div>
          <div><dt>${escapeHtml(t('detail.revives'))}</dt><dd>${formatInteger(p.revives)}</dd></div>
        </dl>
        <div class="match-log-weapon-section"><div class="match-log-subtitle">${escapeHtml(t('detail.weaponDamage'))}</div>${weaponHtml}</div>
        ${reviveHtml}
      </article>`;
  };

  const squadCards = Array.from({ length: 4 }, (_, i) => formatPlayer(squadPlayers[i] ?? null)).join('');
  const selfPlacement = squadPlayers.find((p) => p.isSelf)?.placement ?? squadPlayers[0]?.placement ?? null;

  const teammateKeys = new Set(squadPlayers.map((p) => getMatchLogPlayerKey(p.pubgAccountId, p.pubgPlayerName)));

  interface ActivityEvent {
    type: 'knock' | 'kill' | 'revive';
    timestamp: Date | null;
    actorName: string;
    targetName: string;
    actorAccountId: string | null;
    targetAccountId: string | null;
    extra?: string;
  }

  const events: ActivityEvent[] = [
    ...detail.knockEvents.map((e) => ({
      type: 'knock' as const,
      timestamp: e.eventAt,
      actorName: e.attackerName || t('common.unknown'),
      targetName: e.victimName || t('common.unknown'),
      actorAccountId: e.attackerAccountId,
      targetAccountId: e.victimAccountId,
      extra: e.damageCauserName || e.damageTypeCategory || undefined,
    })),
    ...detail.killEvents.map((e) => ({
      type: 'kill' as const,
      timestamp: e.eventAt,
      actorName: e.killerName || t('common.unknown'),
      targetName: e.victimName || t('common.unknown'),
      actorAccountId: e.killerAccountId,
      targetAccountId: e.victimAccountId,
      extra: e.damageCauserName || e.damageTypeCategory || undefined,
    })),
    ...detail.reviveEvents.map((e) => ({
      type: 'revive' as const,
      timestamp: e.eventAt,
      actorName: e.reviverName || t('common.unknown'),
      targetName: e.victimName || t('common.unknown'),
      actorAccountId: e.reviverAccountId,
      targetAccountId: e.victimAccountId,
    })),
  ].sort((a, b) => (a.timestamp?.getTime() ?? Number.MAX_SAFE_INTEGER) - (b.timestamp?.getTime() ?? Number.MAX_SAFE_INTEGER));

  const formatActivityEvent = (e: ActivityEvent): string => {
    const actorKey = getMatchLogPlayerKey(e.actorAccountId, e.actorName);
    const targetKey = getMatchLogPlayerKey(e.targetAccountId, e.targetName);
    const actorIsTeammate = teammateKeys.has(actorKey);
    const targetIsTeammate = teammateKeys.has(targetKey);

    let highlightClass = 'match-log-activity-item';
    if ((e.type === 'kill' || e.type === 'knock') && actorIsTeammate) {
      highlightClass += ' teammate-kill';
    } else if ((e.type === 'kill' || e.type === 'knock') && targetIsTeammate) {
      highlightClass += ' teammate-death';
    }

    const timeStr = e.timestamp
      ? e.timestamp.toLocaleTimeString(state.locale, { hour: '2-digit', minute: '2-digit', second: '2-digit' })
      : '--:--:--';
    const actionText = e.type === 'knock'
      ? t('detail.activityKnocked')
      : e.type === 'kill'
        ? t('detail.activityKilled')
        : t('detail.activityRevived');
    const sentence = formatActivitySentenceParts({
      locale: state.locale,
      type: e.type,
      actionText,
      targetName: e.targetName,
      extra: e.extra ?? null,
    });

    return `
      <li class="${highlightClass}">
        <div class="match-log-activity-time">${escapeHtml(timeStr)}</div>
        <div class="match-log-activity-body">
          <strong class="${actorIsTeammate ? 'match-log-activity-player match-log-activity-actor teammate' : 'match-log-activity-player match-log-activity-actor'}">${escapeHtml(e.actorName)}</strong>
          <div class="match-log-activity-main">
            <span class="match-log-activity-sentence">${escapeHtml(sentence.beforeTarget)}<strong class="${targetIsTeammate ? 'match-log-activity-player teammate' : 'match-log-activity-player'}">${escapeHtml(e.targetName)}</strong>${escapeHtml(sentence.afterTarget)}</span>
          </div>
        </div>
      </li>
    `;
  };

  const activityFeed = events.length > 0
    ? `<ul class="match-log-activity-feed">${events.map(formatActivityEvent).join('')}</ul>`
    : `<div class="match-log-empty-hint">${escapeHtml(t('detail.noActivityLog'))}</div>`;

  const headerItem = (label: string, value: string) => `<div class="match-log-header-item"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong></div>`;
  const badge = (label: string, value: string) => `<span class="badge badge-info">${escapeHtml(label)}: ${escapeHtml(value)}</span>`;

  return `
    <section class="match-log-panel">
      <div class="match-log-header-grid">
        ${headerItem(t('detail.rank'), selfPlacement ? `#${formatInteger(selfPlacement)}` : '--')}
        ${headerItem(t('detail.endTime'), formatDateTime(detail.match.matchEndAt || detail.match.playedAt))}
        ${headerItem(t('detail.duration'), formatMatchDuration(detail.match.matchStartAt, detail.match.matchEndAt || detail.match.playedAt))}
        ${headerItem(t('detail.matchId'), truncateMatchId(detail.match.matchId))}
      </div>
      <div class="match-log-meta-row">
        ${badge(t('detail.map'), detail.match.mapName || t('common.unknown'))}
        ${badge(t('detail.mode'), detail.match.gameMode || t('common.unknown'))}
        ${badge(t('detail.status'), translateMatchStatus(detail.match.status))}
      </div>
      <div class="match-log-section">
        <div class="match-log-section-label">${escapeHtml(t('detail.teamSummary'))}</div>
        <div class="match-log-squad-grid">${squadCards}</div>
      </div>
      <div class="match-log-section">
        <details class="match-log-activity-details">
          <summary class="match-log-activity-summary">
            <span class="match-log-section-label">${escapeHtml(t('detail.activityLog'))}</span>
            <span class="match-log-activity-summary-count">${formatInteger(events.length)}</span>
          </summary>
          ${activityFeed}
        </details>
      </div>
    </section>`;
}

async function toggleMatchLog(matchId: string) {
  if (state.expandedMatchLogId === matchId) {
    state.expandedMatchLogId = null;
    renderMatchesList();
    return;
  }

  state.expandedMatchLogId = matchId;
  state.matchLogErrors.delete(matchId);
  renderMatchesList();

  if (state.matchLogDetails.has(matchId)) {
    return;
  }

  state.loadingMatchLogId = matchId;
  renderMatchesList();

  try {
    const api = getAPI();
    const detail = await api.matches.getDetail(matchId);
    if (!detail) {
      state.matchLogErrors.set(matchId, t('matches.logUnavailable'));
    } else {
      state.matchLogDetails.set(matchId, detail);
    }
  } catch (error) {
    console.error('Failed to load match log detail:', error);
    state.matchLogErrors.set(matchId, t('toast.matchDetailsFailed'));
  } finally {
    if (state.loadingMatchLogId === matchId) {
      state.loadingMatchLogId = null;
    }
    renderMatchesList();
  }
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
    label.textContent = isSyncing ? t('dashboard.syncingLatestMatch') : t('dashboard.syncNow');
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

  // Helper function to close toast with animation
  const closeToast = () => {
    toast.classList.add('closing');
    // Remove after animation completes (matches 0.3s duration in CSS)
    setTimeout(() => {
      toast.remove();
    }, 300);
  };

  toast.querySelector('.toast-close')?.addEventListener('click', closeToast);

  container.appendChild(toast);

  setTimeout(closeToast, 5000);
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

// Confirmation modal callback
let confirmCallback: (() => void) | null = null;

function closeAllModals() {
  const overlay = document.getElementById('modal-overlay');
  const modals = document.querySelectorAll('.modal');
  const teammateIdInput = document.getElementById('teammate-id') as HTMLInputElement | null;
  const pointNoteMatchIdInput = document.getElementById('point-note-match-id') as HTMLInputElement | null;
  const manualFriendForm = document.getElementById('manual-friend-form') as HTMLFormElement | null;
  const recentTeammatesList = document.getElementById('recent-teammates-list');
  const recentTeammatesEmpty = document.getElementById('recent-teammates-empty');
  if (overlay) {
    overlay.classList.add('hidden');
  }
  modals.forEach(modal => {
    modal.classList.add('hidden');
  });
  if (teammateIdInput) teammateIdInput.value = '';
  if (pointNoteMatchIdInput) pointNoteMatchIdInput.value = '';
  manualFriendForm?.reset();
  if (recentTeammatesList) {
    recentTeammatesList.innerHTML = '';
    recentTeammatesList.classList.add('hidden');
  }
  if (recentTeammatesEmpty) {
    recentTeammatesEmpty.textContent = t('friends.loadingRecentTeammates');
    recentTeammatesEmpty.classList.remove('hidden');
  }
  confirmCallback = null;
}

/**
 * Open a confirmation modal
 * @param title Modal title
 * @param message Modal message
 * @param confirmText Text for confirm button
 * @param onConfirm Callback to run when confirm is clicked
 * @param confirmButtonClass Optional class for the confirm button (e.g. 'btn-danger')
 * @param iconType Optional icon type: 'info' | 'warning' | 'danger' | 'success'
 */
function openConfirmModal(
  title: string,
  message: string,
  confirmText: string,
  onConfirm: () => void,
  confirmButtonClass: string = 'btn-primary',
  iconType: 'info' | 'warning' | 'danger' | 'success' = 'info'
) {
  const titleEl = document.getElementById('confirm-modal-title');
  const messageEl = document.getElementById('confirm-modal-message');
  const confirmBtn = document.getElementById('confirm-modal-confirm') as HTMLButtonElement | null;
  const iconEl = document.getElementById('confirm-modal-icon');
  
  if (titleEl) titleEl.textContent = title;
  if (messageEl) messageEl.textContent = message;
  if (confirmBtn) {
    confirmBtn.textContent = confirmText;
    // Reset button classes
    confirmBtn.className = `btn ${confirmButtonClass}`;
  }
  
  // Set up icon
  if (iconEl) {
    iconEl.className = `modal-icon modal-icon-${iconType}`;
    iconEl.classList.remove('hidden');
    
    // Update icon SVG based on type
    const svg = iconEl.querySelector('svg');
    if (svg) {
      switch (iconType) {
        case 'warning':
          svg.innerHTML = `
            <title>Warning icon</title>
            <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
            <line x1="12" y1="9" x2="12" y2="13"/>
            <line x1="12" y1="17" x2="12.01" y2="17"/>
          `;
          break;
        case 'danger':
          svg.innerHTML = `
            <title>Danger icon</title>
            <circle cx="12" cy="12" r="10"/>
            <line x1="15" y1="9" x2="9" y2="15"/>
            <line x1="9" y1="9" x2="15" y2="15"/>
          `;
          break;
        case 'success':
          svg.innerHTML = `
            <title>Success icon</title>
            <circle cx="12" cy="12" r="10"/>
            <path d="m9 12 2 2 4-4"/>
          `;
          break;
        default:
          svg.innerHTML = `
            <title>Information icon</title>
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
          `;
          break;
      }
    }
  }
  
  confirmCallback = onConfirm;
  openModal('modal-confirm');
}

// Navigation
export function navigateTo(viewId: string) {
  // Save current view to localStorage for persistence
  localStorage.setItem('lastActiveView', viewId);

  if (viewId !== 'points') {
    clearPendingSettleState();
  }

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
        loadRules();
        loadPointRecords();
        break;
     case 'settings':
       loadSettings();
       break;
     case 'notifications':
       loadNotifications();
       break;
    }
}

// Data loading functions
async function loadAppStatus() {
  try {
    const api = getAPI();
    const [appStatus, gameProcessStatus, activeAccount] = await Promise.all([
      api.app.getStatus(),
      api.app.getGameProcessStatus(),
      api.accounts.getActive(),
    ]);
    state.appStatus = appStatus;
    state.gameProcessStatus = gameProcessStatus;
    state.hasConfiguredApiKey = Boolean(activeAccount?.pubgApiKey?.trim());
    state.syncStatus = state.appStatus.syncStatus;
    updateSyncIndicator();
  } catch (error) {
    console.error('Failed to load app status:', error);
  }
}

async function loadDashboard() {
  try {
    const api = getAPI();

    const [appStatus, gameProcessStatus, activeAccount, unsettledSummary, activeRule, matches] = await Promise.all([
      api.app.getStatus(),
      api.app.getGameProcessStatus(),
      api.accounts.getActive(),
      api.points.getUnsettledSummary(),
      api.rules.getActive(),
      api.matches.getAll(10, 0),
    ]);

    state.appStatus = appStatus;
    state.gameProcessStatus = gameProcessStatus;
    state.hasConfiguredApiKey = Boolean(activeAccount?.pubgApiKey?.trim());
    state.syncStatus = appStatus.syncStatus;
    state.unsettledSummary = unsettledSummary;
    state.activeRule = activeRule;
    state.matches = matches;

    const matchResults = await Promise.all(
      matches.map(async (match) => {
        try {
          const players = await api.matches.getPlayers(match.matchId);
          return buildDashboardRecentMatchRow({
            match,
            players,
          });
        } catch (error) {
          console.error(`Failed to load dashboard detail for ${match.matchId}:`, error);
          return null;
        }
      }),
    );

    const successfulResults = matchResults.filter((item): item is DashboardRecentMatchRow => item !== null);
    const failedCount = matches.length - successfulResults.length;
    
    state.dashboardRecentMatches = successfulResults;
    if (
      state.expandedDashboardMatchId
      && !state.dashboardRecentMatches.some((item) => item.matchId === state.expandedDashboardMatchId)
    ) {
      state.expandedDashboardMatchId = null;
    }

    if (failedCount > 0) {
      showToast(t('toast.dashboardRecentMatchesPartialFailed', { count: failedCount }), 'warning');
    }

    updateDashboardStatus();
    updateSyncIndicator();
    updateActiveRule();
    
    // Render new dashboard components
    renderDashboardUnsettledSummary();
    renderDashboardRecentMatches();
  } catch (error) {
    console.error('Failed to load dashboard:', error);
    showToast(t('toast.dashboardLoadFailed'), 'error');
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
    if (text) text.textContent = t('sync.syncing');
  } else if (state.syncStatus.error) {
    dot?.classList.remove('syncing');
    dot?.classList.add('error');
    if (text) text.textContent = t('sync.syncError');
  } else if (!state.hasConfiguredApiKey) {
    dot?.classList.remove('syncing');
    dot?.classList.add('error');
    if (text) text.textContent = t('sync.apiKeyMissing');
  } else {
    dot?.classList.remove('syncing', 'error');
    if (text) text.textContent = t('status.ready');
  }

  setSyncNowButtonState();
}

function updateActiveRule() {
  const activeRuleEl = document.getElementById('active-rule');
  if (activeRuleEl) {
    activeRuleEl.textContent = state.activeRule?.name || t('dashboard.none');
  }
}

function renderDashboardUnsettledSummary() {
  const configEl = document.getElementById('dashboard-unsettled-config');
  const playersEl = document.getElementById('dashboard-unsettled-players');
  const ruleTextEl = document.getElementById('dashboard-unsettled-rule-text');
  const countBadgeEl = document.getElementById('dashboard-unsettled-count-badge');

  if (!configEl || !playersEl || !ruleTextEl || !countBadgeEl) return;

  configEl.innerHTML = '';
  configEl.classList.add('hidden');

  const unsettledSummary = state.unsettledSummary;

  ruleTextEl.textContent = unsettledSummary?.activeRuleName ?? '--';
  countBadgeEl.textContent = `${t('points.unsettledMatches')}: ${unsettledSummary?.unsettledMatchCount ?? 0}`;

  if (!unsettledSummary || unsettledSummary.players.length === 0) {
    playersEl.innerHTML = `<div class="points-summary-empty text-muted">${escapeHtml(t('points.unsettledEmpty'))}</div>`;
    return;
  }

  playersEl.innerHTML = unsettledSummary.players.map((player) => {
    const displayName = escapeHtml(player.displayNickname || player.pubgPlayerName);
    const deltaClass = player.totalDelta > 0 ? 'positive' : player.totalDelta < 0 ? 'negative' : 'zero';
    return `
      <div class="points-summary-player-chip">
        <div class="points-summary-player-name">${displayName}${player.isSelf ? `<span class="points-self-tag">${escapeHtml(t('common.you'))}</span>` : ''}</div>
        <div class="point-delta ${deltaClass}">${escapeHtml(formatSignedInteger(player.totalDelta))}</div>
      </div>
    `;
  }).join('');
}

function renderDashboardRecentMatches() {
  const emptyEl = document.getElementById('dashboard-recent-empty');
  const listEl = document.getElementById('dashboard-recent-list');

  if (!emptyEl || !listEl) return;

  if (state.dashboardRecentMatches.length === 0) {
    emptyEl.classList.remove('hidden');
    listEl.classList.add('hidden');
    return;
  }

  emptyEl.classList.add('hidden');
  listEl.classList.remove('hidden');

  listEl.innerHTML = state.dashboardRecentMatches.map((match) => {
    const isExpanded = state.expandedDashboardMatchId === match.matchId;
    const self = match.selfPlayer;
    const toggleLabel = isExpanded ? t('dashboard.collapseSquad') : t('dashboard.expandSquad');
    const battleChips = match.battleDeltas.length > 0
      ? match.battleDeltas.map((delta) => {
          const deltaClass = delta.delta > 0 ? 'positive' : 'negative';
          return `
            <div class="point-battle-chip ${deltaClass}">
              <span class="point-battle-name">${escapeHtml(delta.displayName)}</span>
              <span class="point-delta ${deltaClass}">${escapeHtml(formatSignedInteger(delta.delta))}</span>
            </div>
          `;
        }).join('')
      : `<span class="text-muted">${escapeHtml(t('dashboard.noBattleData'))}</span>`;

    return `
      <div class="dashboard-match-row ${isExpanded ? 'expanded' : ''}" data-match-id="${match.matchId}" data-dashboard-match="true">
        <button
          type="button"
          class="dashboard-match-header dashboard-match-trigger"
          data-dashboard-match-toggle="${escapeHtml(match.matchId)}"
          aria-expanded="${isExpanded ? 'true' : 'false'}"
          aria-label="${escapeHtml(toggleLabel)}"
        >
          <div class="dashboard-match-info">
            <span class="dashboard-match-map">${escapeHtml(match.mapName)}</span>
            <span class="dashboard-match-mode">${escapeHtml(match.gameMode)}</span>
            <span class="dashboard-match-date">${formatDateTime(match.playedAt)}</span>
            <span class="status-badge status-${match.status}">${translateMatchStatus(match.status)}</span>
          </div>
          <div class="dashboard-match-battle">
            <div class="dashboard-match-battle-label">${escapeHtml(t('dashboard.pointBattle'))}</div>
            <div class="point-battle-row">${battleChips}</div>
          </div>
          <div class="dashboard-match-stats">
            <span class="badge badge-info">${escapeHtml(t('dashboard.selfStats'))}</span>
            <div class="dashboard-stat">
              <span class="dashboard-stat-value">${self.kills}</span>
              <span class="dashboard-stat-label">${t('detail.kills')}</span>
            </div>
            <div class="dashboard-stat">
              <span class="dashboard-stat-value">${formatInteger(self.damage)}</span>
              <span class="dashboard-stat-label">${t('detail.damage')}</span>
            </div>
            <div class="dashboard-stat">
              <span class="dashboard-stat-value">${self.assists}</span>
              <span class="dashboard-stat-label">${t('detail.assists')}</span>
            </div>
            <div class="dashboard-stat">
              <span class="dashboard-stat-value">${self.revives}</span>
              <span class="dashboard-stat-label">${t('detail.revives')}</span>
            </div>
          </div>
        </button>
        ${isExpanded ? `
          <div class="dashboard-squad-rows">
            ${match.squad.map((player) => `
              <div class="dashboard-squad-row ${player.isSelf ? 'self' : ''}">
                <span class="squad-player-name">${escapeHtml(player.displayName)}${player.isSelf ? `<span class="points-self-tag">${escapeHtml(t('common.you'))}</span>` : ''}</span>
                <span class="squad-player-stat">${player.kills} ${t('detail.kills')}</span>
                <span class="squad-player-stat">${formatInteger(player.damage)} ${t('detail.damage')}</span>
                <span class="squad-player-stat">${player.assists} ${t('detail.assists')}</span>
                <span class="squad-player-stat">${player.revives} ${t('detail.revives')}</span>
              </div>
            `).join('')}
          </div>
        ` : ''}
      </div>
    `;
  }).join('');
}

function toggleDashboardMatch(matchId: string) {
  state.expandedDashboardMatchId = state.expandedDashboardMatchId === matchId ? null : matchId;
  renderDashboardRecentMatches();
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
    showToast(t('toast.teammatesLoadFailed'), 'error');
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

  const sortedTeammates = [...state.teammates].sort((left, right) => {
    const leftRecent = isRecentlyActiveTeammate(left);
    const rightRecent = isRecentlyActiveTeammate(right);

    if (leftRecent !== rightRecent) {
      return leftRecent ? -1 : 1;
    }

    const leftSortTime = getTeammateLastPlayedMeta(left).sortTime;
    const rightSortTime = getTeammateLastPlayedMeta(right).sortTime;
    if (leftSortTime !== rightSortTime) {
      return rightSortTime - leftSortTime;
    }

    return left.pubgPlayerName.localeCompare(right.pubgPlayerName, state.locale);
  });

  const recentTeammates = sortedTeammates.filter(isRecentlyActiveTeammate);
  const inactiveTeammates = sortedTeammates.filter((teammate) => !isRecentlyActiveTeammate(teammate));

  const renderTeammateCard = (teammate: Teammate) => {
    const lastPlayedMeta = getTeammateLastPlayedMeta(teammate);
    return `
    <div class="friend-row ${teammate.isPointsEnabled ? 'enabled' : 'disabled'}">
      <div class="friend-row-main">
        <div class="friend-row-label">${t('friends.playerName')}</div>
        <div class="friend-row-value">${escapeHtml(teammate.pubgPlayerName)}</div>
        <div class="friend-row-meta">${escapeHtml(lastPlayedMeta.label)}</div>
        ${teammate.pubgAccountId ? `<div class="friend-row-meta">ID: ${escapeHtml(teammate.pubgAccountId)}</div>` : ''}
      </div>
      <div class="friend-row-main">
        <div class="friend-row-label">${t('friends.savedNickname')}</div>
        <div class="friend-row-value ${teammate.displayNickname ? '' : 'muted'}">${escapeHtml(teammate.displayNickname || t('friends.notSet'))}</div>
      </div>
      <div class="friend-row-actions friend-row-actions-stack">
        <div class="friend-participation-control">
          <span class="friend-row-label">${t('friends.participatesInBattle')}</span>
          <div class="friend-participation-toggle">
            <button
              type="button"
              class="participation-switch ${teammate.isPointsEnabled ? 'active' : ''}"
              aria-pressed="${teammate.isPointsEnabled ? 'true' : 'false'}"
              data-teammate-action="toggle-participation"
              data-teammate-id="${teammate.id}"
              data-teammate-enabled="${teammate.isPointsEnabled ? '1' : '0'}"
            ></button>
            ${teammate.isPointsEnabled ? '' : `<span class="badge badge-warning">${escapeHtml(t('friends.notParticipating'))}</span>`}
          </div>
        </div>
        <div class="friend-action-row">
          <button type="button" class="btn btn-secondary" data-teammate-action="edit-nickname" data-teammate-id="${teammate.id}">
            ${teammate.displayNickname ? t('friends.editNickname') : t('friends.addNickname')}
          </button>
          <button type="button" class="btn btn-danger" data-teammate-action="delete" data-teammate-id="${teammate.id}">
            ${t('friends.delete')}
          </button>
        </div>
      </div>
    </div>
  `;
  };

  const renderSection = (title: string, teammates: Teammate[]) => {
    if (teammates.length === 0) {
      return '';
    }

    return `
      <div class="friend-section">
        <div class="friend-section-divider" aria-hidden="true"></div>
        <div class="friend-section-title">${escapeHtml(title)}</div>
        <div class="friend-section-list">
          ${teammates.map(renderTeammateCard).join('')}
        </div>
      </div>
    `;
  };

  listEl.innerHTML = [
    renderSection(t('friends.sectionRecentActive'), recentTeammates),
    renderSection(t('friends.sectionInactive'), inactiveTeammates),
  ].join('');
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
    syncSelectedUnsettledRuleId();
    renderRulesList();
    if (getActiveViewId() === 'points') {
      renderPointRecordsList();
    }
  } catch (error) {
    console.error('Failed to load rules:', error);
    showToast(t('toast.rulesLoadFailed'), 'error');
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
      ${rule.isActive ? `<span class="badge badge-success">${t('rules.active')}</span>` : ''}
      <div class="card-stats">
        <div class="card-stat">
          <div class="card-stat-value">${rule.damagePointsPerDamage}</div>
          <div class="card-stat-label">${t('rules.pointsPerDamage')}</div>
        </div>
        <div class="card-stat">
          <div class="card-stat-value">${rule.killPoints}</div>
          <div class="card-stat-label">${t('rules.pointsPerKill')}</div>
        </div>
        <div class="card-stat">
          <div class="card-stat-value">${rule.revivePoints}</div>
          <div class="card-stat-label">${t('rules.pointsPerRevive')}</div>
        </div>
      </div>
      <div class="card-actions">
        ${!rule.isActive ? `<button type="button" class="btn btn-secondary" data-rule-action="activate" data-rule-id="${rule.id}">${t('rules.activate')}</button>` : ''}
        <button type="button" class="btn btn-secondary" data-rule-action="edit" data-rule-id="${rule.id}">${t('rules.edit')}</button>
        <button type="button" class="btn btn-danger" data-rule-action="delete" data-rule-id="${rule.id}">${t('rules.delete')}</button>
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
    state.matchPlacements.clear();
    state.matchBattleDeltas.clear();

    const summaries = await Promise.all(
      matches.map(async (match) => {
        try {
          const players = await api.matches.getPlayers(match.matchId);
          const candidates = players.map((player) => ({
            matchPlayerId: player.id,
            isSelf: player.isSelf,
            isPointsEnabled: player.isPointsEnabledSnapshot,
            placement: player.placement,
            points: player.points,
          }));

          return {
            matchId: match.matchId,
            placement: getMatchListPlacement(candidates),
            delta: getMatchListBattleDelta(candidates),
          };
        } catch (error) {
          console.error(`Failed to load players for match ${match.matchId}:`, error);
          return {
            matchId: match.matchId,
            placement: null,
            delta: null,
          };
        }
      }),
    );

    for (const summary of summaries) {
      state.matchPlacements.set(summary.matchId, summary.placement);
      state.matchBattleDeltas.set(summary.matchId, summary.delta);
    }

    state.expandedMatchLogId = null;
    state.loadingMatchLogId = null;
    state.matchLogDetails.clear();
    state.matchLogErrors.clear();
    renderMatchesList();
  } catch (error) {
    console.error('Failed to load matches:', error);
    showToast(t('toast.matchesLoadFailed'), 'error');
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

  const renderMatchRankSummary = (matchId: string) => {
    const placement = state.matchPlacements.get(matchId);
    const delta = state.matchBattleDeltas.get(matchId);
    const rankText = placement != null ? `#${formatInteger(placement)}` : '--';

    if (delta === null || delta === undefined) {
      return `<span class="match-rank">${rankText}</span>`;
    }

    const deltaClass = delta > 0 ? 'positive' : delta < 0 ? 'negative' : 'zero';
    return `<span class="match-rank-summary"><span class="match-rank">${rankText}</span><span class="match-delta ${deltaClass}">${escapeHtml(formatSignedInteger(delta))}</span></span>`;
  };
  
  listEl.innerHTML = state.matches.map(match => `
    <tr class="match-log-summary-row ${state.expandedMatchLogId === match.matchId ? 'expanded' : ''}">
      <td>${renderMatchRankSummary(match.matchId)}</td>
      <td>${match.mapName || t('common.unknown')}</td>
      <td>${match.gameMode || t('common.unknown')}</td>
      <td>${formatDateTime(match.matchEndAt || match.playedAt)}</td>
      <td><span class="status-badge status-${match.status}">${translateMatchStatus(match.status)}</span></td>
      <td class="match-log-action-cell">
        <div class="match-log-action-trigger ${state.expandedMatchLogId === match.matchId ? 'expanded' : ''}" data-match-action="toggle-log" data-match-id="${escapeHtml(match.matchId)}" tabindex="0" role="button" aria-expanded="${state.expandedMatchLogId === match.matchId ? 'true' : 'false'}">
          <span class="match-log-action-label">${escapeHtml(t('matches.view'))}</span>
          <span class="match-log-action-meta">${escapeHtml(state.expandedMatchLogId === match.matchId ? t('matches.hideLog') : t('matches.log'))}</span>
        </div>
      </td>
    </tr>
    ${state.expandedMatchLogId === match.matchId ? `
      <tr class="match-log-detail-row">
        <td colspan="6" class="match-log-detail-cell">${renderMatchLogPanel(match.matchId)}</td>
      </tr>
    ` : ''}
  `).join('');
}

// Point records view
async function loadPointRecords() {
  try {
    const api = getAPI();
    const [pointHistory, unsettledSummary] = await Promise.all([
      api.points.getHistoryGroups(50, 0),
      api.points.getUnsettledSummary(),
    ]);
    state.pointHistory = pointHistory;
    state.unsettledSummary = unsettledSummary;
    syncSelectedUnsettledRuleId();

    if (
      state.pendingSettleMatchId
      && !state.pointHistory.some(
        (item) => item.type === 'match_group' && item.matchId === state.pendingSettleMatchId && !item.isSettled,
      )
    ) {
      clearPendingSettleState();
    }

    renderPointRecordsList();
  } catch (error) {
    console.error('Failed to load point records:', error);
    showToast(t('toast.pointsLoadFailed'), 'error');
  }
}

function renderPointRecordsList() {
  const emptyEl = document.getElementById('points-empty');
  const containerEl = document.getElementById('points-container');
  const unsettledPanelEl = document.getElementById('unsettled-summary-panel');
  const unsettledCountBadgeEl = document.getElementById('unsettled-count-badge');
  const unsettledRuleTextEl = document.getElementById('unsettled-rule-text');
  const unsettledRuleSelectEl = document.getElementById('unsettled-rule-select') as HTMLSelectElement | null;
  const recalculateUnsettledBtnEl = document.getElementById('btn-recalculate-unsettled') as HTMLButtonElement | null;
  const unsettledPlayersContainerEl = document.getElementById('unsettled-players-container');
  const matchHistoryListEl = document.getElementById('match-history-list');

  if (
    !emptyEl
    || !containerEl
    || !unsettledPanelEl
    || !unsettledCountBadgeEl
    || !unsettledRuleTextEl
    || !unsettledRuleSelectEl
    || !recalculateUnsettledBtnEl
    || !unsettledPlayersContainerEl
    || !matchHistoryListEl
  ) return;

  if (state.pointHistory.length === 0) {
    emptyEl.classList.remove('hidden');
    containerEl.classList.add('hidden');
    return;
  }

  emptyEl.classList.add('hidden');
  containerEl.classList.remove('hidden');

  const unsettledSummary = state.unsettledSummary;
  const activeRuleLabel = unsettledSummary?.activeRuleName ?? '--';
  unsettledRuleTextEl.textContent = `${t('points.ruleName')}: ${activeRuleLabel}`;
  unsettledCountBadgeEl.textContent = `${t('points.unsettledMatches')}: ${unsettledSummary?.unsettledMatchCount ?? 0}`;

  unsettledRuleSelectEl.innerHTML = state.rules.map((rule) => `
    <option value="${rule.id}">${escapeHtml(rule.name)}</option>
  `).join('');

  if (state.selectedUnsettledRuleId !== null) {
    unsettledRuleSelectEl.value = state.selectedUnsettledRuleId.toString();
  }

  const hasUnsettledMatches = (unsettledSummary?.unsettledMatchCount ?? 0) > 0;
  unsettledRuleSelectEl.disabled = state.rules.length === 0;
  recalculateUnsettledBtnEl.disabled = !hasUnsettledMatches || state.selectedUnsettledRuleId === null;

  if (!unsettledSummary || unsettledSummary.players.length === 0) {
    unsettledPlayersContainerEl.innerHTML = `<div class="points-summary-empty text-muted">${escapeHtml(t('points.unsettledEmpty'))}</div>`;
  } else {
    unsettledPlayersContainerEl.innerHTML = unsettledSummary.players.map((player) => {
      const displayName = escapeHtml(player.displayNickname || player.pubgPlayerName);
      const deltaClass = player.totalDelta > 0 ? 'positive' : player.totalDelta < 0 ? 'negative' : 'zero';
      return `
        <div class="points-summary-player-chip">
          <div class="points-summary-player-name">${displayName}${player.isSelf ? `<span class="points-self-tag">${escapeHtml(t('common.you'))}</span>` : ''}</div>
          <div class="point-delta ${deltaClass}">${escapeHtml(formatSignedInteger(player.totalDelta))}</div>
        </div>
      `;
    }).join('');
  }

  matchHistoryListEl.innerHTML = state.pointHistory.map((item) => {
    if (item.type === 'rule_change_marker') {
      return `
        <div class="rule-change-marker">
          <span>${escapeHtml(item.previousRuleName)} → ${escapeHtml(item.nextRuleName)}</span>
          <span class="text-muted">${escapeHtml(formatDateTime(item.createdAt))}</span>
        </div>
      `;
    }

    const playerNamesById = new Map(item.players.map((player) => [
      player.matchPlayerId,
      player.displayNicknameSnapshot || player.pubgPlayerName,
    ]));

    const playerRows = item.players.map((player) => {
      const displayName = escapeHtml(player.displayNicknameSnapshot || player.pubgPlayerName);
      const disabledBadge = player.isPointsEnabledSnapshot
        ? ''
        : `<span class="badge badge-warning">${escapeHtml(t('friends.notParticipating'))}</span>`;

      const damageRate = formatInteger(player.damagePointsPerDamageSnapshot);
      const damageValue = formatInteger(player.damage);
      const damagePoints = formatInteger(player.damagePoints);
      const kills = formatInteger(player.kills);
      const killRate = formatInteger(player.killPointsSnapshot);
      const killPoints = formatInteger(player.killPoints);
      const revives = formatInteger(player.revives);
      const reviveRate = formatInteger(player.revivePointsSnapshot);
      const revivePoints = formatInteger(player.revivePoints);
      const totalPoints = formatInteger(player.totalPoints);

      return `
        <div class="point-player-row ${player.isPointsEnabledSnapshot ? '' : 'disabled'}">
          <div class="point-player-header">
            <div class="point-player-name-wrap">
              <span class="point-player-name">${displayName}${player.isSelf ? `<span class="points-self-tag">${escapeHtml(t('common.you'))}</span>` : ''}</span>
              ${disabledBadge}
            </div>
            <span class="point-player-total">${escapeHtml(totalPoints)}</span>
          </div>
          <div class="point-calc-line">
            ${escapeHtml(t('detail.damage'))} ${escapeHtml(damageValue)}×${escapeHtml(damageRate)}=${escapeHtml(damagePoints)} +
            ${escapeHtml(t('detail.kills'))} ${escapeHtml(kills)}×${escapeHtml(killRate)}=${escapeHtml(killPoints)} +
            ${escapeHtml(t('detail.revives'))} ${escapeHtml(revives)}×${escapeHtml(reviveRate)}=${escapeHtml(revivePoints)} =
            ${escapeHtml(t('detail.points'))} ${escapeHtml(totalPoints)}
          </div>
        </div>
      `;
    }).join('');

    const battleDeltas = item.battleDeltas.map((delta) => {
      const deltaClass = delta.delta > 0 ? 'positive' : delta.delta < 0 ? 'negative' : 'zero';
      const displayName = escapeHtml(playerNamesById.get(delta.matchPlayerId) || delta.pubgPlayerName);
      return `
        <div class="point-battle-chip ${deltaClass}">
          <span class="point-battle-name">${displayName}</span>
          <span class="point-delta ${deltaClass}">${escapeHtml(formatSignedInteger(delta.delta))}</span>
        </div>
      `;
    }).join('');

    const noteRow = item.note && item.note.trim()
      ? `
        <div class="note-row">
          <div class="note-content">
            <div class="note-text"><strong>${escapeHtml(t('points.note'))}:</strong> ${escapeHtml(item.note)}</div>
          </div>
        </div>
      `
      : '';

    const settleLabel = state.pendingSettleMatchId === item.matchId ? t('points.confirmSettle') : t('points.settle');
    const settleButton = item.isSettled
      ? `<span class="badge badge-success settled-badge">${escapeHtml(t('points.settled'))}</span>`
      : `
        <button type="button" class="btn btn-settlement ${state.pendingSettleMatchId === item.matchId ? 'pending' : ''}" data-points-action="settle" data-match-id="${escapeHtml(item.matchId)}">
          ${escapeHtml(settleLabel)}
        </button>
      `;

    return `
      <article class="match-history-card ${item.isSettled ? 'settled' : ''}">
        <div class="point-match-layout">
          <div class="point-match-content">
            <div class="match-history-header">
              <div class="match-history-meta">
                <div class="match-history-title">${escapeHtml(item.mapName || t('common.unknown'))} · ${escapeHtml(item.gameMode || t('common.unknown'))}</div>
                <div class="match-history-date">${escapeHtml(formatDateTime(item.playedAt))} · ${escapeHtml(truncateMatchId(item.matchId))}</div>
                <div class="point-match-submeta">
                  <span class="badge badge-info">${escapeHtml(t('points.ruleName'))}: ${escapeHtml(item.ruleNameSnapshot)}</span>
                  ${item.isSettled ? `<span class="badge badge-success">${escapeHtml(t('points.settled'))}</span>` : ''}
                </div>
              </div>
            </div>
            <div class="point-section-label">${escapeHtml(t('points.calculation'))}</div>
            <div class="point-player-list">${playerRows}</div>
            <div class="point-section-label">${escapeHtml(t('points.netBattle'))}</div>
            <div class="point-battle-row">${battleDeltas}</div>
            ${noteRow}
          </div>
          <aside class="match-action-column point-match-actions">
            <button type="button" class="btn btn-secondary" data-points-action="note" data-match-id="${escapeHtml(item.matchId)}">
              ${escapeHtml(item.note ? t('points.editNote') : t('points.addNote'))}
            </button>
            ${settleButton}
          </aside>
        </div>
      </article>
    `;
  }).join('');
}

function openPointNoteModal(matchId: string) {
  const group = getHistoryMatchGroup(matchId);
  const titleEl = document.getElementById('point-note-modal-title');
  const matchIdInput = document.getElementById('point-note-match-id') as HTMLInputElement | null;
  const noteContentInput = document.getElementById('point-note-content') as HTMLTextAreaElement | null;

  if (!group || !matchIdInput || !noteContentInput || !titleEl) {
    return;
  }

  titleEl.textContent = group.note ? t('points.editNote') : t('points.addNote');
  matchIdInput.value = matchId;
  noteContentInput.value = group.note ?? '';
  openModal('modal-point-note');
}

async function handlePointNoteSubmit(e: Event) {
  e.preventDefault();

  const form = e.target as HTMLFormElement;
  const matchIdInput = document.getElementById('point-note-match-id') as HTMLInputElement | null;
  const noteContentInput = document.getElementById('point-note-content') as HTMLTextAreaElement | null;

  if (!matchIdInput || !noteContentInput) return;

  try {
    const api = getAPI();
    await api.points.updateMatchNote({
      matchId: matchIdInput.value,
      note: noteContentInput.value.trim() || null,
    });
    showToast(t('toast.pointNoteSaved'));
    closeAllModals();
    form.reset();
    await loadPointRecords();
  } catch (error) {
    console.error('Failed to save note:', error);
    showToast(t('toast.pointNoteSaveFailed'), 'error');
  }
}

async function handleSettleMatch(matchId: string) {
  const group = getHistoryMatchGroup(matchId);
  if (!group || group.isSettled) {
    return;
  }

  if (state.pendingSettleMatchId !== matchId) {
    clearPendingSettleState();
    state.pendingSettleMatchId = matchId;
    state.pendingSettleTimerId = window.setTimeout(() => {
      clearPendingSettleState();
      renderPointRecordsList();
    }, 2000);
    renderPointRecordsList();
    return;
  }

  clearPendingSettleState();

  try {
    const api = getAPI();
    await api.points.settleThroughMatch({ endMatchId: matchId });
    showToast(t('toast.pointSettled'));
    await loadPointRecords();
  } catch (error) {
    console.error('Failed to settle points through match:', error);
    showToast(t('toast.pointSettleFailed'), 'error');
    renderPointRecordsList();
  }
}

async function handleRecalculateUnsettledMatches() {
  const ruleId = state.selectedUnsettledRuleId;
  if (ruleId === null || (state.unsettledSummary?.unsettledMatchCount ?? 0) === 0) {
    return;
  }

  openConfirmModal(
    t('confirm.pointRecalculate.title'),
    t('confirm.pointRecalculate.message'),
    t('confirm.pointRecalculate.confirm'),
    async () => {
      try {
        const api = getAPI();
        await api.points.recalculateUnsettled({ ruleId });
        await Promise.all([loadRules(), loadPointRecords(), loadDashboard()]);
        showToast(t('toast.pointRecalculated'));
      } catch (error) {
        console.error('Failed to recalculate unsettled matches:', error);
        showToast(t('toast.pointRecalculateFailed'), 'error');
      }
    },
    'btn-secondary',
    'warning'
  );
}

async function handleToggleTeammateParticipation(teammateId: number, isEnabled: boolean) {
  try {
    const api = getAPI();
    const teammate = state.teammates.find((entry) => entry.id === teammateId);
    if (!teammate) {
      return;
    }

    await api.teammates.update({
      id: teammateId,
      displayNickname: teammate.displayNickname,
      isPointsEnabled: isEnabled,
    });

    await Promise.all([loadTeammates(), loadPointRecords(), loadDashboard()]);
  } catch (error) {
    console.error('Failed to toggle participation:', error);
    showToast(t('toast.friendSaveFailed'), 'error');
  }
}

// Form handlers
async function handleTeammateSubmit(e: Event) {
  e.preventDefault();
  
  const form = e.target as HTMLFormElement;
  const idInput = document.getElementById('teammate-id') as HTMLInputElement;
  const nicknameInput = document.getElementById('teammate-nickname') as HTMLInputElement;
  
  try {
    const api = getAPI();
    if (!idInput.value) return;

    await api.teammates.update({
      id: parseInt(idInput.value, 10),
      displayNickname: nicknameInput.value || null,
    });
    showToast(t('toast.friendSaved'));
    
    closeAllModals();
    form.reset();
    await Promise.all([loadTeammates(), loadDashboard()]);
  } catch (error) {
    console.error('Failed to save teammate:', error);
    showToast(t('toast.friendSaveFailed'), 'error');
  }
}

async function handleManualFriendSubmit(e: Event) {
  e.preventDefault();

  const form = e.target as HTMLFormElement;
  const nameInput = document.getElementById('manual-friend-name') as HTMLInputElement | null;

  try {
    const api = getAPI();
    const activeAccount = await api.accounts.getActive();
    const playerName = nameInput?.value.trim() ?? '';

    if (!activeAccount || !playerName) {
      return;
    }

    await api.teammates.create({
      platform: activeAccount.selfPlatform,
      pubgAccountId: null,
      pubgPlayerName: playerName,
      isPointsEnabled: false,
    });

    closeAllModals();
    form.reset();
    showToast(t('toast.friendAdded'));
    await Promise.all([loadTeammates(), loadDashboard()]);
  } catch (error) {
    console.error('Failed to add manual friend:', error);
    showToast(t('toast.friendSaveFailed'), 'error');
  }
}

async function handleAddRecentTeammate(index: number) {
  const candidate = state.recentTeammateCandidates[index];
  if (!candidate || candidate.isFriend) {
    return;
  }

  try {
    const api = getAPI();
    await api.teammates.create({
      platform: candidate.platform,
      pubgAccountId: candidate.pubgAccountId,
      pubgPlayerName: candidate.pubgPlayerName,
      isPointsEnabled: false,
    });

    state.recentTeammateCandidates = state.recentTeammateCandidates.map((entry, entryIndex) => (
      entryIndex === index ? { ...entry, isFriend: true } : entry
    ));
    showToast(t('toast.friendAdded'));
    closeAllModals();
    await Promise.all([loadTeammates(), loadDashboard()]);
  } catch (error) {
    console.error('Failed to add recent teammate:', error);
    showToast(t('toast.friendSaveFailed'), 'error');
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
        id: parseInt(idInput.value, 10),
        name: nameInput.value,
        damagePointsPerDamage: parseInt(damageInput.value, 10) || 0,
        killPoints: parseInt(killInput.value, 10) || 0,
        revivePoints: parseInt(reviveInput.value, 10) || 0,
        roundingMode,
      });
      showToast(t('toast.ruleUpdated'));
    } else {
      // Create new
      await api.rules.create({
        name: nameInput.value,
        damagePointsPerDamage: parseInt(damageInput.value, 10) || 0,
        killPoints: parseInt(killInput.value, 10) || 0,
        revivePoints: parseInt(reviveInput.value, 10) || 0,
        roundingMode,
      });
      showToast(t('toast.ruleCreated'));
    }
    
    closeAllModals();
    form.reset();
    loadRules();
    loadDashboard();
  } catch (error) {
    console.error('Failed to save rule:', error);
    showToast(t('toast.ruleSaveFailed'), 'error');
  }
}

async function handleSyncSubmit(e: Event) {
  e.preventDefault();
  
  const matchIdInput = document.getElementById('sync-match-id') as HTMLInputElement;
  const platformInput = document.getElementById('sync-platform') as HTMLSelectElement;
  const submitBtn = (e.target as HTMLFormElement).querySelector('button[type="submit"]') as HTMLButtonElement | null;
  const btnText = submitBtn?.querySelector('.btn-text');
  const btnSpinner = submitBtn?.querySelector('.btn-spinner');
  
  // Check for valid API key before proceeding
  if (!(await hasValidApiKey())) return;
  
  try {
    // Show loading state
    if (submitBtn) submitBtn.disabled = true;
    if (btnText) btnText.classList.add('hidden');
    if (btnSpinner) btnSpinner.classList.remove('hidden');
    
    const api = getAPI();
    const result = await api.sync.startMatch(matchIdInput.value, platformInput.value);
    
    if (result.success) {
      showToast(t('sync.matchCompleted'));
      closeAllModals();
      (e.target as HTMLFormElement).reset();
      loadMatches();
      loadPointRecords();
      loadDashboard();
    } else {
      showToast(result.error || t('sync.matchFailed'), 'error');
    }
  } catch (error) {
    console.error('Failed to sync match:', error);
    showToast(t('sync.matchFailed'), 'error');
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
    showToast(t('toast.pollingUnavailable'), 'error');
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
    showToast(t('toast.pollingSaved'));
  } catch (error) {
    console.error('Failed to save polling settings:', error);
    showToast(t('toast.pollingFailed'), 'error');
  } finally {
    if (submitBtn) submitBtn.disabled = false;
  }
}

async function handleImmediateRecentMatchCheck() {
  const syncNowButton = document.getElementById('btn-sync-now') as HTMLButtonElement | null;
  if (state.syncStatus?.isSyncing) return;
  
  // Check for valid API key before proceeding
  if (!(await hasValidApiKey())) return;

  try {
    if (syncNowButton) syncNowButton.disabled = true;

    const api = getAPI();
    const result = await api.sync.start();

    if (!result.success) {
      showToast(result.error || t('sync.checkRecentFailed'), 'error');
      return;
    }

    showToast(t('sync.checkRecentCompleted'));
    await Promise.all([loadDashboard(), loadMatches(), loadPointRecords()]);
  } catch (error) {
    console.error('Failed to check recent match:', error);
    showToast(t('sync.checkRecentFailed'), 'error');
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
    updateTeammateModalMode(teammate);
    openModal('modal-teammate');
  } catch (error) {
    console.error('Failed to load teammate:', error);
    showToast(t('toast.friendDetailsFailed'), 'error');
  }
};

window.deleteTeammate = async (id: number) => {
  openConfirmModal(
    t('confirm.friendDelete.title'),
    t('confirm.friendDelete.message'),
    t('confirm.friendDelete.confirm'),
    async () => {
      try {
        const api = getAPI();
        await api.teammates.delete(id);
        showToast(t('toast.friendDeleted'));
        await Promise.all([loadTeammates(), loadDashboard(), loadPointRecords()]);
      } catch (error) {
        console.error('Failed to delete friend:', error);
        showToast(t('toast.friendDeleteFailed'), 'error');
      }
    },
    'btn-danger',
    'danger'
  );
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
    if (titleEl) titleEl.textContent = t('rules.edit');
    
    openModal('modal-rule');
  } catch (error) {
    console.error('Failed to load rule:', error);
    showToast(t('toast.ruleLoadFailed'), 'error');
  }
};

window.activateRule = async (id: number) => {
  try {
    const api = getAPI();
    await api.rules.activate(id);
    showToast(t('toast.ruleActivated'));
    loadRules();
    loadDashboard();
  } catch (error) {
    console.error('Failed to activate rule:', error);
    showToast(t('toast.ruleActivateFailed'), 'error');
  }
};

window.deleteRule = async (id: number) => {
  openConfirmModal(
    t('confirm.ruleDelete.title'),
    t('confirm.ruleDelete.message'),
    t('confirm.ruleDelete.confirm'),
    async () => {
      try {
        const api = getAPI();
        await api.rules.delete(id);
        showToast(t('toast.ruleDeleted'));
        loadRules();
        loadDashboard();
      } catch (error) {
        console.error('Failed to delete rule:', error);
        showToast(t('toast.ruleDeleteFailed'), 'error');
      }
    },
    'btn-danger',
    'danger'
  );
};

/**
 * Custom Dropdown Initialization
 * Converts native <select> elements into styled custom dropdowns
 */
function initCustomDropdowns() {
  // Close all open dropdowns when clicking outside
  document.addEventListener('click', (e) => {
    const openDropdowns = document.querySelectorAll('.custom-dropdown-trigger.open');
    openDropdowns.forEach(trigger => {
      const dropdown = trigger.closest('.custom-dropdown');
      if (dropdown && !dropdown.contains(e.target as Node)) {
        closeDropdown(trigger as HTMLElement);
      }
    });
  });

  // Close all dropdowns on Escape key
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      const openDropdowns = document.querySelectorAll('.custom-dropdown-trigger.open');
      openDropdowns.forEach(trigger => {
        closeDropdown(trigger as HTMLElement);
      });
    }
  });

  // Initialize all select elements
  document.querySelectorAll('select.form-select').forEach(selectEl => {
    const select = selectEl as HTMLSelectElement;
    if (select.closest('.custom-dropdown')) return; // Already initialized
    if (select.classList.contains('native-select')) return; // Opt-out of custom dropdown

    // Create wrapper
    const wrapper = document.createElement('div');
    wrapper.className = 'custom-dropdown';
    select.parentNode?.insertBefore(wrapper, select);
    wrapper.appendChild(select);
    select.tabIndex = -1;
    select.setAttribute('aria-hidden', 'true');

    // Create trigger
    const trigger = document.createElement('div');
    trigger.className = 'custom-dropdown-trigger';
    trigger.setAttribute('tabindex', '0');
    trigger.setAttribute('role', 'combobox');
    trigger.setAttribute('aria-haspopup', 'listbox');
    trigger.setAttribute('aria-expanded', 'false');
    
    // Get selected option text
    const selectedOption = select.options[select.selectedIndex];
    trigger.textContent = selectedOption?.textContent || '';
    wrapper.appendChild(trigger);

    if (select.id) {
      document.querySelectorAll(`label[for="${select.id}"]`).forEach((label) => {
        label.addEventListener('click', (event) => {
          event.preventDefault();
          trigger.focus();
          if (!trigger.classList.contains('open')) {
            openDropdown(trigger);
          }
        });
      });
    }

    // Create menu
    const menu = document.createElement('div');
    menu.className = 'custom-dropdown-menu hidden';
    menu.setAttribute('role', 'listbox');
    wrapper.appendChild(menu);

    // Populate options
    function populateOptions() {
      menu.innerHTML = '';
      Array.from(select.options).forEach((option) => {
        const optionEl = document.createElement('div');
        optionEl.className = 'custom-dropdown-option';
        optionEl.setAttribute('role', 'option');
        optionEl.setAttribute('data-value', option.value);
        optionEl.setAttribute('aria-selected', option.selected ? 'true' : 'false');
        optionEl.textContent = option.textContent || '';
        
        if (option.selected) {
          optionEl.classList.add('selected');
        }
        
        if (option.disabled) {
          optionEl.classList.add('disabled');
        }

        // Click handler
        optionEl.addEventListener('click', () => {
          if (!option.disabled) {
            select.value = option.value;
            updateSelectedValue(wrapper, option.value);
            select.dispatchEvent(new Event('change', { bubbles: true }));
            closeDropdown(trigger);
            trigger.focus();
          }
        });

        menu.appendChild(optionEl);
      });
    }

    populateOptions();

    // Trigger click handler
    trigger.addEventListener('click', () => {
      if (trigger.classList.contains('open')) {
        closeDropdown(trigger);
      } else {
        // Close other open dropdowns first
        document.querySelectorAll('.custom-dropdown-trigger.open').forEach(otherTrigger => {
          if (otherTrigger !== trigger) {
            closeDropdown(otherTrigger as HTMLElement);
          }
        });
        openDropdown(trigger);
      }
    });

    // Trigger keyboard navigation
    trigger.addEventListener('keydown', (e) => {
      switch(e.key) {
        case 'Enter':
        case ' ': {
          e.preventDefault();
          trigger.click();
          break;
        }
        case 'ArrowDown': {
          e.preventDefault();
          if (!trigger.classList.contains('open')) {
            openDropdown(trigger);
          }
          focusNextOption(menu);
          break;
        }
        case 'ArrowUp': {
          e.preventDefault();
          if (!trigger.classList.contains('open')) {
            openDropdown(trigger);
          }
          focusPrevOption(menu);
          break;
        }
      }
    });

    // Menu keyboard navigation
    menu.addEventListener('keydown', (e) => {
      switch(e.key) {
        case 'Enter':
        case ' ': {
          e.preventDefault();
          const focused = menu.querySelector('.custom-dropdown-option.focused') as HTMLElement | null;
          if (focused && !focused.classList.contains('disabled')) {
            focused.click();
          }
          break;
        }
        case 'ArrowDown': {
          e.preventDefault();
          focusNextOption(menu);
          break;
        }
        case 'ArrowUp': {
          e.preventDefault();
          focusPrevOption(menu);
          break;
        }
        case 'Tab': {
          closeDropdown(trigger);
          break;
        }
      }
    });

    // Update trigger text when select value changes programmatically
    select.addEventListener('change', () => {
      updateSelectedValue(wrapper, select.value);
    });

    // Mutation observer to update options if they change dynamically (like language select)
    const observer = new MutationObserver(() => {
      populateOptions();
      updateSelectedValue(wrapper, select.value);
    });

    observer.observe(select, { childList: true, subtree: true });
  });
}

function openDropdown(trigger: HTMLElement) {
  const menu = trigger.nextElementSibling as HTMLElement;
  if (!menu) return;

  trigger.classList.add('open');
  trigger.setAttribute('aria-expanded', 'true');
  menu.classList.remove('hidden');
  
  // Focus first selected option or first option
  const selected = menu.querySelector('.custom-dropdown-option.selected');
  const firstOption = menu.querySelector('.custom-dropdown-option:not(.disabled)');
  (selected || firstOption)?.classList.add('focused');
}

function closeDropdown(trigger: HTMLElement) {
  const menu = trigger.nextElementSibling as HTMLElement;
  if (!menu) return;

  trigger.classList.remove('open');
  trigger.setAttribute('aria-expanded', 'false');
  menu.classList.add('hidden');
  
  // Remove focused state
  menu.querySelectorAll('.custom-dropdown-option.focused').forEach(el => {
    el.classList.remove('focused');
  });
}

function updateSelectedValue(wrapper: HTMLElement, value: string) {
  const trigger = wrapper.querySelector('.custom-dropdown-trigger') as HTMLElement;
  const menu = wrapper.querySelector('.custom-dropdown-menu') as HTMLElement;
  const select = wrapper.querySelector('select') as HTMLSelectElement;

  if (!trigger || !menu) return;

  // Update trigger text
  const selectedOption = Array.from(select.options).find(opt => opt.value === value);
  if (selectedOption) {
    trigger.textContent = selectedOption.textContent || '';
  }

  // Update selected classes
  menu.querySelectorAll('.custom-dropdown-option').forEach(el => {
    el.classList.remove('selected');
    el.setAttribute('aria-selected', 'false');
    if (el.getAttribute('data-value') === value) {
      el.classList.add('selected');
      el.setAttribute('aria-selected', 'true');
    }
  });
}

function focusNextOption(menu: HTMLElement) {
  const options = Array.from(menu.querySelectorAll('.custom-dropdown-option:not(.disabled)'));
  const currentIndex = options.findIndex(el => el.classList.contains('focused'));
  const nextIndex = (currentIndex + 1) % options.length;

  options.forEach(el => {
    el.classList.remove('focused');
  });
  options[nextIndex]?.classList.add('focused');
  options[nextIndex]?.scrollIntoView({ block: 'nearest' });
}

function focusPrevOption(menu: HTMLElement) {
  const options = Array.from(menu.querySelectorAll('.custom-dropdown-option:not(.disabled)'));
  const currentIndex = options.findIndex(el => el.classList.contains('focused'));
  const prevIndex = currentIndex === -1 ? options.length - 1 : (currentIndex - 1 + options.length) % options.length;

  options.forEach(el => {
    el.classList.remove('focused');
  });
  options[prevIndex]?.classList.add('focused');
  options[prevIndex]?.scrollIntoView({ block: 'nearest' });
}

// Event listeners
document.addEventListener('DOMContentLoaded', async () => {
  try {
    await loadLanguagePreference();

    // Initial data load
    await loadAppStatus();
    
    // Initialize custom dropdowns
    initCustomDropdowns();
    
    // Hide loading screen after a brief delay
    setTimeout(() => {
      hideLoadingScreen();
      // Load last active view from localStorage, default to dashboard
      const lastView = localStorage.getItem('lastActiveView') || 'dashboard';
      navigateTo(lastView);
    }, 1000);

    // Restore view when window becomes active again
    window.addEventListener('focus', () => {
      const lastView = localStorage.getItem('lastActiveView') || 'dashboard';
      if (getActiveViewId() !== lastView) {
        navigateTo(lastView);
      }
    });
    
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
    
    // Confirm modal confirm button
    document.getElementById('confirm-modal-confirm')?.addEventListener('click', () => {
      if (confirmCallback) {
        confirmCallback();
      }
      closeAllModals();
    });
    
    // Quick action buttons
    document.getElementById('btn-sync-now')?.addEventListener('click', () => {
      void handleImmediateRecentMatchCheck();
    });
    
    document.getElementById('btn-new-teammate')?.addEventListener('click', () => {
      void openRecentTeammatesModal();
    });

    document.getElementById('btn-manual-friend')?.addEventListener('click', () => {
      openManualFriendModal();
    });
    
    document.getElementById('btn-new-rule')?.addEventListener('click', () => {
      // Reset form
      const form = document.getElementById('rule-form') as HTMLFormElement;
      form?.reset();
      const idInput = document.getElementById('rule-id') as HTMLInputElement;
      const titleEl = document.getElementById('rule-modal-title');
      
      if (idInput) idInput.value = '';
      if (titleEl) titleEl.textContent = t('rules.create');
      
      openModal('modal-rule');
    });
    
    document.getElementById('btn-empty-create-rule')?.addEventListener('click', () => {
      document.getElementById('btn-new-rule')?.click();
    });

    document.getElementById('rules-list')?.addEventListener('click', (event) => {
      const target = (event.target as HTMLElement).closest<HTMLButtonElement>('button[data-rule-action][data-rule-id]');
      if (!target) {
        return;
      }

      const action = target.dataset.ruleAction;
      const id = Number.parseInt(target.dataset.ruleId ?? '', 10);
      if (!Number.isFinite(id)) {
        return;
      }

      if (action === 'activate') {
        void window.activateRule?.(id);
      } else if (action === 'edit') {
        void window.editRule?.(id);
      } else if (action === 'delete') {
        void window.deleteRule?.(id);
      }
    });

    document.getElementById('teammates-list')?.addEventListener('click', (event) => {
      const target = (event.target as HTMLElement).closest<HTMLButtonElement>('button[data-teammate-action][data-teammate-id]');
      if (!target) {
        return;
      }

      const teammateId = Number.parseInt(target.dataset.teammateId ?? '', 10);
      if (!Number.isFinite(teammateId)) {
        return;
      }

      if (target.dataset.teammateAction === 'edit-nickname') {
        void window.editTeammateNickname?.(teammateId);
        return;
      }

      if (target.dataset.teammateAction === 'delete') {
        void window.deleteTeammate?.(teammateId);
        return;
      }

      if (target.dataset.teammateAction === 'toggle-participation') {
        const isEnabled = target.dataset.teammateEnabled === '1';
        void handleToggleTeammateParticipation(teammateId, !isEnabled);
      }
    });

    document.getElementById('recent-teammates-list')?.addEventListener('click', (event) => {
      const target = (event.target as HTMLElement).closest<HTMLButtonElement>('button[data-recent-teammate-index]');
      if (!target || target.disabled) {
        return;
      }

      const index = Number.parseInt(target.dataset.recentTeammateIndex ?? '', 10);
      if (!Number.isFinite(index)) {
        return;
      }

      void handleAddRecentTeammate(index);
    });

    document.getElementById('match-history-list')?.addEventListener('click', (event) => {
      const target = (event.target as HTMLElement).closest<HTMLElement>('[data-points-action][data-match-id]');
      if (!target) {
        return;
      }

      const matchId = target.dataset.matchId;
      if (!matchId) {
        return;
      }

      if (target.dataset.pointsAction === 'note') {
        openPointNoteModal(matchId);
        return;
      }

      if (target.dataset.pointsAction === 'settle') {
        void handleSettleMatch(matchId);
      }
    });

    document.getElementById('matches-list')?.addEventListener('click', (event) => {
      const target = (event.target as HTMLElement).closest<HTMLElement>('[data-match-action="toggle-log"][data-match-id]');
      const matchId = target?.dataset.matchId;
      if (!target || !matchId) {
        return;
      }

      void toggleMatchLog(matchId);
    });

    document.getElementById('matches-list')?.addEventListener('keydown', (event) => {
      if (event.key !== 'Enter' && event.key !== ' ') {
        return;
      }

      const target = (event.target as HTMLElement).closest<HTMLElement>('[data-match-action="toggle-log"][data-match-id]');
      const matchId = target?.dataset.matchId;
      if (!target || !matchId) {
        return;
      }

      event.preventDefault();
      void toggleMatchLog(matchId);
    });

    document.getElementById('unsettled-rule-select')?.addEventListener('change', (event) => {
      const value = Number.parseInt((event.target as HTMLSelectElement).value, 10);
      state.selectedUnsettledRuleId = Number.isNaN(value) ? null : value;
      renderPointRecordsList();
    });

    document.getElementById('btn-recalculate-unsettled')?.addEventListener('click', () => {
      void handleRecalculateUnsettledMatches();
    });

    document.querySelector('[data-dashboard-view-link="matches"]')?.addEventListener('click', (event) => {
      event.preventDefault();
      navigateTo('matches');
    });

    // Dashboard recent matches click handler for expanding/collapsing squad rows
    document.getElementById('dashboard-recent-list')?.addEventListener('click', (event) => {
      const target = (event.target as HTMLElement).closest<HTMLElement>('[data-dashboard-match-toggle]');
      if (!target) {
        return;
      }

      const matchId = target.dataset.dashboardMatchToggle;
      if (!matchId) {
        return;
      }

      toggleDashboardMatch(matchId);
    });
    document.getElementById('btn-empty-sync-matches')?.addEventListener('click', () => {
      openModal('modal-sync');
    });
    
    // Form submissions
document.getElementById('teammate-form')?.addEventListener('submit', handleTeammateSubmit);
document.getElementById('manual-friend-form')?.addEventListener('submit', handleManualFriendSubmit);
document.getElementById('rule-form')?.addEventListener('submit', handleRuleSubmit);
document.getElementById('sync-form')?.addEventListener('submit', handleSyncSubmit);
document.getElementById('polling-settings-form')?.addEventListener('submit', handlePollingSettingsSubmit);
document.getElementById('language-settings-form')?.addEventListener('submit', handleLanguageSubmit);
document.getElementById('account-settings-form')?.addEventListener('submit', handleAccountSettingsSubmit);
document.getElementById('api-key-settings-form')?.addEventListener('submit', handleApiKeySettingsSubmit);
document.getElementById('point-note-form')?.addEventListener('submit', handlePointNoteSubmit);
     document.getElementById('btn-sync-friends-manual')?.addEventListener('click', () => {
       void openRecentTeammatesModal();
     });
     document.getElementById('btn-logout')?.addEventListener('click', handleLogout);
    
  } catch (error) {
    console.error('Failed to initialize app:', error);
    showErrorScreen(t('error.connectionMessage'));
  }
});

console.log('PUBG Point Rankings - Renderer App Loaded');
