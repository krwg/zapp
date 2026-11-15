## zapp 0.2.2

Quiet Windows client for [zapret](https://github.com/bol-van/zapret): hidden `winws`, no Flowseal consoles, separate Application / Engine / Strategies updates.

### Install

1. Run **`zapp_0.2.2_x64-setup.exe`** (NSIS) and open zapp.
2. Press **Start** and approve UAC once.
3. Wait for the engine pack to download (or enable auto-install on Updates), then run **Smart check**.
4. If the installer cannot replace files, quit zapp from the tray first.

### Added

- **Hidden engine.** Starts `winws` through one elevated helper — one UAC, no BAT consoles.
- **NSIS full-cleanup page** when previous `%APPDATA%\app.zapp.desktop` / LocalAppData data exists (default-checked). Kills `zapp.exe` / `winws.exe`, removes both folders; shows a non-fatal message if a folder is locked.
- **Uninstall prompt** to wipe leftover app data.
- **Application auto-update** launches the downloaded installer with `Start-Process -Verb RunAs` so Windows shows a normal UAC prompt. Declining UAC returns the channel to Available.
- Sidebar shows the **full semver** (e.g. `[0.2.2]`).
- Separate **Engine / Strategies** updates from Flowseal GitHub Releases.

### Known limitations

- **Not a VPN.** Does not hide your IP or encrypt traffic.
- Engine pack is **not** bundled; first run downloads Flowseal from GitHub into `%APPDATA%\app.zapp.desktop\engine\`.
- **MSI** has no cleanup page — use NSIS for full reinstall.
- WinDivert-based tools may trip antivirus / SmartScreen; verify SHA256 or build from source.
- No accounts, no telemetry.

### Downloads

| File | Notes |
|------|-------|
| `zapp_0.2.2_x64-setup.exe` | Recommended · NSIS · cleanup page |
| `zapp_0.2.2_x64_en-US.msi` | Optional · no cleanup page |

SHA256:

```
zapp_0.2.2_x64-setup.exe
FCDC03F40072C2816E14F8F9A5A7FF2182E7CA2E2F03F9B8E1055D15EE11B24A

zapp_0.2.2_x64_en-US.msi
61CCD64193DDD0C26B986724DD3B257DF86787EA419B6C1D6EC1121B97568A7E
```

Engine / strategies: [bol-van/zapret](https://github.com/bol-van/zapret) · [Flowseal/zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube)

Issues: https://github.com/krwg/zapp/issues
