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
2. `%ProgramData%\zapp\engine\current` (preferred machine-wide pack)
3. Newest pack under `%ProgramData%\zapp\engine\<version>\`
4. `%APPDATA%\app.zapp.desktop\engine\current` (legacy / fallback)
5. Newest pack under `%APPDATA%\app.zapp.desktop\engine\<version>\`
6. Dev: `vendor/zapret`
7. Sibling `zapret-discord-youtube-*` folders

The source repo does **not** ship WinDivert / `winws`. End-user packs are downloaded from [Flowseal/zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube/releases).

## Helper

- Binary: `%LOCALAPPDATA%\zapp\zapp-helper.exe` (copy of the elevated helper)
- Listens on `127.0.0.1:39221`
- Auth: token file `%LOCALAPPDATA%\zapp\helper.token`
- Starts / stops `winws` after the user approved UAC once
- Path allowlist: managed packs under AppData or ProgramData (`bin` or pack-root cwd)
- `InstallEngine`: copies a staged AppData pack into ProgramData and applies ACL

## Update channels

| Channel | Source |
|---------|--------|
| Application | [krwg/zapp](https://github.com/krwg/zapp/releases) |
| Engine | Flowseal GitHub Releases (zip) → stage in AppData, prefer elevated ProgramData install |
| Strategies | Same Flowseal pack (profiles from `general*.bat`) |

## Data on disk

| Path | Contents |
|------|----------|
| `%ProgramData%\zapp\` | Preferred engine packs (ACL: SYSTEM/Admins full, Users RX) |
| `%APPDATA%\app.zapp.desktop\` | Settings, downloads/staging, fallback engine packs, logs |
| `%LOCALAPPDATA%\zapp\` | Helper exe + token |

## Stack

| Layer | Tech |
|-------|------|
| UI | React 19, TypeScript, CSS Modules |
| Shell | Tauri 2 |
| Backend | Rust (`src-tauri`) |
| Browser demo | Mock API (`npm run dev`) |
