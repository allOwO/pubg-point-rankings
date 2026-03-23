# MAIN PACKAGE KNOWLEDGE BASE

## OVERVIEW
Electron main backend: SQLite persistence, PUBG API sync, GEP integration, IPC surface, app bootstrap.

## STRUCTURE
```text
src/
├── db/          # connection, schema, migrations
├── repository/  # row mapping + persistence API
├── services/    # sync orchestration, GEP service
├── ipc/         # main-process invoke handlers
├── parser/      # telemetry parsing + stat aggregation
├── pubg/        # official PUBG API client
├── engine/      # redbag calculation logic
└── main/        # bootstrap + BrowserWindow lifecycle
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| App startup | `main/bootstrap.ts` | Initializes DB, defaults, sync, GEP, window |
| BrowserWindow paths | `main/window.ts` | Preload/html resolution and window policy |
| IPC registration | `ipc/handlers.ts` | Thin adapters with shared contracts |
| GEP package handling | `services/overwolf.ts` | `setRequiredFeatures`, status events, PUBG detection |
| Match sync | `services/sync.ts` | API fetch -> telemetry parse -> calculate -> persist |
| SQLite bootstrap | `db/connection.ts`, `db/migrations.ts` | WAL mode + migrations |
| Repository rules | `repository/*.ts` | snake_case row mapping, transactions, snapshots |

## CONVENTIONS
- Services orchestrate; repositories persist; parsers/calculators stay pure.
- Multi-table writes use DB transactions.
- Repository mappers convert snake_case DB rows to camelCase domain models, including `Date` and `0/1 -> boolean` conversion.
- `MatchesRepository.resetSyncingMatches()` implements startup recovery semantics: recent syncs retry, stale ones fail.
- `SyncService` owns runtime sync state and concurrency guard.

## ANTI-PATTERNS
- Do not add SQL directly to IPC handlers when a repository should own it.
- Do not move sync rules into renderer/preload.
- Do not write money as floats or strings.
- Do not skip snapshot fields when persisting historical payouts.
- Do not assume GEP is available on non-Windows or outside Overwolf runtime.
