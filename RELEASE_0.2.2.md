## zapp 0.2.2

Quiet Windows client for [zapret](https://github.com/bol-van/zapret): hidden `winws`, no Flowseal consoles, separate Application / Engine / Strategies updates.

### Upgrading from 0.2.0 / 0.2.1

1. Prefer **`zapp_0.2.2_x64-setup.exe`** (NSIS). It can offer a full cleanup of previous app data.
2. If auto-update stuck on **Installing…** with **os error 740**, that build could not request elevation. Run the file from `%APPDATA%\app.zapp.desktop\downloads\` once, or install from Assets below and confirm UAC.
3. Close the tray icon if the installer cannot replace `zapp.exe`.

### Fixed

- **Application auto-update failed with os error 740.** The client now launches the downloaded installer with `Start-Process -Verb RunAs`, so Windows shows a normal UAC prompt. Declining UAC returns the channel to Available instead of freezing on Installing….

### Added

- **NSIS full-cleanup page** when previous `%APPDATA%\app.zapp.desktop` / LocalAppData data exists (default-checked). Kills `zapp.exe` / `winws.exe`, removes both folders; shows a non-fatal message if a folder is locked.
- **Uninstall prompt** to wipe leftover app data.
- Sidebar shows the **full semver** (e.g. `[0.2.2]`).

### Changed

- Installer hooks always terminate a running client before replace, so tray-hidden processes do not block updates.

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
