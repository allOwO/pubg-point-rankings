import { EventEmitter } from 'node:events';
import { app, type App, type Event } from 'electron';
import type { OverwolfStatus } from '@pubg-point-rankings/shared';

const STATUS_CHANGED_EVENT = 'status-changed';

const PUBG_FEATURES = [
  'gep_internal',
  'kill',
  'revived',
  'death',
  'killer',
  'match',
  'match_info',
  'rank',
  'counters',
  'location',
  'me',
  'team',
  'phase',
  'map',
  'roster',
];

const NON_MATCH_PHASES = new Set(['lobby', 'loading_screen']);

type OverwolfPackageName = 'gep' | 'overlay' | 'recorder' | 'utility' | string;

interface SupportedGame {
  id: number;
  name: string;
}

interface GepGameLaunchEvent {
  enable(): void;
}

interface GepInfoUpdate {
  feature: string;
  key: string;
  value: unknown;
}

interface GepGameEvent {
  feature: string;
  key: string;
  value: unknown;
}

interface OverwolfGameEventPackage extends NodeJS.EventEmitter {
  getSupportedGames(): Promise<SupportedGame[]>;
  setRequiredFeatures(gameId: number, features: string[] | undefined): Promise<void>;
  getInfo(gameId: number): Promise<unknown>;
  on(eventName: 'new-info-update', listener: (event: Event, gameId: number, data: GepInfoUpdate) => void): this;
  on(eventName: 'new-game-event', listener: (event: Event, gameId: number, data: GepGameEvent) => void): this;
  on(eventName: 'game-detected', listener: (event: GepGameLaunchEvent, gameId: number, name: string) => void): this;
  on(eventName: 'game-exit', listener: (event: Event, gameId: number, gameName: string) => void): this;
  on(eventName: 'elevated-privileges-required', listener: (event: Event, gameId: number, name: string) => void): this;
  on(eventName: 'error', listener: (event: Event, gameId: number, errorMessage: string) => void): this;
}

interface OverwolfPackageManager extends NodeJS.EventEmitter {
  gep: OverwolfGameEventPackage;
  on(eventName: 'ready', listener: (event: Event, packageName: OverwolfPackageName, version: string) => void): this;
  on(eventName: 'failed-to-initialize', listener: (event: Event, packageName: OverwolfPackageName) => void): this;
  on(eventName: 'crashed', listener: (event: Event, canRecover: boolean) => void): this;
}

interface OverwolfApiHost {
  overwolf?: {
    packages: OverwolfPackageManager;
  };
}

type OverwolfApp = App & OverwolfApiHost;

export class OverwolfGepService extends EventEmitter {
  private status: OverwolfStatus = {
    isRunning: false,
    isGEPAvailable: false,
    isPUBGRunning: false,
    lastError: null,
    gameInfo: null,
  };

  private readonly overwolfApp = this.resolveOverwolfApp();
  private pubgGameId: number | null = null;
  private initialized = false;

  async initialize(): Promise<void> {
    if (this.initialized) {
      return;
    }

    this.initialized = true;

    if (!this.overwolfApp) {
      this.setStatus({
        isRunning: false,
        isGEPAvailable: false,
        isPUBGRunning: false,
        lastError: null,
        gameInfo: null,
      });
      return;
    }

    this.registerPackageListeners();
    await this.resolvePubgGame();
    this.bindGepEvents();
    await this.configureGep();
    await this.refreshCurrentInfo();
  }

  getStatus(): OverwolfStatus {
    return {
      ...this.status,
      gameInfo: this.status.gameInfo ? { ...this.status.gameInfo } : null,
    };
  }

  onStatusChanged(listener: (status: OverwolfStatus) => void): void {
    this.on(STATUS_CHANGED_EVENT, listener);
  }

  offStatusChanged(listener: (status: OverwolfStatus) => void): void {
    this.off(STATUS_CHANGED_EVENT, listener);
  }

  private resolveOverwolfApp(): OverwolfApp | null {
    const candidate = app as OverwolfApp;
    return candidate.overwolf?.packages ? candidate : null;
  }

  private registerPackageListeners(): void {
    if (!this.overwolfApp) {
      return;
    }

    const packageManager = this.overwolfApp.overwolf!.packages;

    packageManager.on('ready', (_event: Event, packageName: OverwolfPackageName) => {
      if (packageName === 'gep') {
        void this.configureGep();
      }
    });

    packageManager.on('failed-to-initialize', (_event: Event, packageName: OverwolfPackageName) => {
      if (packageName === 'gep') {
        this.setStatus({
          isRunning: true,
          isGEPAvailable: false,
          lastError: 'Overwolf GEP package failed to initialize',
        });
      }
    });

    packageManager.on('crashed', (_event: Event, canRecover: boolean) => {
      this.setStatus({
        lastError: canRecover
          ? 'An Overwolf package crashed and can recover automatically'
          : 'An Overwolf package crashed and requires manual recovery',
      });
    });
  }

  private async resolvePubgGame(): Promise<void> {
    if (!this.overwolfApp) {
      return;
    }

    const supportedGames = await this.overwolfApp.overwolf!.packages.gep.getSupportedGames();
    const pubgGame = supportedGames.find((game) => /playerunknown|pubg/i.test(game.name));

    this.pubgGameId = pubgGame?.id ?? null;

    this.setStatus({
      isRunning: true,
      isGEPAvailable: Boolean(this.pubgGameId),
      lastError: this.pubgGameId ? null : 'PUBG is not available in the current GEP catalog',
    });
  }

