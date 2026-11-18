## zapp 0.2.5

Hardened engine lifecycle, safer Quit/Cancel, steadier Bypass UI, faster probes, and non-blocking updates.

### Upgrading from 0.2.4

1. Install **`zapp_0.2.5_x64-setup.exe`** (NSIS), or let Application auto-update download it and confirm UAC.
2. Quit from the tray if the installer cannot replace `zapp.exe` — Quit now also stops `winws` and the helper.
3. Run **Smart check** if you want a fresh strategy pick (Fast is the default).

### Fixed

- **Stop-before-restart.** Switching Modes / applying a Smart check winner no longer orphans a live `winws` by dropping `Child` without kill.
- **Owned-only Running status.** A foreign Flowseal `winws.exe` no longer paints Bypass as Running.
- **Quit / Exit cleanup.** Tray Quit, Exit, and update-quit paths stop owned `winws` and shut down the elevated helper.
- **Cancel clears queued Smart check.** Cancel no longer auto-restarts a pending busy start.
- **Helper accept loop timeouts.** A hung localhost client without a newline can no longer freeze elevated IPC forever.
- **Safer stale-helper kill.** Port `:39221` cleanup only stops `zapp-helper` / `zapp`, not an unrelated process.
- **UI snapshot races.** Singleton API, poll merge that preserves `starting`/`stopping`/`checking`, Settings debounce, live Sites details dialog.
- **Cancel stays clickable** while a start invoke is in flight (separate busy keys).

### Added

- **DNS resolve timeout** (~1.5s) so Smart check fails soft instead of hanging on broken DNS.
- **Shared HTTPS client** + tighter abort checks during parallel probes.
- **Async channel updates** — download/install runs off the invoke path; UI polls progress.
- **Helper token ACL** (current user + SYSTEM) after writing `helper.token`.
- **Diagnostics panic catch** with Failed restore path; poison-resistant cancel locks.
- Distinct **Stale** vs **Not checked** site labels.

### Changed

- Application/installer quit stops the engine before exit.
- Release panic strategy is `unwind` so Smart check panics can be recovered.

### Known limitations

- Probes are still **TCP/HTTPS**, not a real Discord client. **Voice (UDP)** can fail while Sites cards look green.
- Helper token ACL reduces casual world-readable tokens; it is not a full same-user security boundary.
- Engine pack is downloaded from GitHub on first run; not bundled in the installer.
- **MSI** has no cleanup page — prefer NSIS.
- Not a VPN; no telemetry.

### Downloads

| File | Notes |
|------|-------|
| `zapp_0.2.5_x64-setup.exe` | Recommended · NSIS · cleanup page |
| `zapp_0.2.5_x64_en-US.msi` | Optional · no cleanup page |

SHA256:

```
zapp_0.2.5_x64-setup.exe
9BE59F8CE89779B68528AC8C353872C094B9373E44514EC5222BD372918CA722

zapp_0.2.5_x64_en-US.msi
0092621E116F154C98EA8B6022BBA896B92005D327BE36B4D67720961E14CB9D
```

Engine / strategies: [bol-van/zapret](https://github.com/bol-van/zapret) · [Flowseal/zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube)

Issues: https://github.com/krwg/zapp/issues
