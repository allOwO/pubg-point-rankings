# PUBG Point Rankings

An Electron-based app for tracking PUBG matches and calculating point rankings for teammates.

## Architecture

This application uses a multi-package architecture:

- **`packages/shared`** - Shared types, IPC contracts, and validation schemas
- **`packages/main`** - Electron main process (backend core, database, sync service)
- **`packages/renderer`** - Electron renderer process (UI, preload scripts)

## Development

### Prerequisites

- Node.js 18+ 
- npm 9+

### Install Dependencies

```bash
npm install
```

### Development Mode

```bash
# Run with hot-reload
npm run dev

# Or run with watch mode for development
npm run dev:watch
```

### Build

```bash
# Build all packages
npm run build

# Build specific packages
npm run build:shared
npm run build:main
npm run build:renderer
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
# Package for current platform
npm run package

# Package for specific platforms
npm run dist:win    # Windows
npm run dist:mac    # macOS
npm run dist:linux  # Linux

# Create portable build (directory, no installer)
npm run package:dir
```

## Application Structure

### Main Process (`packages/main`)

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
- **`renderer.ts`** - Renderer entry script, UI logic
- **`preload/preload.ts`** - Preload script exposing safe API to renderer
- **`preload/types.ts`** - TypeScript types for the exposed API
- **`preload/api.ts`** - Helper functions for accessing the exposed API

### Shared (`packages/shared`)

- **`types.ts`** - Domain types (Teammate, Match, rules, records, etc.)
- **`ipc.ts`** - IPC channel definitions and request/response types
- **`schemas.ts`** - Zod validation schemas
- **`overwolf.ts`** - Overwolf/GEP integration types and stubs

## Configuration

### PUBG API Key

You need a PUBG API key to sync matches. Get one at:
https://developer.pubg.com/

Enter your API key in the application's settings screen.

### Overwolf Integration

The application includes typed stubs for Overwolf GEP integration:

- `packages/shared/src/overwolf.ts` - Type definitions and stub implementation
- `packages/main/src/main/bootstrap.ts` - Status tracking and IPC handlers
- `packages/renderer/src/preload/types.ts` - API types for renderer

When running within the Overwolf environment, these stubs can be populated with actual Overwolf API data.

## Scripts Reference

| Script | Description |
|--------|-------------|
| `npm run dev` | Build and run in development mode |
| `npm run dev:watch` | Run with file watching for development |
| `npm run build` | Build all packages for production |
| `npm run package` | Package the app for distribution |
| `npm run dist` | Build and package for distribution |
| `npm run clean` | Clean all build artifacts |

## Data Storage

The application uses SQLite via better-sqlite3 for local data storage:

- **Windows**: `%LOCALAPPDATA%/pubg-redbag-plugin/app.db` (legacy compatibility path)
- **macOS**: `~/Library/Application Support/pubg-redbag-plugin/app.db` (legacy compatibility path)
- **Linux**: `~/.config/pubg-redbag-plugin/app.db` (legacy compatibility path)

## License

MIT
