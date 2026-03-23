/**
 * PUBG API Client
 * Handles all communication with PUBG Developer API
 * https://documentation.pubg.com/
 */

import type { Platform } from '@pubg-point-rankings/shared';

// PUBG API Response Types
export interface PUBGPlayer {
  id: string;
  attributes: {
    name: string;
    stats?: unknown;
    titleId?: string;
    shardId?: string;
    patchVersion?: string;
    banType?: string;
    clanId?: string;
  };
  relationships?: {
    assets?: { data: unknown[] };
    matches?: { data: Array<{ id: string; type: string }> };
  };
}

export interface PUBGPlayerResponse {
  data: PUBGPlayer[];
  links?: {
    self: string;
  };
  meta?: unknown;
}

export interface PUBGMatchParticipant {
  id: string;
  type: string;
  attributes: {
    stats: {
      DBNOs?: number;
      assists?: number;
      boosts?: number;
      damageDealt?: number;
      deathType?: string;
      headshotKills?: number;
      heals?: number;
      killPlace?: number;
      killStreaks?: number;
      kills?: number;
      longestKill?: number;
      name: string;
      playerId?: string;
      revives?: number;
      rideDistance?: number;
      roadKills?: number;
      swimDistance?: number;
      teamKills?: number;
      timeSurvived?: number;
      vehicleDestroys?: number;
      walkDistance?: number;
      weaponsAcquired?: number;
      winPlace?: number;
    };
    actor?: string;
    shardId?: string;
  };
}

export interface PUBGMatchRoster {
  id: string;
  type: string;
  attributes: {
    stats?: {
      rank?: number;
      teamId?: number;
    };
    won?: string;
    shardId?: string;
  };
  relationships?: {
    participants?: {
      data: Array<{ id: string; type: string }>;
    };
    team?: {
      data: unknown;
    };
  };
}

export interface PUBGAsset {
  id: string;
  type: string;
  attributes: {
    URL: string;
    name?: string;
    description?: string;
    createdAt?: string;
  };
}

export interface PUBGMatch {
  id: string;
  attributes: {
    gameMode: string;
    mapName: string;
    isCustomMatch: boolean;
    seasonState?: string;
    duration?: number;
    stats?: unknown;
    titleId?: string;
    shardId: string;
    tags?: unknown;
    createdAt: string;
  };
  relationships: {
    assets: {
      data: Array<{ id: string; type: string }>;
    };
    rosters: {
      data: Array<{ id: string; type: string; }>;
    };
    rounds?: {
      data: unknown[];
    };
    spectators?: {
      data: unknown[];
    };
  };
  included?: Array<PUBGMatchParticipant | PUBGMatchRoster | PUBGAsset>;
}

export interface PUBGMatchResponse {
  data: PUBGMatch;
  included?: Array<PUBGMatchParticipant | PUBGMatchRoster | PUBGAsset>;
  links?: {
    self: string;
  };
  meta?: unknown;
}

export interface PUBGErrorResponse {
  errors: Array<{
    title: string;
    detail?: string;
    status?: string;
  }>;
}

// Rate limiting
const RATE_LIMIT_REQUESTS = 10;
const RATE_LIMIT_WINDOW_MS = 60000; // 1 minute

interface RateLimitEntry {
  timestamp: number;
  count: number;
}

/**
 * PUBG API Client
 */
export class PUBGApiClient {
  private apiKey: string;
  private baseUrl = 'https://api.pubg.com/shards';
  private rateLimits: Map<string, RateLimitEntry> = new Map();

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  /**
   * Update API key
   */
  setApiKey(apiKey: string): void {
    this.apiKey = apiKey;
  }

  /**
   * Build request headers
   */
  private getHeaders(): Record<string, string> {
    return {
      'Authorization': `Bearer ${this.apiKey}`,
      'Accept': 'application/vnd.api+json',
      'Content-Type': 'application/vnd.api+json',
    };
  }

  /**
   * Check and handle rate limiting
   */
  private checkRateLimit(endpoint: string): boolean {
    const now = Date.now();
    const limit = this.rateLimits.get(endpoint);

    if (!limit || (now - limit.timestamp) > RATE_LIMIT_WINDOW_MS) {
      this.rateLimits.set(endpoint, { timestamp: now, count: 1 });
      return true;
    }

    if (limit.count < RATE_LIMIT_REQUESTS) {
      limit.count++;
      return true;
    }

    return false;
  }

