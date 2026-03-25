# RENDERER PACKAGE KNOWLEDGE BASE

## OVERVIEW
Framework-free Tauri renderer UI: one-window desktop dashboard with vanilla TypeScript. All backend communication goes through `tauri-api.ts`.

## STRUCTURE
```text
src/
├── app.ts           # state, routing, loaders, modal actions
├── tauri-api.ts     # typed Tauri invoke adapter + date hydration
├── index.html       # DOM contract
└── styles.css       # visual system
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Main UI state/routing | `app.ts` | Large hotspot; owns navigation, forms, dashboard refresh |
| Backend API calls | `tauri-api.ts` | Typed invoke adapter, DTO hydration (string -> Date) |
| Static shell | `index.html` | Dashboard views, modals, semantic IDs |
| Styling | `styles.css` | Dark desktop utility theme |

## CONVENTIONS
- All Tauri invoke calls go through `tauri-api.ts`; do not import `invoke` directly elsewhere.
- `tauri-api.ts` exports `getAPI()` returning a typed `AppAPIClient` with settings, accounts, teammates, rules, matches, points, sync, and app namespaces.
- DTOs from Tauri use string dates; hydration functions in `tauri-api.ts` convert to `Date` objects centrally.
- DOM ids/classes are part of the app contract; `app.ts` relies on them heavily.
- Keep forms modal-driven; add new actions by following existing modal/reset/open flow.
- Accessibility lint matters here: explicit button types, titled SVGs.

## ANTI-PATTERNS
- Do not import main-process modules here.
- Do not use `invoke` from `@tauri-apps/api/core` outside of `tauri-api.ts`.
- Do not duplicate command names or response shapes here; use shared types.
- Do not scatter state into many globals; extend `AppState` and existing loaders.
- Do not handle date hydration outside `tauri-api.ts`.

## NOTES
- `app.ts` is the largest hotspot in the repo; prefer extracting helpers before adding more branching.
- `index.html` and `styles.css` are linted for accessibility/markup correctness, not just visual output.
- When adding a new backend command, update `tauri-api.ts` first: add DTO interface, hydrate function, and API method.
