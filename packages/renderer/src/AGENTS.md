# RENDERER PACKAGE KNOWLEDGE BASE

## OVERVIEW
Framework-free renderer UI: one-window desktop dashboard plus preload bridge types.

## STRUCTURE
```text
src/
├── app.ts           # state, routing, loaders, modal actions
├── index.html       # DOM contract
├── styles.css       # visual system
└── preload/         # safe Electron bridge
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Main UI state/routing | `app.ts` | Large hotspot; owns navigation, forms, dashboard refresh |
| Static shell | `index.html` | Dashboard views, modals, semantic IDs |
| Styling | `styles.css` | Dark desktop utility theme |
| Safe bridge | `preload/preload.ts` | `contextBridge` exposure only |
| Renderer-facing API types | `preload/types.ts` | Mirror shared IPC responses |
| Convenience wrapper | `preload/api.ts` | Throws when preload bridge missing |

## CONVENTIONS
- Renderer never talks to Electron directly except through preload-exposed `window.electronAPI`.
- DOM ids/classes are part of the app contract; `app.ts` relies on them heavily.
- Dashboard status combines app sync status and Overwolf status.
- Keep forms modal-driven; add new actions by following existing modal/reset/open flow.
- Accessibility lint matters here: explicit button types, titled SVGs.

## ANTI-PATTERNS
- Do not import main-process modules here.
- Do not duplicate IPC channel names or response shapes here; use preload/shared types.
- Do not scatter state into many globals; extend `AppState` and existing loaders.
- Do not bypass preload for Node/Electron access.

## NOTES
- `app.ts` is the largest hotspot in the repo; prefer extracting helpers before adding more branching.
- `index.html` and `styles.css` are linted for accessibility/markup correctness, not just visual output.
