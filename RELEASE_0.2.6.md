## zapp 0.2.6

Keeps the stable 0.2.5 engine lifecycle and adds clearer Updates / About UX, richer Smart check results, and stricter checks before applying Application updates. No Telegram proxy in this build.

### Upgrading from 0.2.5

1. Install **`zapp_0.2.6_x64-setup.exe`** (NSIS), or let Application auto-update download it and confirm UAC.
2. Quit from the tray if the installer cannot replace `zapp.exe`.
3. Open **Updates** for Markdown release notes; **About → Report a bug** if something breaks.

### Fixed

- **Release notes Markdown** — headings, lists, fenced code, and callouts render correctly in Updates (no more raw `###`).
- **Update check failures** — network / GitHub errors show as Error on the channel instead of looking “up to date”.
- **Preferred installer** — Application updates prefer the NSIS setup over MSI.

### Added

- **Richer Smart check results** — per-service scores, host details, soft score, median/max latency; expandable rows.
- **Report a bug** on About — prefilled GitHub Issue; error toasts can open About; issue template added.
- **Safer Application installs** — downloaded installers are checked against published SHA256 (GitHub digest or release notes) before launch.
- **Engine download checks** — reject obviously invalid zip downloads before extract.
- **Helper launch checks** — elevated helper only starts the managed engine binary from the installed pack layout.

### Changed

- Settings no longer show unused advanced toggles that did nothing in the UI.
- Continues on the hardened 0.2.5 Bypass / helper / Quit / Cancel lifecycle.

### Known limitations

- Probes are still **TCP/HTTPS**, not a real Discord client. **Voice (UDP)** can fail while Sites cards look green.
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
