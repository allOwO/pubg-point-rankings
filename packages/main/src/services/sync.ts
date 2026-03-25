/**
 * Sync Service
 * Orchestrates the end-to-end match synchronization workflow
 */

import type { Database } from 'better-sqlite3';
import type { 
  Match, 
  MatchPlayer, 
  CalculatedPoints,
  Platform 
} from '@pubg-point-rankings/shared';
import { PUBGApiClient } from '../pubg';
import { parseTelemetry, aggregatePlayerStats } from '../parser';
import { calculatePoints } from '../engine';
import { 
  SettingsRepository, 
  TeammatesRepository, 
  MatchesRepository, 
  PointRulesRepository,
  PointRecordsRepository 
} from '../repository';

export interface SyncResult {
  success: boolean;
  match?: Match;
  players?: MatchPlayer[];
  points?: CalculatedPoints[];
  error?: string;
}

export interface SyncRuntimeStatus {
  isSyncing: boolean;
  currentMatchId: string | null;
  lastError: string | null;
}

export class SyncService {
  private readonly db: Database;
  private pubgClient: PUBGApiClient;
  private settingsRepo: SettingsRepository;
  private teammatesRepo: TeammatesRepository;
  private matchesRepo: MatchesRepository;
  private rulesRepo: PointRulesRepository;
  private pointsRepo: PointRecordsRepository;
  private runtimeStatus: SyncRuntimeStatus = {
    isSyncing: false,
    currentMatchId: null,
    lastError: null,
  };

  constructor(
    db: Database,
    apiKey: string
  ) {
    this.db = db;
    this.pubgClient = new PUBGApiClient(apiKey);
    this.settingsRepo = new SettingsRepository(db);
    this.teammatesRepo = new TeammatesRepository(db);
    this.matchesRepo = new MatchesRepository(db);
    this.rulesRepo = new PointRulesRepository(db);
    this.pointsRepo = new PointRecordsRepository(db);
  }

  /**
   * Update API key
   */
  setApiKey(apiKey: string): void {
    this.pubgClient.setApiKey(apiKey);
  }

  getStatus(): SyncRuntimeStatus {
    return { ...this.runtimeStatus };
  }

  private beginSync(matchId: string): SyncResult | null {
    if (this.runtimeStatus.isSyncing) {
      return {
        success: false,
        error: this.runtimeStatus.currentMatchId
          ? `Another sync is already running for ${this.runtimeStatus.currentMatchId}`
          : 'Another sync is already running',
      };
    }

    this.runtimeStatus = {
      isSyncing: true,
      currentMatchId: matchId,
      lastError: null,
    };

    return null;
  }

  private endSync(error?: string): void {
    this.runtimeStatus = {
      isSyncing: false,
      currentMatchId: null,
      lastError: error ?? null,
    };
  }

