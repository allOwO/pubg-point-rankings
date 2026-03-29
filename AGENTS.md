# AGENTS.md

## Purpose
Repository guidance for agentic coding tools working in this project.
This app is **Tauri-first**: Rust backend in `src-tauri/`, vanilla TypeScript renderer in `packages/renderer/`, shared contracts in `packages/shared/`, and legacy/reference Node code in `packages/main/`.

## Rule sources checked
- No `.cursor/rules/` directory found.
- No `.cursorrules` file found.
- No `.github/copilot-instructions.md` file found.
- If any of those files are added later, treat them as additional repository instructions.

## Architecture snapshot
```text
pubg-point-rankings/
├── src-tauri/            # Live backend, DB, sync, telemetry, Tauri commands
├── packages/renderer/    # Live frontend (vanilla TS + Vite bundle)
├── packages/shared/      # Shared TS types + Zod schemas + contracts
├── packages/main/        # Legacy/reference TS implementation + tests
├── docs/                 # Product, API, migration, and data notes
└── AGENTS.md             # This file
```

## Runtime ownership
- `src-tauri/` is the production backend.
- `packages/renderer/` is the production UI.
- `packages/shared/` is the single source of truth for cross-layer TS shapes.
- `packages/main/` is **not** the active runtime path; use it for tests, reference behavior, and migration comparison only.

## Build / lint / test commands

### Root commands
```bash
npm install
npm run dev
npm run build
npm run typecheck
npm test
npm run tauri:build
npm run package
```

### Workspace-specific commands
```bash
npm run build --workspace @pubg-point-rankings/shared
npm run build --workspace @pubg-point-rankings/main
npm run build --workspace @pubg-point-rankings/renderer
npm run typecheck --workspace @pubg-point-rankings/renderer
npm run dev --workspace @pubg-point-rankings/renderer
```

