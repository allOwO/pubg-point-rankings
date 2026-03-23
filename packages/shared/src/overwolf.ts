/**
 * Overwolf/GEP Types for PUBG Point Rankings
 * 
 * These types define the interface for Overwolf Game Events Provider (GEP) integration.
 * Currently implemented as typed stubs that can be populated by:
 * 1. Actual Overwolf APIs when running in Overwolf environment
 * 2. Manual game state detection via telemetry polling
 * 3. Future GEP integration when properly documented
 */

/**
 * Overwolf/GEP connection status
 */
export interface OverwolfStatus {
  /** Whether Overwolf is running and the app is launched within it */
  isRunning: boolean;
  /** Whether the Game Events Provider is available for PUBG */
  isGEPAvailable: boolean;
  /** Whether PUBG is currently running */
  isPUBGRunning: boolean;
  /** Last error message, if any */
  lastError: string | null;
  /** Current game information, if in a match */
  gameInfo: OverwolfGameInfo | null;
}

/**
 * Game information from Overwolf/GEP
 */
export interface OverwolfGameInfo {
  /** Whether the player is currently in a match */
  isInGame: boolean;
  /** Current match ID, if available */
  matchId: string | null;
  /** Current map name, if available */
  mapName: string | null;
  /** Current game mode, if available */
  gameMode: string | null;
  /** Match start timestamp */
  matchStartTime: Date | null;
  /** Squad/team information */
  squad: OverwolfSquadInfo | null;
}

/**
 * Squad/team information from Overwolf
 */
export interface OverwolfSquadInfo {
  /** Team ID */
  teamId: number;
  /** Squad members */
  members: OverwolfSquadMember[];
}

/**
 * Squad member information
 */
export interface OverwolfSquadMember {
  /** Player name */
  playerName: string;
  /** Account ID */
  accountId: string;
  /** Whether this is the local player */
  isLocalPlayer: boolean;
}

/**
 * Real-time game event from GEP
 */
export interface OverwolfGameEvent {
  /** Event type */
  type: 'kill' | 'death' | 'match_start' | 'match_end' | 'revive' | 'damage' | 'other';
  /** Event timestamp */
  timestamp: Date;
  /** Event data (varies by event type) */
  data: Record<string, unknown>;
}

/**
 * Overwolf API interface (typed stub)
 * 
 * This interface defines the expected shape of Overwolf APIs.
 * Actual implementation would use overwolf.games.events, overwolf.games.launchers, etc.
 * when running in Overwolf Electron environment.
 */
export interface OverwolfAPI {
  /** Check if Overwolf is available */
  isAvailable(): boolean;
  /** Get current status */
  getStatus(): OverwolfStatus;
  /** Register event listener */
  onEvent(callback: (event: OverwolfGameEvent) => void): void;
  /** Unregister event listener */
  offEvent(callback: (event: OverwolfGameEvent) => void): void;
}

/**
 * Stub implementation of Overwolf API
 * Safe placeholder that doesn't use undocumented APIs
 */
export class OverwolfAPIStub implements OverwolfAPI {
  private listeners: Array<(event: OverwolfGameEvent) => void> = [];
  private status: OverwolfStatus;

  constructor() {
    this.status = {
      isRunning: false,
      isGEPAvailable: false,
      isPUBGRunning: false,
      lastError: null,
      gameInfo: null,
    };
  }

  isAvailable(): boolean {
    return 'overwolf' in globalThis;
  }

  getStatus(): OverwolfStatus {
    return { ...this.status };
  }

  onEvent(callback: (event: OverwolfGameEvent) => void): void {
    this.listeners.push(callback);
  }

  offEvent(callback: (event: OverwolfGameEvent) => void): void {
    this.listeners = this.listeners.filter(cb => cb !== callback);
  }

  /**
   * Update status (called by main process or when real events occur)
   */
  updateStatus(updates: Partial<OverwolfStatus>): void {
    this.status = { ...this.status, ...updates };
  }

  /**
   * Emit an event to all listeners
   */
  emitEvent(event: OverwolfGameEvent): void {
    this.listeners.forEach(cb => {
      try {
        cb(event);
      } catch (error) {
        console.error('Error in Overwolf event listener:', error);
      }
    });
  }
}

// Export a singleton instance
export const overwolfStub = new OverwolfAPIStub();
