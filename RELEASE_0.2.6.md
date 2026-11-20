## zapp 0.2.6

Stable 0.2.5 engine lifecycle plus GUI polish and slice-A trust hardening: helper `winws` path allowlist, SHA256 for Application installs, visible update-check errors, and removal of dead Settings toggles. No Telegram MTProto proxy. No ProgramData migration yet (roadmap 1b).

### Upgrading from 0.2.5

1. Install **`zapp_0.2.6_x64-setup.exe`** (NSIS), or let Application auto-update download it and confirm UAC.
2. Quit from the tray if the installer cannot replace `zapp.exe`.
3. Open **Updates** for Markdown release notes; **About → Report a bug** if something breaks.

### Fixed

- **Release notes Markdown** — ATX headings, lists, fenced code, blockquotes/callouts (no more raw `###` in Updates).
- **Silent update checks** — GitHub fetch failures set channel status to Error with a real message (rate-limit / offline), instead of looking “up to date”.
- **Application asset preference** — NSIS `setup.exe` preferred over MSI (matches README).

### Added

- **Helper path allowlist** — elevated helper only starts `…\app.zapp.desktop\engine\<ver>\bin\winws.exe` with matching work_dir (blocks arbitrary elevated exe via Start IPC).
- **SHA256 for Application updates** — verify GitHub asset `digest` or SHA256 from release notes before launching the installer; refuse if missing/mismatch.
- **Engine zip sanity** — reject non-zip / tiny downloads before extract.
- **Richer Smart check scoring** — per-service scores, host details, soft score, median/max latency; expandable rows.
- **Report a bug** on About — prefilled GitHub Issue; error toasts link to About; issue template.

### Changed

- Settings UI no longer shows non-functional toggles (`autoDiagnostics`, interval, retries, verbose logging, engine/strategies paths). Values remain in stored settings for compatibility.
- Ships on the hardened 0.2.5 Bypass / helper / Quit / Cancel lifecycle.

### Known limitations

- Probes are still **TCP/HTTPS**, not a real Discord client. **Voice (UDP)** can fail while Sites cards look green.
- Helper path allowlist is not a full UAC boundary: engine pack remains under user-writable `%APPDATA%` until ProgramData migration (roadmap 1b). Token is still TCP+file based (1c later).
- Flowseal engine packs are not SHA256-pinned yet (zip magic only); signed manifest later.
- Engine pack downloads from GitHub on first run; not bundled in the installer.
- **MSI** has no cleanup page — prefer NSIS.
- Not a VPN; no telemetry.

### Downloads

| File | Notes |
|------|-------|
| `zapp_0.2.6_x64-setup.exe` | Recommended · NSIS · cleanup page |
| `zapp_0.2.6_x64_en-US.msi` | Optional · no cleanup page |

SHA256:

```
zapp_0.2.6_x64-setup.exe
82D1658C12B918DDACBC9E6A15030B68B8E16C8A67A4F0BFE99C7BFF54B273F8

zapp_0.2.6_x64_en-US.msi
28F7F32F04611D4648ECB65CDD66A9D88B872147F24030BA93320D8A2C705C5E
```

### Verify

```powershell
Get-FileHash .\zapp_0.2.6_x64-setup.exe -Algorithm SHA256
Get-FileHash .\zapp_0.2.6_x64_en-US.msi -Algorithm SHA256
```
