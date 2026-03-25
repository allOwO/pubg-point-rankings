# Game Process State Module Plan

## Goal

Implement a standalone game-process state module for the current Tauri app so the app can:

1. detect whether PUBG is running,
2. maintain three runtime states,
3. adjust sync polling cadence based on those states,
4. expose the state to the renderer.

## Current Repo Facts

- `src-tauri/` already exists and is the active desktop host.
- `src-tauri/src/app_state.rs` currently stores DB connection, DB path, and app version only.
- `src-tauri/src/commands/app.rs` exposes `app_get_status`.
- `src-tauri/src/commands/sync.rs` exposes sync status/start commands, but sync orchestration is still mostly stubbed.
- `packages/renderer/src/tauri-api.ts` is the typed invoke adapter.
- `packages/renderer/src/app.ts` already renders dashboard runtime status and sync status.

## Module Boundaries

Add these standalone modules:

- `src-tauri/src/runtime/mod.rs`
- `src-tauri/src/runtime/game_state.rs`
- `src-tauri/src/runtime/scheduler.rs`
- `src-tauri/src/platform/mod.rs`
- `src-tauri/src/platform/process.rs`

Responsibilities:

- `game_state.rs`: pure state machine and transition helpers
- `scheduler.rs`: background loop and cadence decisions
- `process.rs`: ordinary process enumeration only

## State Model

States:

- `NotRunning`
- `Running`
- `CooldownPolling`

Tracked timestamps:

- `last_seen_running_at`
- `cooldown_started_at`
- `last_process_check_at`
- `last_recent_match_check_at`

Core rules:

- `NotRunning -> Running` when PUBG process appears
- `Running -> CooldownPolling` when PUBG process disappears
- `CooldownPolling -> Running` if PUBG reappears during cooldown
- `CooldownPolling -> NotRunning` after 40 minutes with no PUBG process

## Implementation Plan

1. Add runtime/platform modules and pure state machine tests.
   - QA: `cargo test --manifest-path src-tauri/Cargo.toml game_state`
   - Expected: transition and cooldown tests pass.
2. Extend `AppState` with runtime state protected by `Mutex`/`RwLock`.
   - QA: `cargo check --manifest-path src-tauri/Cargo.toml`
   - Expected: app state compiles with new runtime fields.
3. Implement Windows process detection with safe ordinary process enumeration only.
   - QA: `cargo test --manifest-path src-tauri/Cargo.toml process`
   - Expected: process module tests pass; non-Windows path returns safe fallback.
4. Add background scheduler loop from Tauri startup.
   - QA: `cargo check --manifest-path src-tauri/Cargo.toml`
   - Expected: bootstrap compiles and scheduler starts without type or lifetime errors.
5. Make scheduler update game runtime state and derive polling mode.
   - QA: `cargo test --manifest-path src-tauri/Cargo.toml scheduler`
   - Expected: cadence decisions and cooldown expiry behavior pass tests.
6. Add `app_get_game_process_status` command.
   - QA: `cargo check --manifest-path src-tauri/Cargo.toml`
   - Expected: command is registered and serializable.
7. Expose the command in `packages/renderer/src/tauri-api.ts`.
   - QA: `npm run build`
   - Expected: renderer types and invoke adapter compile.
8. Update `packages/renderer/src/app.ts` to render game process state.
   - QA: `npm run build`
   - Expected: dashboard compiles and reads the new API without TS errors.
9. Keep recent-match sync triggering minimal if full sync path is still not ready, but wire the cadence logic and public status now.
   - QA: `cargo check --manifest-path src-tauri/Cargo.toml && npm run build`
   - Expected: backend and renderer integrate without breaking existing commands.

## Constraints

- No memory reading, injection, hooks, packet capture, or anti-cheat-risk behavior.
- Process detection must use only normal OS process listing.
- Non-Windows behavior should fail safe and report not running.
- Keep changes minimal and aligned with current Tauri structure.

## Acceptance Criteria

- A standalone runtime module exists for process state.
- Unit tests cover the three-state transitions and cooldown expiry.
- Tauri app exposes current game process state via command.
- Renderer can display game state without crashing when backend state changes.
- Build passes.

## Final Verification

1. `cargo test --manifest-path src-tauri/Cargo.toml`
   - Expected: Rust unit tests pass.
2. `npm run build`
   - Expected: workspace build passes.
3. Manual runtime check via `npm run tauri:dev`
   - Expected: app launches and `app_get_game_process_status` returns a non-crashing payload on the current platform.
