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
- IPC: Windows named pipe `\\.\pipe\zapp-helper` (no TCP listen; token still required)
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
| `%APPDATA%\app.zapp.desktop\` | Settings (`settings.json`), session (`session.json` bypass desired), downloads/staging, fallback engine packs, logs |
| `%LOCALAPPDATA%\zapp\` | Helper exe + token |

## Bypass restore (roadmap §5) + network memory (§7)

- `session.json` remembers `bypassDesired` + active strategy (set on Start, cleared on Stop / tray Quit).
- After engine bootstrap at login: if `restoreBypassOnLogin` → `ensure_helper` + start.
- Background wall-clock watcher: gap &gt; 25s ≈ sleep resume → if `restoreBypassOnResume` → start again.
- Network fingerprint = default IPv4 gateway + WLAN SSID; on change (if `restoreBypassOnNetworkChange`) restore bypass.

## Smart check probes (§6)

- Critical HTTPS (YouTube / Discord gateway / discord.media TCP alts).
- Soft: ytimg, youtubei.googleapis.com, **googlevideo.com**, Discord CDN, STUN UDP, **QUIC UDP:443** smoke (reply-based, does not alone fail a mode).

## Elevation (P2)

- Helper / installer / winws UAC via `ShellExecuteExW` `runas` (`win32.rs`) — no PowerShell.
- `is_elevated` via process token (`TokenElevation`), not `net session`.
- After engine install: keep newest 2 version dirs under the pack prefix (N−1 cleanup).

## Dev console

- Debug builds attach/alloc a console for logs.
- Release: set env `ZAPP_CONSOLE=1` before launch.

## CSP

- Tauri `app.security.csp` restricts script/connect/img to self + Tauri IPC/asset schemes.

## Updates · mirrors · offline pack

- GitHub API bases: `ZAPP_GITHUB_API_BASE`, `ZAPP_GITHUB_API_MIRRORS` (comma-separated), then `api.github.com`.
- Asset CDN rewrite: `ZAPP_GITHUB_ASSET_PREFIX` prepended to release asset URLs.
- **Import zip** on Updates: native file picker → same install path as online Engine (ProgramData preferred).

## Bug report

- IPC: `get_bug_report_bundle` (+ ACL in `capabilities/default.json`).
- Issue body includes environment, recent errors, and recent history log.

## Stack

| Layer | Tech |
|-------|------|
| UI | React 19, TypeScript, CSS Modules |
| Shell | Tauri 2 |
| Backend | Rust (`src-tauri`) |
| Browser demo | Mock API (`npm run dev`) |