### Rust backend commands
```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

### Packaging / icon commands
```bash
./node_modules/.bin/tauri icon "src-tauri/icons/icon.png" -o "src-tauri/icons"
./node_modules/.bin/tauri build --bundles app
```

### Lint reality
- `npm run lint` is a placeholder only.
- Real verification is `npm run typecheck`, `npm run build`, `cargo check`, and `cargo test`.

## Running a single test

### Rust: single test by substring
```bash
cargo test --manifest-path src-tauri/Cargo.toml scheduler
cargo test --manifest-path src-tauri/Cargo.toml parses_and_aggregates_basic_stats
```

### Legacy Node tests: single file / single case
Root `npm test` runs compiled JS from `packages/main/dist`, so build first.
```bash
npm run build --workspace @pubg-point-rankings/main
node --test "packages/main/dist/engine/calculator.test.js"
node --test --test-name-pattern "rounding" "packages/main/dist/**/*.test.js"
```

## Expected verification before finishing
- If Rust changed: run `cargo check --manifest-path src-tauri/Cargo.toml` and `cargo test --manifest-path src-tauri/Cargo.toml`.
- If renderer or shared TS changed: run `npm run typecheck` and `npm run build`.
- If shared contracts changed: verify `packages/shared`, renderer hydration, and Tauri command DTOs together.
- If sync / parsing / DB behavior changed: run all four (`typecheck`, `build`, `cargo check`, `cargo test`).

## Code style guidelines

### General
- Keep changes minimal, local, and aligned with existing structure.
- Extend current layers before inventing new abstractions.
- Avoid mixing points-page UI with matches-page UI; keep page ownership clear.
- Preserve the Tauri-first architecture; do not reintroduce Electron/Overwolf assumptions into active code.

### Imports
- TypeScript: ES module syntax, external imports before workspace imports, use `import type` when possible.
- Rust: group `std` imports first, then external crates, then crate-local modules.
- Remove unused imports immediately.

### Formatting
- TypeScript uses semicolons and single quotes.
- Prefer readable multiline objects/DTOs over dense inline blobs.
- Rust should follow `rustfmt` defaults.
- HTML/Markdown should remain structured and scannable.
- Do not compress complex renderer templates into unreadable one-liners unless it clearly reduces duplication.

### Types
- Prefer explicit types and existing interfaces.
- Never use `any`, `as any`, `@ts-ignore`, or `@ts-expect-error`.
- Keep `packages/shared/src/types.ts` and `packages/shared/src/schemas.ts` aligned.
- Centralize renderer date hydration in `packages/renderer/src/tauri-api.ts`.

### Naming
- TypeScript: `camelCase` for values/functions, `PascalCase` for interfaces/types/classes.
- Rust: `snake_case` for functions/modules, `PascalCase` for structs/enums.
- Tauri commands stay `snake_case`; use `rename_all = "camelCase"` only for argument/response shape bridging.
- DB keys and app setting keys should stay lowercase `snake_case`.

### Error handling
- Rust repository/service APIs should return `Result<_, AppError>`.
- Tauri commands should translate failures into `ErrorPayload`.
- Renderer async actions should `console.error(...)` and show a toast when the user should know about the failure.
- No empty catch blocks, silent fallbacks, or swallowed parse errors.

### Data / domain rules
- Use repositories for DB access; do not put SQL into commands or renderer code.
- Preserve snapshot columns and historical payout semantics.
- User-facing points remain integers.
- Prefer `/matches` + telemetry data for PUBG enrichment work; avoid expanding to other rate-limited APIs unless required.

### Renderer rules
- Treat `index.html` IDs/classes as a hard contract with `app.ts`.
- Keep state in `AppState` and page-specific helpers; avoid ad hoc globals.
- All backend calls go through `tauri-api.ts`.
- Keep points page and matches page separate in markup, i18n keys, and styles.
- Prefer deleting dead modal/DOM paths instead of preserving unused UI.

### Backend rules
- Keep command handlers thin; business logic belongs in services/repositories/parsers.
- Scheduler/runtime logic must not do recent-match checks in `not_running`.
- Process detection must stay ordinary and non-invasive.
- Never add anti-cheat-risky features: memory reading, DLL injection, packet capture, hooks, overlays, or similar.

## Project-specific anti-patterns
- Do not invent cross-layer contracts outside `packages/shared` unless migration demands it.
- Do not bypass `packages/shared/src/types.ts` for shared shapes.
- Do not place business logic in renderer DOM handlers.
- Do not treat `packages/main` as the live backend.
- Do not let renderer page features drift into each other through shared state or shared labels.

## Recent modifications worth remembering
- Match detail/log enrichment now relies primarily on `/matches` and telemetry.
- Match ordering should use `match_end_at DESC` with local DB index support; keep that behavior when editing queries/migrations.
- Recent match sync uses bounded concurrent remote fetching; preserve concurrency limits and sequential DB persistence.
- Match log UI now belongs to the **matches page only**.
- Points page should remain focused on settlement, history, and unsettled-summary behavior only.
- Dashboard sync is now a compact action beside the global ready status, not a full dashboard card.
- Dashboard now shows a read-only unsettled summary plus the current player's latest 10 matches; it no longer owns system-status, quick-actions, friends-summary, or polling sections.
- Dashboard latest-match rows are self-first summaries (kills, damage, assists, revives) with expandable squad rows; keep that lightweight view separate from full match-detail rendering.
- Polling settings now live under the settings page; do not move the polling form back into the dashboard.

## Practical notes for agents
- Read adjacent code before editing pattern-heavy files.
- `packages/renderer/src/app.ts` is large; prefer small helpers or deletion of duplication over piling on branches.
- If you change a shared type, inspect `schemas.ts`, renderer hydration, and related Tauri command DTOs in one pass.
- If you touch sync or telemetry parsing, review parsing tests and DB persistence together.
- If macOS app icon work appears correct but the app still shows a generic icon, suspect local icon cache before changing code again; try refreshing Dock/Finder.
