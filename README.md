# PUBG Point Rankings

A Tauri-based desktop app for tracking PUBG matches and calculating point rankings for teammates.

## Architecture

This application uses a multi-package architecture:

- **`packages/shared`** - Shared types, IPC contracts, and validation schemas
- **`packages/main`** - Legacy TypeScript backend reference/tests during Rust migration
- **`packages/renderer`** - Tauri renderer process (UI)
- **`src-tauri`** - Tauri 2 + Rust host/backend

## Development

### Prerequisites

- Node.js 18+ 
- npm 9+
- Rust toolchain (`cargo`)

### Install Dependencies

```bash
npm install
```

### Development Mode

```bash
# Run the macOS Tauri app
npm run dev

# Frontend-only Vite dev server
npm run dev:watch
```

### Build

```bash
# Build TypeScript packages and renderer assets
npm run build

# Build specific packages
npm run build:shared
npm run build:main
npm run build:renderer

# Build the Tauri app
npm run tauri:build
```

### Type Checking

```bash
npm run typecheck
```

### Testing

```bash
npm test
```

### Packaging

```bash
# Package current platform via Tauri
npm run package

npm run dist:mac    # macOS
```

## Application Structure

### Legacy TS Backend Reference (`packages/main`)

- **`main/bootstrap.ts`** - Application initialization, database setup, window creation
- **`main/window.ts`** - BrowserWindow management
- **`ipc/handlers.ts`** - IPC request handlers
- **`db/`** - Database connection and migrations
- **`repository/`** - Data access layer
- **`services/`** - Business logic (sync, calculations)
- **`pubg/`** - PUBG API client
- **`parser/`** - Telemetry parsing
- **`engine/`** - Points calculation engine

### Renderer Process (`packages/renderer`)

- **`index.html`** - Main application HTML
- **`app.ts`** - Renderer entry script, UI logic
- **`tauri-api.ts`** - Typed Tauri invoke adapter and date hydration layer

### Tauri Host (`src-tauri`)

- **`src/lib.rs`** - Tauri app bootstrap and command registration
- **`src/commands/`** - Tauri command handlers
- **`src/db/`** - SQLite path resolution, schema, migrations
- **`src/repository/`** - Rust-side data access layer

### Shared (`packages/shared`)

- **`types.ts`** - Domain types (Teammate, Match, rules, records, etc.)
- **`ipc.ts`** - Legacy IPC contract reference during migration
- **`schemas.ts`** - Zod validation schemas
- **`overwolf.ts`** - Overwolf/GEP integration types and stubs

## Configuration

### PUBG API Key

You need a PUBG API key to sync matches. Get one at:
https://developer.pubg.com/

Enter your API key in the application's settings screen.

### Tauri-first migration note

Electron runtime entry scripts were removed from the root workflow. The remaining `packages/main` code stays only as a migration reference until Rust commands fully replace it.

## Scripts Reference

| Script | Description |
|--------|-------------|
| `npm run dev` | Run Tauri + Vite in development |
| `npm run dev:watch` | Run the Vite dev server only |
| `npm run build` | Build TypeScript packages and renderer assets |
| `npm run package` | Build the Tauri app |
| `npm run dist:mac` | Build the macOS app bundle |
| `npm run clean` | Clean all build artifacts |

## Data Storage

The application uses SQLite via better-sqlite3 for local data storage:

- **Windows**: `%LOCALAPPDATA%/pubg-point-rankings/app.db`
- **macOS**: `~/Library/Application Support/pubg-point-rankings/app.db`
- **Linux**: `~/.config/pubg-point-rankings/app.db`

## License

MIT
