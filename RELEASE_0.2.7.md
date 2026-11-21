## zapp 0.2.7

The leap from **0.2.6 → 0.2.7**: from a solid trust-hardened client to a product that *explains* what works, *remembers* what worked, *watches* quietly after you leave, and *gets out of the way* faster on Start.

0.2.6 made zapp safer (ProgramData engine, helper allowlist, clearer errors). **0.2.7 makes it smarter and faster** — path verdicts that don’t lie about Discord voice, recent working modes one click away, a quiet watchdog, a first-run wizard, conflict shield, localized faults, a real landing page, and a colder Start→UAC path that stops wasting seconds before the admin prompt.

### Upgrading

1. Install **`zapp_0.2.7_x64-setup.exe`** (NSIS), or let Application auto-update download it and confirm UAC.
2. Quit from the tray if the installer cannot replace `zapp.exe`.
3. First launch may show the short setup wizard; approve UAC once on Start as usual.

### From 0.2.6 — what changed for real

| Area | 0.2.6 | 0.2.7 |
|------|-------|-------|
| Home status | Site cards / Soft probes | **Path verdict strip** — YouTube HTTPS, Discord web, Discord voice UDP called out separately |
| Modes you trust | Pick from the full list | **Recent working modes** — last good strategies, re-apply + network restore |
| After Start | Hope it stays up | **Quiet watchdog** — soft recheck, optional auto re-check |
| First run | Figure it out | **First-run wizard** — engine → UAC → Smart check |
| Conflicts | Errors after the fact | **Conflict shield** before you fight GoodbyeDPI / other GUIs |
| Errors language | Mixed | Fault catalog follows **UI locale** (EN/RU) |
| Start → UAC | Preflight then elevate; long pipe timeouts; always re-copy helper | **Elevate early**, skip dead-pipe waits, skip helper copy when already staged |
| Idle CPU / IPC | Aggressive snapshot poll | Throttled OS refresh + conflict cache; lighter UI poll; live diagnostics progress |
| Smart check DNS | Sequential resolve | Parallel resolve + host cache (invalidate on network change) |
| Signing story | SignPath hoped for | Policy updated: **unsigned Authenticode for now**; updater minisign still on |
| Presence | Repo only | **GitHub Pages landing** — mobile, dark/light, EN/RU |

### Added

- **Path verdict strip** on Home — separate honesty for YouTube, Discord web, and Discord voice.
- **Recent working modes** — persist last successful strategies; quick re-apply; play with network restore.
- **Quiet watchdog** — interval recheck while bypass is on; toast + optional auto re-check (Settings).
- **First-run wizard** — guided first minute without dumping you into a blank shell.
- **Conflict shield** — surface competing bypass tools before Start burns a UAC.
- **UI-locale faults** — engine errors match Settings language.
- **Landing** at [krwg.github.io/zapp](https://krwg.github.io/zapp/) — minimal hero, theme + language toggles.
- **DNS cache / parallel resolve** for Smart check and watchdog targets.

### Fixed / faster

- **UAC hang / `UacLaunchFailed`** — Medium-IL pipe ACL so the UI can talk to an elevated helper; stuck elevated helper cleared before re-stage; staging retries without a silent Access Denied loop.
- **Cold Start → admin prompt** — `ensure_helper` before heavy preflight; quick pipe probe instead of multi-second connect timeouts when nothing is listening; skip helper binary copy when size/mtime already match; don’t `taskkill` sleep when helper isn’t running.
- **Snapshot / idle** — OS status refresh and conflict ToolHelp scans throttled; UI poll eased; `diagnostics:progress` updates the UI without a full refresh storm.

### Changed

- Code signing policy documents SignPath Foundation decline; installers ship without Authenticode; in-app updater still minisign-verified.
- Continues on the 0.2.5/0.2.6 Bypass / helper / Quit / ProgramData trust base.

### Known limitations

- Path verdicts and Smart check are still **TCP/HTTPS + soft UDP smoke**, not a full Discord/YouTube client. Voice can still fail in edge cases — that’s why voice is called out separately now.
- First engine download needs GitHub; offline zip import exists on Updates, but no full offline pack browser yet.
- Installers are **not Authenticode-signed**; SmartScreen may warn — verify SHA256 from the release.
- **MSI** has no cleanup page — prefer NSIS.
- Not a VPN; no telemetry.

### Downloads

| File | Notes |
|------|-------|
| `zapp_0.2.7_x64-setup.exe` | Recommended · NSIS · cleanup page |
| `zapp_0.2.7_x64_en-US.msi` | Optional · no cleanup page |

SHA256:

```
zapp_0.2.7_x64-setup.exe
7E7A0DD5303E486EBAF561F7FD028B403FB93B7E9E2E4BD07C05745887C4870F

zapp_0.2.7_x64_en-US.msi
7C010B5091EB70B6F666E51A6F09B6DDCB6E5C1971DAD1E6C948C8FDAC959680
```

### Verify

```powershell
Get-FileHash .\zapp_0.2.7_x64-setup.exe -Algorithm SHA256
Get-FileHash .\zapp_0.2.7_x64_en-US.msi -Algorithm SHA256
```
