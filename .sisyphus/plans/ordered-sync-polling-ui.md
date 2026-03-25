# Ordered Sync / Polling / Manual Trigger Plan

## Goal

Complete the next three implementation steps in order:

1. connect the game-process runtime to automatic recent-match checking,
2. add polling configuration settings in backend + UI,
3. make the dashboard quick action perform an immediate recent-match check.

## Current Repo Facts

- Tauri backend exists under `src-tauri/src`.
- Current game-process runtime exists under `src-tauri/src/runtime/*` and only updates state/cadence.
- `src-tauri/src/commands/sync.rs` is still stubbed.
- No Rust PUBG API client / telemetry parser / points engine exists yet in `src-tauri`.
- Legacy implementation exists in TypeScript under:
  - `packages/main/src/services/sync.ts`
  - `packages/main/src/pubg/client.ts`
  - `packages/main/src/parser/telemetry.ts`
  - `packages/main/src/engine/calculator.ts`
- Renderer already uses Tauri through `packages/renderer/src/tauri-api.ts`.
- Dashboard quick action `btn-sync-now` currently opens the manual sync modal instead of immediately checking the latest match.

## Scope

### Step 1 — Auto recent-match checking

- Add minimal Rust modules needed to support recent-match syncing:
  - `src-tauri/src/pubg/*`
  - `src-tauri/src/parser/*`
  - `src-tauri/src/engine/*`
  - `src-tauri/src/services/*`
- Implement Rust-side recent-match sync orchestration equivalent to the existing TypeScript `syncRecentMatch()` and `syncMatch()` behavior.
- Keep the implementation focused on current product needs:
  - player lookup
  - recent match ID lookup
  - match fetch
  - telemetry fetch/parse
  - points calculation
  - DB persistence through existing repositories
- Extend the background scheduler to trigger recent-match checks only in:
  - `running`
  - `cooldown_polling`
- Keep `not_running` as process-monitor-only.

### Step 2 — Polling settings

- Add persistent app settings keys for:
  - auto recent match checking enabled
  - running polling interval
  - not-running process check interval
  - cooldown polling interval
  - cooldown window minutes
  - first retry delay / retry limit as needed by the current runtime
- Add typed parsing helpers on the Rust side so invalid values fall back safely.
- Add renderer settings UI so users can view/change these values.

### Step 3 — Dashboard immediate check button

- Change the dashboard quick action to run immediate latest-match checking directly.
- Keep the existing match-ID sync modal available separately.
- Disable the immediate-check button while a sync is in progress.
- Refresh dashboard/sync state after completion.

## Constraints

- No memory reading, injection, hooks, DLLs, packet capture, or anti-cheat-risk behavior.
- Process detection must remain ordinary OS process enumeration only.
- No type suppressions.
- No commit.
- Keep changes limited to the Tauri path plus the renderer files needed for settings/button wiring.

## Implementation Tasks

1. Add Rust PUBG client + telemetry parser + points engine modules.
   - QA: `cargo test --manifest-path src-tauri/Cargo.toml`
   - Expected: new module tests pass.
2. Add Rust sync service and replace stubbed `sync_start` / `sync_start_match` behavior with working logic.
   - QA: `cargo test --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml`
   - Expected: sync commands compile and tests pass.
3. Integrate scheduler with the real sync service for automatic recent-match checks in `running` and `cooldown_polling` only.
   - QA: `cargo test --manifest-path src-tauri/Cargo.toml scheduler`
   - Expected: scheduler tests pass with no `not_running` recent-match checks.
4. Add backend settings defaults and parsing helpers for polling configuration.
   - QA: `cargo test --manifest-path src-tauri/Cargo.toml && npm run typecheck`
   - Expected: DB defaults and settings command usage compile cleanly.
5. Add renderer settings controls and adapter methods for the new polling settings.
   - QA: `npm run typecheck && npm run build`
   - Expected: renderer compiles with no TS errors.
6. Update dashboard quick action to perform immediate recent-match checking and refresh UI status.
   - QA: `npm run typecheck && npm run build`
   - Expected: dashboard compiles and button flow is wired.

## Acceptance Criteria

- Automatic recent-match checking really runs from the scheduler in `running` and `cooldown_polling`.
- `not_running` never performs high-frequency recent-match checks.
- `sync_start` checks the latest match successfully when settings are valid.
- `sync_start_match` still supports explicit match sync.
- Polling settings are persisted in the DB and editable from the renderer.
- Dashboard quick action triggers immediate latest-match checking instead of only opening the modal.
- Existing build and typecheck remain green.

## Final Verification

1. `cargo test --manifest-path src-tauri/Cargo.toml`
   - Expected: Rust tests pass.
2. `cargo check --manifest-path src-tauri/Cargo.toml`
   - Expected: Rust backend compiles.
3. `npm run typecheck`
   - Expected: TypeScript passes.
4. `npm run build`
   - Expected: workspace build passes.
