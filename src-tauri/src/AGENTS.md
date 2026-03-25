# TAURI BACKEND KNOWLEDGE BASE

## OVERVIEW
Tauri 2 + Rust backend: SQLite persistence, PUBG API sync, telemetry parsing, points calculation, game process detection, and background scheduling.

## STRUCTURE
```text
src/
├── lib.rs           # module declarations, Tauri builder, command registration
├── main.rs          # binary entry point
├── app_state.rs     # global state: DB, game process runtime, sync status
├── commands/        # Tauri command handlers (thin adapters)
├── repository/      # SQLite data access layer
├── services/        # sync orchestration, polling logic
├── runtime/         # scheduler + game process state machine
├── platform/        # process detection (PUBG executable)
├── db/              # connection, schema, migrations
├── dto/             # response DTOs for commands
├── pubg/            # PUBG official API client
├── parser/          # telemetry parsing + stat aggregation
├── engine/          # points calculation logic
└── error.rs         # AppError enum
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Command registration | `lib.rs` | All Tauri commands registered in `invoke_handler` |
| Global state | `app_state.rs` | DB connection, game process runtime, sync status |
| IPC handlers | `commands/*.rs` | Thin adapters; call services/repositories |
| Data access | `repository/*.rs` | snake_case row mapping, transactions, snapshots |
| Match sync flow | `services/sync.rs` | API fetch -> telemetry parse -> calculate -> persist |
| Background scheduler | `runtime/scheduler.rs` | Polls game process, triggers cooldown sync |
| Game process state | `runtime/game_state.rs` | State machine: not_running, running, cooldown_polling |
| Process detection | `platform/process.rs` | Platform-specific PUBG executable detection |
| DB bootstrap | `db/connection.rs`, `db/migrations.rs` | Path resolution, schema, migrations |
| Points calculation | `engine/calculator.rs` | Pure calculation from match stats |
| Telemetry parsing | `parser/telemetry.rs` | Extract kills, damage, revives from telemetry |

## CONVENTIONS
- Commands are thin; business logic lives in services/repositories.
- Repositories return `Result<_, AppError>` and handle snake_case <-> camelCase mapping.
- Multi-table writes use DB transactions.
- Scheduler must not perform recent-match checks in `not_running` state.
- Process detection stays non-invasive; no memory reading or DLL injection.

## ANTI-PATTERNS
- Do not put business logic in command handlers.
- Do not add SQL directly to commands when a repository should own it.
- Do not use floats for user-facing points.
- Do not skip snapshot fields when persisting historical point records.
- Do not add anti-cheat-risky code (memory reading, hooks, packet capture).

## NOTES
- New commands must be registered in `lib.rs` invoke_handler.
- When adding a new command, check `dto/` for existing response shapes before creating new ones.
- Tests live alongside source files; run with `cargo test --manifest-path src-tauri/Cargo.toml`.
