# .github AGENTS.md

## Purpose
Guidance for agents editing GitHub Actions workflows and release automation in this repository.

## Release workflow scope
- Release automation is triggered by pushed tags matching `v*`.
- Supported release targets are macOS and Windows only.
- Release assets must come from Tauri-generated bundle output only.
- The GitHub Release page is the canonical download surface.

## Allowed upload sources
- `src-tauri/target/release/bundle/**`

## Never upload
- API keys
- `.env` files or any local credential files
- application settings
- logs
- databases
- cache files
- telemetry dumps
- local test artifacts
- unbundled build directories outside the intended Tauri bundle outputs

## Forbidden file patterns
- `.env*`
- `**/*.db`
- `**/*.sqlite`
- `**/*.sqlite3`
- `**/*.log`
- `**/logs/**`
- `**/cache/**`
- `**/Library/Application Support/**`
- `**/AppData/**`

## Workflow editing rules
- Prefer `tauri-apps/tauri-action@v0.6.2` for release builds.
- Keep `permissions` minimal and limited to what release publishing requires.
- Do not enable updater JSON uploads unless auto-update support is explicitly added later.
- Do not upload plain binaries unless there is an explicit portable-build requirement.
- Keep asset naming explicit and platform-aware.