  /**
   * Make an API request with retry logic
   */
  private async request<T>(
    url: string,
    options: RequestInit,
    retries: number = 3
  ): Promise<T> {
    // Check rate limit
    if (!this.checkRateLimit(url)) {
      throw new Error('Rate limit exceeded');
    }

    for (let attempt = 0; attempt < retries; attempt++) {
      try {
        const response = await fetch(url, {
          ...options,
          headers: {
            ...this.getHeaders(),
            ...(options.headers || {}),
          },
        });

        // Handle rate limit response
        if (response.status === 429) {
          const retryAfter = parseInt(response.headers.get('Retry-After') || '60', 10) * 1000;
          await new Promise(resolve => setTimeout(resolve, retryAfter));
          continue;
        }

        // Handle other errors
        if (!response.ok) {
          const errorData = await response.json().catch(() => null) as PUBGErrorResponse | null;
          const errorMessage = errorData?.errors?.[0]?.detail || errorData?.errors?.[0]?.title || `HTTP ${response.status}`;
          throw new Error(errorMessage);
        }

        return await response.json() as T;
      } catch (error) {
        if (attempt === retries - 1) throw error;
        
        // Exponential backoff
        await new Promise(resolve => setTimeout(resolve, Math.pow(2, attempt) * 1000));
      }
    }

    throw new Error('Max retries exceeded');
  }

  /**
   * Get player by name
   */
  async getPlayerByName(playerName: string, platform: Platform): Promise<PUBGPlayer | null> {
    const url = `${this.baseUrl}/${platform}/players?filter[playerNames]=${encodeURIComponent(playerName)}`;
    
    const response = await this.request<PUBGPlayerResponse>(url, {
      method: 'GET',
    });

    return response.data?.[0] || null;
  }

  /**
   * Get player by ID
   */
  async getPlayerById(playerId: string, platform: Platform): Promise<PUBGPlayer | null> {
    const url = `${this.baseUrl}/${platform}/players/${encodeURIComponent(playerId)}`;
    
    const response = await this.request<{ data: PUBGPlayer }>(url, {
      method: 'GET',
    });

    return response.data || null;
  }

  /**
   * Get multiple players by IDs
   */
  async getPlayersByIds(playerIds: string[], platform: Platform): Promise<PUBGPlayer[]> {
    if (playerIds.length === 0) return [];
    if (playerIds.length > 10) {
      // PUBG API limits to 10 players per request
      const chunks = [];
      for (let i = 0; i < playerIds.length; i += 10) {
        chunks.push(playerIds.slice(i, i + 10));
      }
      const results = await Promise.all(chunks.map(chunk => this.getPlayersByIds(chunk, platform)));
      return results.flat();
    }

    const url = `${this.baseUrl}/${platform}/players?filter[playerIds]=${playerIds.join(',')}`;
    
    const response = await this.request<PUBGPlayerResponse>(url, {
      method: 'GET',
    });

    return response.data || [];
  }

  /**
   * Get match details
   */
  async getMatch(matchId: string, platform: Platform): Promise<PUBGMatch | null> {
    const url = `${this.baseUrl}/${platform}/matches/${encodeURIComponent(matchId)}`;
    
    const response = await this.request<PUBGMatchResponse>(url, {
      method: 'GET',
    });

    if (!response.data) return null;

    // Merge included data into match object for easier access
    if (response.included) {
      response.data.included = response.included;
    }

    return response.data;
  }

  /**
   * Get telemetry data URL from match
   */
  getTelemetryUrl(match: PUBGMatch): string | null {
    const assetId = match.relationships?.assets?.data?.[0]?.id;
    if (!assetId || !match.included) return null;

    const asset = match.included.find(
      (item): item is PUBGAsset => 
        item.type === 'asset' && 
        item.id === assetId && 
        'attributes' in item
    );

    return asset?.attributes?.URL || null;
  }

  /**
   * Fetch telemetry data
   */
  async getTelemetry(telemetryUrl: string): Promise<string> {
    const response = await fetch(telemetryUrl, {
      method: 'GET',
      headers: {
        'Accept': 'application/json',
      },
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch telemetry: ${response.status} ${response.statusText}`);
    }

    return response.text();
  }

  /**
   * Get recent matches for a player
   */
  async getRecentMatches(playerId: string, platform: Platform, limit: number = 5): Promise<string[]> {
    const player = await this.getPlayerById(playerId, platform);
    if (!player?.relationships?.matches?.data) return [];

    return player.relationships.matches.data
      .slice(0, limit)
      .map(m => m.id);
  }
}

/**
 * Create a PUBG API client instance
 */
export function createPUBGClient(apiKey: string): PUBGApiClient {
  return new PUBGApiClient(apiKey);
}