  /**
   * Sync a specific match by ID
   */
  async syncMatch(matchId: string, platform?: Platform): Promise<SyncResult> {
    const busyResult = this.beginSync(matchId);
    if (busyResult) {
      return busyResult;
    }

    try {
      // Get platform from settings if not provided
      const targetPlatform = platform || this.settingsRepo.getString('self_platform', 'steam') as Platform;
      
      // Check if match already exists
      let match = this.matchesRepo.getById(matchId);
      
      if (!match) {
        // Fetch match from PUBG API
        const pubgMatch = await this.pubgClient.getMatch(matchId, targetPlatform);
        if (!pubgMatch) {
          return { success: false, error: 'Match not found in PUBG API' };
        }

        // Get telemetry URL
        const telemetryUrl = this.pubgClient.getTelemetryUrl(pubgMatch);

        // Create match record
        match = this.matchesRepo.create({
          matchId: pubgMatch.id,
          platform: targetPlatform,
          mapName: pubgMatch.attributes.mapName || null,
          gameMode: pubgMatch.attributes.gameMode || null,
          playedAt: new Date(pubgMatch.attributes.createdAt),
          telemetryUrl,
        });
      }

      // Skip if already synced
      if (match.status === 'ready' && this.pointsRepo.existsForMatch(matchId)) {
        const players = this.matchesRepo.getPlayers(matchId);
        return { success: true, match, players };
      }

      // Update status to syncing
      this.matchesRepo.updateStatus(matchId, 'syncing');

      // Check if we have telemetry URL
      if (!match.telemetryUrl) {
        this.matchesRepo.updateStatus(matchId, 'failed');
        return { success: false, error: 'No telemetry URL available for match' };
      }

      // Fetch and parse telemetry
      const telemetryJson = await this.pubgClient.getTelemetry(match.telemetryUrl);
      const events = parseTelemetry(telemetryJson);
      const playerStats = aggregatePlayerStats(events);

      // Get active rule
      const rule = this.rulesRepo.getActive();
      if (!rule) {
        this.matchesRepo.updateStatus(matchId, 'failed');
          return { success: false, error: 'No active point rule configured' };
      }

      // Find self player name from settings
      const selfPlayerName = this.settingsRepo.getString('self_player_name');
      
      // Get enabled player set
      const enabledPlayerIds = new Set<string>();
      const teammatesByAccountId = new Map<string, ReturnType<TeammatesRepository['findOrCreate']>>();
      const teammatesByName = new Map<string, ReturnType<TeammatesRepository['findOrCreate']>>();
      
      for (const stats of playerStats) {
        // Find or create teammate
        const teammate = this.teammatesRepo.findOrCreate(
          stats.pubgPlayerName,
          targetPlatform,
          stats.pubgAccountId
        );

        // Check if this is the self player
        const isSelf = stats.pubgPlayerName.toLowerCase() === selfPlayerName.toLowerCase();
        
        // Update teammate last seen
        this.teammatesRepo.updateLastSeen(teammate.id);

        // Add to enabled set if participating in points tracking
        if (teammate.pubgAccountId) {
          teammatesByAccountId.set(teammate.pubgAccountId, teammate);
        }
        teammatesByName.set(teammate.pubgPlayerName.toLowerCase(), teammate);

        if (teammate.isPointsEnabled || isSelf) {
          enabledPlayerIds.add(stats.pubgAccountId);
        }
      }

      // Calculate points
      const calculatedPoints = calculatePoints({
        rule,
        players: playerStats,
        enabledPlayerIds,
      });

      // Save match players and point records atomically.
      const savedPlayers: MatchPlayer[] = [];
      const selfAccountId = playerStats.find(
        p => p.pubgPlayerName.toLowerCase() === selfPlayerName.toLowerCase()
      )?.pubgAccountId;

      this.db.transaction(() => {
        for (const calc of calculatedPoints) {
          const teammate = (calc.pubgAccountId
            ? teammatesByAccountId.get(calc.pubgAccountId)
            : undefined)
            ?? teammatesByName.get(calc.pubgPlayerName.toLowerCase())
            ?? null;

          const player = this.matchesRepo.createPlayer({
            matchId,
            teammateId: teammate?.id ?? null,
            pubgAccountId: calc.pubgAccountId,
            pubgPlayerName: calc.pubgPlayerName,
            displayNicknameSnapshot: teammate?.displayNickname ?? calc.pubgPlayerName,
            damage: calc.damage,
            kills: calc.kills,
            revives: calc.revives,
            isSelf: calc.pubgAccountId === selfAccountId,
            isPointsEnabledSnapshot: calc.isPointsEnabled,
            points: calc.totalPoints,
          });

          savedPlayers.push(player);

          this.pointsRepo.create({
            matchId,
            matchPlayerId: player.id,
            teammateId: teammate?.id ?? null,
            ruleId: rule.id,
            ruleNameSnapshot: rule.name,
            damagePointsPerDamageSnapshot: rule.damagePointsPerDamage,
            killPointsSnapshot: rule.killPoints,
            revivePointsSnapshot: rule.revivePoints,
            roundingModeSnapshot: rule.roundingMode,
            points: calc.totalPoints,
          });

          if (teammate) {
            const totalPoints = this.pointsRepo.getTotalForTeammate(teammate.id);
            this.teammatesRepo.updateTotalPoints(teammate.id, totalPoints);
          }
        }

        this.matchesRepo.updateStatus(matchId, 'ready');
        this.settingsRepo.set('last_sync_at', new Date().toISOString());
      })();

      return {
        success: true,
        match,
        players: savedPlayers,
        points: calculatedPoints,
      };
    } catch (error) {
      this.matchesRepo.updateStatus(matchId, 'failed');
      const errorMessage = error instanceof Error ? error.message : 'Unknown error during sync';
      this.endSync(errorMessage);
      
      return {
        success: false,
        error: errorMessage,
      };
    } finally {
      if (this.runtimeStatus.isSyncing) {
        this.endSync();
      }
    }
  }

  /**
   * Sync the most recent match for the configured player
   */
  async syncRecentMatch(): Promise<SyncResult> {
    try {
      const apiKey = this.settingsRepo.getString('pubg_api_key');
      if (!apiKey) {
        return { success: false, error: 'PUBG API key not configured' };
      }

      const selfPlayerName = this.settingsRepo.getString('self_player_name');
      if (!selfPlayerName) {
        return { success: false, error: 'Player name not configured' };
      }

      const platform = this.settingsRepo.getString('self_platform', 'steam') as Platform;

      // Get player info
      const player = await this.pubgClient.getPlayerByName(selfPlayerName, platform);
      if (!player) {
        return { success: false, error: 'Player not found in PUBG API' };
      }

      // Get recent matches
      const matchIds = await this.pubgClient.getRecentMatches(player.id, platform, 1);
      if (matchIds.length === 0) {
        return { success: false, error: 'No recent matches found' };
      }

      const matchId = matchIds[0];

      // Check if already synced
      if (this.matchesRepo.exists(matchId) && this.pointsRepo.existsForMatch(matchId)) {
        const match = this.matchesRepo.getById(matchId)!;
        const players = this.matchesRepo.getPlayers(matchId);
        return { success: true, match, players };
      }

      // Sync the match
      return await this.syncMatch(matchId, platform);
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error during sync',
      };
    }
  }

  /**
   * Retry failed matches
   */
  async retryFailedMatches(): Promise<SyncResult[]> {
    const failedMatches = this.matchesRepo.getByStatus('failed');
    const results: SyncResult[] = [];

    for (const match of failedMatches) {
      const result = await this.syncMatch(match.matchId, match.platform);
      results.push(result);
    }

    return results;
  }
}