  private bindGepEvents(): void {
    if (!this.overwolfApp) {
      return;
    }

    const gep = this.overwolfApp.overwolf!.packages.gep;

    gep.on('game-detected', (event: GepGameLaunchEvent, gameId: number, name: string) => {
      if (!this.isPubgGame(gameId, name)) {
        return;
      }

      event.enable();
      this.setStatus({
        isRunning: true,
        isGEPAvailable: true,
        isPUBGRunning: true,
        lastError: null,
      });
    });

    gep.on('game-exit', (_event: Event, gameId: number, gameName: string) => {
      if (!this.isPubgGame(gameId, gameName)) {
        return;
      }

      this.setStatus({
        isPUBGRunning: false,
        gameInfo: null,
      });
    });

    gep.on('new-info-update', (_event: Event, gameId: number, data: GepInfoUpdate) => {
      if (!this.isPubgGame(gameId)) {
        return;
      }

      this.applyInfoUpdate(data.feature, data.key, data.value);
    });

    gep.on('new-game-event', (_event: Event, gameId: number, data: GepGameEvent) => {
      if (!this.isPubgGame(gameId)) {
        return;
      }

      this.applyGameEvent(data.feature, data.key, data.value);
    });

    gep.on('elevated-privileges-required', (_event: Event, gameId: number, name: string) => {
      if (!this.isPubgGame(gameId, name)) {
        return;
      }

      this.setStatus({
        isPUBGRunning: true,
        lastError: 'PUBG is running with elevated privileges; run the app as administrator for GEP access',
      });
    });

    gep.on('error', (_event: Event, gameId: number, errorMessage: string) => {
      if (!this.isPubgGame(gameId)) {
        return;
      }

      this.setStatus({
        lastError: errorMessage,
      });
    });
  }

  private async configureGep(): Promise<void> {
    if (!this.overwolfApp || this.pubgGameId === null) {
      return;
    }

    await this.overwolfApp.overwolf!.packages.gep.setRequiredFeatures(this.pubgGameId, PUBG_FEATURES);
    this.setStatus({
      isRunning: true,
      isGEPAvailable: true,
      lastError: null,
    });
  }

  private async refreshCurrentInfo(): Promise<void> {
    if (!this.overwolfApp || this.pubgGameId === null) {
      return;
    }

    const info = await this.overwolfApp.overwolf!.packages.gep.getInfo(this.pubgGameId);
    if (!info || typeof info !== 'object' || !('info' in info)) {
      return;
    }

    const infoRecord = (info as { info: Record<string, Record<string, unknown>> }).info;
    for (const [feature, entries] of Object.entries(infoRecord)) {
      for (const [key, value] of Object.entries(entries)) {
        this.applyInfoUpdate(feature, key, value);
      }
    }
  }

  private isPubgGame(gameId: number, name?: string): boolean {
    if (this.pubgGameId !== null) {
      return gameId === this.pubgGameId;
    }

    return Boolean(name && /playerunknown|pubg/i.test(name));
  }

  private applyGameEvent(feature: string, key: string, _value: unknown): void {
    switch (`${feature}:${key}`) {
      case 'match:matchStart':
        this.setStatus({
          isPUBGRunning: true,
          gameInfo: {
            ...(this.status.gameInfo ?? this.createGameInfo()),
            isInGame: true,
            matchStartTime: new Date(),
          },
          lastError: null,
        });
        break;
      case 'match:matchEnd':
      case 'match:matchSummary':
      case 'death:death':
        this.setStatus({
          gameInfo: {
            ...(this.status.gameInfo ?? this.createGameInfo()),
            isInGame: false,
          },
        });
        break;
      default:
        break;
    }
  }

  private applyInfoUpdate(feature: string, key: string, value: unknown): void {
    const nextGameInfo = {
      ...(this.status.gameInfo ?? this.createGameInfo()),
    };

    switch (`${feature}:${key}`) {
      case 'match_info:match_id':
        nextGameInfo.matchId = this.stringifyValue(value);
        this.setStatus({ isPUBGRunning: true, gameInfo: nextGameInfo, lastError: null });
        break;
      case 'match_info:mode':
        nextGameInfo.gameMode = this.stringifyValue(value);
        this.setStatus({ isPUBGRunning: true, gameInfo: nextGameInfo });
        break;
      case 'match_info:map':
        nextGameInfo.mapName = this.stringifyValue(value);
        this.setStatus({ isPUBGRunning: true, gameInfo: nextGameInfo });
        break;
      case 'game_info:phase': {
        const phase = this.stringifyValue(value);
        nextGameInfo.isInGame = phase !== null && !NON_MATCH_PHASES.has(phase);
        this.setStatus({
          isPUBGRunning: phase !== null,
          gameInfo: nextGameInfo,
        });
        break;
      }
      default:
        break;
    }
  }

  private createGameInfo(): NonNullable<OverwolfStatus['gameInfo']> {
    return {
      isInGame: false,
      matchId: null,
      mapName: null,
      gameMode: null,
      matchStartTime: null,
      squad: null,
    };
  }

  private stringifyValue(value: unknown): string | null {
    if (typeof value === 'string') {
      return value;
    }

    if (typeof value === 'number' || typeof value === 'boolean') {
      return String(value);
    }

    return null;
  }

  private setStatus(updates: Partial<OverwolfStatus>): void {
    this.status = {
      ...this.status,
      ...updates,
      gameInfo: Object.prototype.hasOwnProperty.call(updates, 'gameInfo')
        ? (updates.gameInfo ? { ...updates.gameInfo } : null)
        : this.status.gameInfo,
    };

    this.emit(STATUS_CHANGED_EVENT, this.getStatus());
  }
}
