# PROJECT KNOWLEDGE BASE

**Generated:** 2026-03-24 Asia/Shanghai
**Branch:** unknown

## OVERVIEW
Overwolf Electron monorepo for local PUBG point rankings. Electron main owns SQLite + PUBG sync + GEP; renderer is a vanilla TS desktop UI; shared package is the contract hub.

## STRUCTURE
```text
pubg-redbag-plugin/
├── packages/main/src/      # Electron main, DB, sync, GEP, IPC
├── packages/renderer/src/  # UI, preload bridge, HTML/CSS shell
├── packages/shared/src/    # Types, schemas, IPC channels, Overwolf status types
├── docs/                   # API/setup + product plan
└── electron-builder.config.js  # packaging rules
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| App bootstrap | `packages/main/src/main/bootstrap.ts` | Wires DB, IPC, GEP, window lifecycle |
| Packaging/runtime | `package.json`, `electron-builder.config.js` | `ow-electron` + builder config live here |
| Production bundles | `packages/main/bundle/`, `packages/renderer/bundle/` | Packaging must use bundle outputs, not raw `dist/` |
| PUBG sync flow | `packages/main/src/services/sync.ts` | End-to-end match ingestion |
| GEP integration | `packages/main/src/services/overwolf.ts` | Real Overwolf package integration |
| DB schema/migrations | `packages/main/src/db/` | SQLite bootstrap + schema versioning |
| UI logic | `packages/renderer/src/app.ts` | Single large renderer state/view module |
| Preload bridge | `packages/renderer/src/preload/` | Sole renderer ↔ main bridge |
| Cross-process contract | `packages/shared/src/ipc.ts` | Channel names + request/response shapes |
| Core domain model | `packages/shared/src/types.ts`, `schemas.ts` | Type + Zod pairing |

## CONVENTIONS
- npm workspaces; build order is `shared -> main -> renderer`.
- Root `npm run build` must stay `tsc -b` + bundle generation; do not revert to ad-hoc per-package build ordering.
- User-facing scores are integer points. Legacy DB columns still use cent/redbag names internally for compatibility; do not introduce float score state.
- Renderer must go through preload + IPC. No direct DB/fs/main imports.
- `shared/src` is authoritative for domain shapes and IPC contracts.
- Historical rows use snapshot fields to preserve past rule/teammate state.
- Main package tests live in `packages/main/src/**/*.test.ts`; root test runs compiled tests from `packages/main/dist`.
- Development uses `packages/*/dist`; production packaging uses `packages/main/bundle` and `packages/renderer/bundle`.
- Main bundle must externalize native/runtime-sensitive modules such as `electron` and `better-sqlite3`.
- Renderer production output consists of bundled `app.js`, bundled preload, plus copied `index.html` and `styles.css`.

## ANTI-PATTERNS (THIS PROJECT)
- Do not invent IPC channel strings outside `packages/shared/src/ipc.ts`.
- Do not treat GEP damage/kills as payout truth; final calculation stays telemetry/API-driven.
- Do not put business logic in renderer or preload.
- Do not bypass repositories for routine DB access from handlers/services.
- Do not store mutable historical data without snapshot columns.
- Do not use memory reading, DLL injection, packet capture, game hooks, or any unsupported runtime inspection to obtain PUBG real-time data.
- Do not perform any operation that could risk player accounts, trigger anti-cheat, or violate PUBG/KRAFTON/Overwolf policies; use sanctioned APIs/GEP only.
- Do not point production packaging back at `packages/*/dist/**`; packaged builds must consume bundle outputs to preserve size optimizations.
- Do not broaden `asarUnpack` back to whole native package directories; keep unpacking as narrow as possible.
- Do not add Windows `portable` target, UPX compression, custom runtime trimming, or other anti-cheat-sensitive packaging tricks without explicit review.
- Do not add extra locales/resources/assets to the package unless they are required and size impact is understood.

## UNIQUE STYLES
- Backend is layered: `db -> repository -> services/engine/parser/pubg -> ipc/main`.
- GEP status is pushed from main to renderer via `overwolf:statusChanged` event.
- The renderer is intentionally framework-free; `app.ts` owns state, view routing, forms, and dashboard refresh.
- `packages/shared/src/overwolf.ts` models Overwolf status even when runtime APIs are absent.

## COMMANDS
```bash
npm install
npm run dev
npm run build
npm run typecheck
npm test
npm run package:dir
npm run dist:win
```

## NOTES
- Windows is the real runtime target for GEP/overlay behavior; macOS validation is build-only.
- `package:dir` rebuilds native deps (`better-sqlite3`) against `ow-electron`.
- Package size is dominated by the `ow-electron` runtime, not project code.
- Current packaging optimizations that must be preserved:
  - root build uses `tsc -b` before bundling
  - production entry resolves to `packages/main/bundle/index.js`
  - packaged renderer paths resolve to `packages/renderer/bundle/**`
  - `electron-builder.config.js` includes bundle outputs and excludes raw `dist/`, tests, maps, sources, and native build junk
  - `asarUnpack` is limited to the native `.node` binary path for `better-sqlite3`
  - Windows distribution is NSIS-only with `compression: 'maximum'`
