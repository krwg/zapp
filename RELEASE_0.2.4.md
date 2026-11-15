## zapp 0.2.4

Faster Smart check, safer cancel/restore, clearer Bypass UI, and quieter Home site status.

### Upgrading from 0.2.3

1. Install **`zapp_0.2.4_x64-setup.exe`** (NSIS), or let Application auto-update download it and confirm UAC.
2. Quit from the tray if the installer cannot replace `zapp.exe`.
3. Open Home — site cards stay grey until a fresh check; run **Smart check** (Fast is the default).

### Fixed

- **Diagnostics race on cancel/restart.** One worker at a time with `run_id`; cancel restores the previous mode/engine state and does not emit Completed over Cancelled.
- **Cancelled runs no longer apply a profile** after the strategy loop (Analyzing phase respects cancel).
- **Best-strategy start errors** report Failed (with fallback by score) instead of Completed while `winws` is still stopped.
- **Portable zip update** waits for `zapp.exe` to exit, stops `winws`, checks `xcopy`, then relaunches.
- **Orphan `winws`** from a previous helper session is cleaned before env-preflight, so it is not mistaken for a foreign Flowseal process.
- **No-network Smart check** stops early with a clear DNS/network message and restores previous state instead of scoring every strategy as zero.
- **Late `winws` crash during probes** is marked as startup-failed, not as “weak services”.

### Added

- **Faster Smart check path:** ToolHelp instead of `tasklist`/`taskkill` on the hot path; env-preflight once per run; short poll instead of long fixed sleeps; DNS once per run; IPv4-first probes; HTTPS without redundant TCP; parallel Discord media ports; Fast / Thorough modes.
- **Queued Smart check** when a run is still finishing — the latest start is kept and begins after cancel cleanup (no silent drop, no UI freeze on join).
- **Bypass wording** across Home, sidebar, tray, toasts, and hints (EN/RU) — avoids “Protection” sounding like antivirus/VPN.
- **Home Current mode** shows a short name + status line; technical details stay collapsed; first-run CTA to open Smart check.
- **Stale Sites status:** when bypass is off or a probe is older than a few minutes, cards show grey **Not checked** (and a silent Home refresh at most every ~45s, never during Smart check).

### Changed

- Manual Smart check no longer prefers the previous strategy first (that hid UDP/voice failures behind a passing TCP check). Prefer-previous remains available for background re-check.
- Sidebar bypass chip is hidden on Home (the large Bypass card is enough).
- Session time on the Bypass card is hidden while bypass is off.
- Repair renamed to **Reset engine**.

### Known limitations

- Probes are still **TCP/HTTPS**, not a real Discord client. **Voice (UDP)** can fail while Sites cards look green.
- A high score can still feel worse than a hand-picked Flowseal mode on some ISPs — use **Modes** manually if needed.
- Engine pack is downloaded from GitHub on first run; not bundled in the installer.
- **MSI** has no cleanup page — prefer NSIS.
- Not a VPN; no telemetry.

### Downloads

| File | Notes |
|------|-------|
| `zapp_0.2.4_x64-setup.exe` | Recommended · NSIS · cleanup page |
| `zapp_0.2.4_x64_en-US.msi` | Optional · no cleanup page |

SHA256:

```
zapp_0.2.4_x64-setup.exe
6CFA9423318DAE7B0EB59BDF47FF909BC93B8E4189F8EE11EAFB3A6E00B8E6F5

zapp_0.2.4_x64_en-US.msi
64B7D6816854483E863D05461CDE09E01EDC4DEE044F4549151013CAD1026602
```

Engine / strategies: [bol-van/zapret](https://github.com/bol-van/zapret) · [Flowseal/zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube)

Issues: https://github.com/krwg/zapp/issues
