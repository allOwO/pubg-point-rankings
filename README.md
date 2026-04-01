# PUBG Point Rankings

A Tauri-based desktop app for tracking PUBG matches and calculating point rankings for teammates.

## Run from source

### Prerequisites

- Node.js 18+
- npm 9+
- Rust toolchain (`cargo`)

### Install

```bash
npm install
```

### Start the app

```bash
npm run dev
```

### Build

```bash
npm run build
npm run tauri:build
```

### Type check

```bash
npm run typecheck
```

### Test

```bash
npm test
```

## Configuration

You need a PUBG API key to sync matches. Get one at https://developer.pubg.com/

Enter your API key in the application's settings screen.

## Release

Push a Git tag matching `v*` to trigger the GitHub Actions release workflow.

Make sure the tag matches the app version in `package.json` and `src-tauri/tauri.conf.json`, so the Release tag and uploaded asset filenames stay aligned.

Example:

```bash
git tag v1.0.0
git push origin v1.0.0
```

The workflow will:

- Build macOS artifacts
- Build Windows artifacts
- Create or update the matching GitHub Release
- Attach downloadable files to the Release page

**Note:** Only build artifacts are uploaded. API keys, app settings, logs, databases, and runtime data are never included.

## License

Sustainable Use License v1.0 (`SUL-1.0`). See [LICENSE](./LICENSE).
