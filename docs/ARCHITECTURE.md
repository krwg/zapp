# Architecture (0.2.7)

```
UI (React)
  → ZappApi (tauriClient | mockClient)
  → Rust commands
      → AppState
      → ManagedEngine        # starts/stops hidden winws
      → elevated helper      # one UAC, then silent local control
      → UpdateManager        # Application / Engine / Strategies
      → Diagnostics          # Smart check probes + progress events
      → Watchdog             # quiet path recheck while bypass is on
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

- Binary: `%LOCALAPPDATA%\zapp\zapp-helper.exe` (staged copy of the elevated helper; skip copy when size/mtime already match)
- IPC: Windows named pipe `\\.\pipe\zapp-helper` (no TCP listen; token required; Medium-IL ACL so unelevated UI can connect)
- Auth: token file `%LOCALAPPDATA%\zapp\helper.token`
- Starts / stops `winws` after the user approved UAC once
- Path allowlist: managed packs under AppData or ProgramData (`bin` or pack-root cwd)
- `InstallEngine`: copies a staged AppData pack into ProgramData and applies ACL
- Cold Start path: `ensure_helper` (UAC) runs **before** full preflight when elevation is allowed; dead-pipe probes use a short timeout

## Product surfaces (0.2.7)

| Piece | Role |
|-------|------|
| Path verdict strip | Separate YouTube HTTPS / Discord web / Discord voice UDP honesty on Home |
| Recent working modes | Persist last good strategies in `session.json`; one-click re-apply + network restore |
| Conflict shield | Surface competing bypass images before Start |
| First-run wizard | Engine → UAC → Smart check guided flow |
| Quiet watchdog | Interval soft recheck; toast; optional auto re-check (Settings) |
| Locale-aware faults | `set_ui_locale` → EN/RU catalogs in `engine/faults.rs` |

## Update channels

| Channel | Source |
|---------|--------|
| Application | [krwg/zapp](https://github.com/krwg/zapp/releases) · minisign `.sig` |
| Engine | Flowseal GitHub Releases (zip) → stage in AppData, prefer elevated ProgramData install |
| Strategies | Same Flowseal pack (profiles from `general*.bat`) |

Authenticode: not currently applied — see [CODE_SIGNING_POLICY.md](CODE_SIGNING_POLICY.md).

## Data on disk

| Path | Contents |
|------|----------|
| `%ProgramData%\zapp\` | Preferred engine packs (ACL: SYSTEM/Admins full, Users RX) |
| `%APPDATA%\app.zapp.desktop\` | Settings (`settings.json`), session (`session.json`: bypass desired, working modes), downloads/staging, fallback engine packs, logs |
| `%LOCALAPPDATA%\zapp\` | Helper exe + token |

## Bypass restore + network memory

- `session.json` remembers `bypassDesired` + active strategy (set on Start, cleared on Stop / tray Quit) and recent working modes.
- After engine bootstrap at login: if `restoreBypassOnLogin` → `ensure_helper` + start.
- Background wall-clock watcher: gap &gt; 25s ≈ sleep resume → if `restoreBypassOnResume` → start again.
- Network fingerprint = default IPv4 gateway + WLAN SSID; on change (if `restoreBypassOnNetworkChange`) restore bypass and invalidate DNS probe cache.

## Smart check / probes

- Critical HTTPS (YouTube / Discord gateway / discord.media TCP alts).
- Soft: ytimg, youtubei.googleapis.com, googlevideo.com, Discord CDN, STUN UDP, QUIC UDP:443 smoke (reply-based, does not alone fail a mode).
- DNS: parallel resolve + in-process host cache; `invalidate_dns_cache` on network change.
- Progress: `diagnostics:progress` / `diagnostics:completed` events for live UI without full snapshot storms.

## Snapshot / idle

- `get_snapshot` throttles helper/OS status refresh (~8s) and conflict ToolHelp scans (~12s).
- UI idle poll is lighter; transitional Start/Stop still merges carefully.

## Elevation

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

## Landing (GitHub Pages)

- Static site under `docs/` (`index.html`, `site.css`, `site.js`) — hero-only, theme + EN/RU, mobile menu.
- Homepage: https://krwg.github.io/zapp/

## Stack

| Layer | Tech |
|-------|------|
| UI | React 19, TypeScript, CSS Modules |
| Shell | Tauri 2 |
| Backend | Rust (`src-tauri`) |
| Browser demo | Mock API (`npm run dev`) |
