# SHARED PACKAGE KNOWLEDGE BASE

## OVERVIEW
Cross-package contract layer: domain models, runtime schemas, legacy IPC channel map, Overwolf status types.

## STRUCTURE
```text
src/
├── types.ts     # compile-time domain model
├── schemas.ts   # runtime validation/input contracts
├── ipc.ts       # legacy IPC channel map (reference only during migration)
└── overwolf.ts  # shared GEP/status types
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Core domain entities | `types.ts` | Matches, players, rules, sync status |
| Runtime validation | `schemas.ts` | Zod schemas + input types |
| Legacy IPC reference | `ipc.ts` | Channel names and request/response map (legacy) |
| Overwolf contract | `overwolf.ts` | Shared GEP/status model |
| Barrel export | `index.ts` | Public surface of the package |

## CONVENTIONS
- Keep compile-time type + runtime schema pairs aligned.
- `ipc.ts` is **legacy reference** from Electron migration; Tauri commands use snake_case names directly. Keep for reference but do not treat as the active command contract.
- Shared package stays dependency-light; only generic libs belong here (`zod` is fine).
- This package must remain process-agnostic: no Tauri, Electron main, BrowserWindow, DB, or DOM code.

## ANTI-PATTERNS
- Do not import from `packages/main` or `packages/renderer`.
- Do not add app-specific side effects or runtime initialization here.
- Do not change a type without checking matching schema and IPC payload definitions.
- Do not assume `ipc.ts` channel names match current Tauri command names exactly.

## NOTES
- This package is the safest place to start when adding a new cross-process feature.
- If a renderer/backend disagreement appears, fix `shared` first, then adapt both sides.
- When porting to Rust, `ipc.ts` serves as a historical reference for the expected payloads.
