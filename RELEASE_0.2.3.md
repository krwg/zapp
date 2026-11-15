## zapp 0.2.3

Smarter Discord scoring for Smart check, parallel site probes, and less chance of crowning a YouTube-only mode.

### Upgrading from 0.2.2

1. Install **`zapp_0.2.3_x64-setup.exe`** (NSIS), or let Application auto-update download it and confirm UAC.
2. Quit from the tray if the installer cannot replace `zapp.exe`.
3. Run **Smart check** again — winners now require both YouTube and Discord paths.

### Fixed

- **Smart check could pick a YouTube-strong mode while Discord stayed broken.** Scoring is per service: YouTube and Discord must both pass. Discord probes include `discord.com`, `gateway.discord.gg`, and `discord.media` on Flowseal TCP ports (2053 / 2083 / 8443). CDN is a soft factor only.
- Early-stop no longer locks onto a high aggregate score after a couple of YouTube-friendly tries.

### Added

- **Parallel host probes** inside each Smart check strategy (hosts concurrently; strategies still one `winws` at a time).
- **Parallel “Check all”** on the Sites page.
- Discord built-in site host defaults to **`gateway.discord.gg`**.

### Changed

- Smart check result notes show per-service scores (e.g. `youtube 88 · discord 71`).
- Sidebar / About keep the full semver (`[0.2.3]`).

### Known limitations

- Probes are still **TCP/HTTPS**, not a real Discord client. **Voice (UDP)** can fail while Sites cards look green.
- A high score can still feel worse than a hand-picked Flowseal mode on some ISPs — use **Modes** manually if needed.
- Engine pack is downloaded from GitHub on first run; not bundled in the installer.
- **MSI** has no cleanup page — prefer NSIS.
- Not a VPN; no telemetry.

### Downloads

| File | Notes |
|------|-------|
| `zapp_0.2.3_x64-setup.exe` | Recommended · NSIS · cleanup page |
| `zapp_0.2.3_x64_en-US.msi` | Optional · no cleanup page |

SHA256:

```
zapp_0.2.3_x64-setup.exe
EA2EF4021F04ED322E4619F8D4A6B2B83A53522CB2CDB9F3AF93A71F56DE0CD4

zapp_0.2.3_x64_en-US.msi
31FE13D268D83E769901CC9D09EA7638B6ECD0926943238D9BD225BB74285E9A
```

Engine / strategies: [bol-van/zapret](https://github.com/bol-van/zapret) · [Flowseal/zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube)

Issues: https://github.com/krwg/zapp/issues

