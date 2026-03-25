# AGENTS.md

## Purpose
Guidance for coding agents working in this repository.
This repo is **Tauri-first**: Rust backend in `src-tauri/`, vanilla TS renderer in `packages/renderer/`, shared contracts in `packages/shared/`, and legacy TS reference code in `packages/main/`.

## Architecture snapshot
```text
pubg-point-rankings/
├── src-tauri/            # Active Rust backend + Tauri host
├── packages/renderer/    # Active frontend (vanilla TS + Vite)
├── packages/shared/      # Shared TS types + Zod schemas
├── packages/main/        # Legacy TS backend reference/tests
├── docs/                 # Product + migration docs
└── README.md
```

### Active vs legacy
- `src-tauri/` is the live backend.
- `packages/renderer/` is the live UI.
- `packages/shared/` is the shared contract layer.
- `packages/main/` is migration/reference code; do not treat it as the runtime path unless explicitly porting behavior.

## Build / lint / test commands
### Root
```bash
npm install
npm run dev
npm run build
npm run typecheck
npm test
npm run tauri:build
npm run package
```

### Package-specific
```bash
npm run build --workspace @pubg-point-rankings/shared
npm run build --workspace @pubg-point-rankings/main
npm run build --workspace @pubg-point-rankings/renderer
npm run typecheck --workspace @pubg-point-rankings/renderer
```

### Rust backend
```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

### Lint status
- `npm run lint` is a placeholder only.
- Real verification is `npm run typecheck`, `cargo check`, `cargo test`, and `npm run build`.

## Running a single test
### Rust
`cargo test` supports substring filtering:
```bash
cargo test --manifest-path src-tauri/Cargo.toml scheduler
cargo test --manifest-path src-tauri/Cargo.toml parses_and_aggregates_basic_stats
```

### Legacy Node tests
Root tests run compiled JS from `packages/main/dist`, so build first:
```bash
npm run build --workspace @pubg-point-rankings/main
node --test "packages/main/dist/engine/calculator.test.js"
node --test --test-name-pattern "rounding" "packages/main/dist/**/*.test.js"
```

## Expected verification before finishing
- If Rust changed: run `cargo check --manifest-path src-tauri/Cargo.toml` and `cargo test --manifest-path src-tauri/Cargo.toml`
- If renderer/shared TS changed: run `npm run typecheck` and `npm run build`
- If cross-layer behavior changed: run all four

## Code style guidelines
### General
- Keep changes minimal and aligned with existing patterns.
- Prefer extending the current layers over inventing new ones.
- Preserve the Tauri-first architecture.
- Do not reintroduce Electron/Overwolf runtime assumptions into active code.

### Imports
- TypeScript: ES imports, external before workspace imports, use `import type` where appropriate.
- Rust: group `std` imports first, then crate/external imports.
- Remove unused imports.

### Formatting
- TypeScript uses semicolons and single quotes.
- Keep object literals/DTOs readable and vertically aligned.
- Rust should follow `rustfmt` defaults and existing `serde` style.
- Keep Markdown/HTML readable; avoid collapsing structured markup into one-liners.

### Types
- Prefer explicit types and existing interfaces.
- Never use `any`, `as any`, `@ts-ignore`, or `@ts-expect-error`.
- Keep `packages/shared/src/types.ts` and `schemas.ts` aligned.
- In renderer adapters, hydrate string dates to `Date` centrally rather than scattering conversions.

### Naming
- TypeScript: `camelCase` for values/functions, `PascalCase` for interfaces/types.
- Rust: `snake_case` for functions/modules, `PascalCase` for structs/enums.
- Tauri commands stay `snake_case`; use `rename_all = "camelCase"` only for argument shape needs.
- DB/app setting keys use lowercase `snake_case` strings.

### Error handling
- Rust repositories/services should return `Result<_, AppError>`.
- Tauri commands should convert backend failures into `ErrorPayload`.
- Renderer async flows should log errors and show a user-visible toast when appropriate.
- Do not swallow errors and do not use empty catch blocks.

### Data / domain rules
- Use repositories for routine DB access; avoid raw SQL in commands/UI.
- Preserve snapshot columns and historical semantics.
- User-facing points remain **integers**; do not turn scoring state into floats.

### Renderer rules
- Treat `index.html` IDs/classes as a hard contract with `app.ts`.
- Keep UI state in the existing `AppState` flow rather than adding ad hoc globals.
- Renderer should go through `tauri-api.ts`; do not spread direct Tauri calls everywhere.
- Do not import backend-only modules into renderer.

### Backend rules
- Keep command handlers thin; business logic belongs in services/runtime/repository layers.
- Keep process detection ordinary and non-invasive.
- Never add memory reading, DLL injection, packet capture, hooks, or anything anti-cheat-risky.
- Scheduler/runtime code must not perform recent-match checks in `not_running`.

## Project-specific anti-patterns
- Do not invent new cross-layer contracts outside `packages/shared` unless migration requires it.
- Do not bypass `packages/shared/src/types.ts` for shared shapes.
- Do not put business logic into renderer DOM handlers.
- Do not rewire packaging away from current Tauri entrypoints.
- Do not assume `packages/main` is the live backend.

## Practical notes for agents
- Read adjacent code before changing pattern-heavy files.
- `packages/renderer/src/app.ts` is large; prefer helper extraction over adding more branching.
- When porting logic from `packages/main` to Rust, preserve behavior first and refactor second.
- If you change a shared type, inspect `schemas.ts`, renderer hydration, and related command DTOs in the same pass.
- If you change sync or polling behavior, review scheduler state transitions and UI status display together.
