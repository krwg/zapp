# Architecture

```
UI (React)
  → ZappApi (tauriClient | mockClient)
  → Rust commands
      → AppState
      → ManagedEngine        # starts/stops hidden winws
      → elevated helper      # one UAC, then silent local control
      → UpdateManager        # Application / Engine / Strategies
      → Diagnostics          # Smart check probes
```

## Engine resolve order

1. `ZAPP_ENGINE_DIR` (or legacy `HAPP_ZAPRET_ENGINE_DIR`)
2. `%APPDATA%\app.zapp.desktop\engine\current`
3. Newest pack under `%APPDATA%\app.zapp.desktop\engine\<version>\`
4. Dev: `vendor/zapret`
5. Sibling `zapret-discord-youtube-*` folders

The source repo does **not** ship WinDivert / `winws`. End-user packs are downloaded from [Flowseal/zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube/releases).

## Helper

- Binary: `%LOCALAPPDATA%\zapp\zapp-helper.exe` (copy of the elevated helper)
- Listens on `127.0.0.1:39221`
- Auth: token file `%LOCALAPPDATA%\zapp\helper.token`
- Starts / stops `winws` after the user approved UAC once

## Update channels

| Channel | Source |
|---------|--------|
| Application | [krwg/zapp](https://github.com/krwg/zapp/releases) |
| Engine | Flowseal GitHub Releases (zip) |
| Strategies | Same Flowseal pack (profiles from `general*.bat`) |

## Data on disk

| Path | Contents |
|------|----------|
| `%APPDATA%\app.zapp.desktop\` | Settings, engine packs, downloads, logs |
| `%LOCALAPPDATA%\zapp\` | Helper exe + token |

## Stack

| Layer | Tech |
|-------|------|
| UI | React 19, TypeScript, CSS Modules |
| Shell | Tauri 2 |
| Backend | Rust (`src-tauri`) |
| Browser demo | Mock API (`npm run dev`) |
