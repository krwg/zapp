## zapp 0.2.6

Native Telegram MTProto proxy (separate from Bypass), richer Smart check results UI, Markdown release notes, and in-app bug reports via GitHub Issues.

### Upgrading from 0.2.5

1. Install **`zapp_0.2.6_x64-setup.exe`** (NSIS), or let Application auto-update download it and confirm UAC.
2. Quit from the tray if the installer cannot replace `zapp.exe`.
3. Open **Updates** → install **Telegram proxy** if you want the local MTProto sidecar, then use the **Telegram** card on Home (separate from Bypass Start).
4. On first successful proxy start, use **Open in Telegram** (`tg://proxy`) if offered.

### Fixed

- **Release notes Markdown** — ATX headings, ordered/unordered lists, fenced code, blockquotes/callouts (no more raw `###` in Updates).
- **Cancel-flag poison** — diagnostics cancel locks recover with `into_inner` instead of panicking.

### Added

- **Telegram proxy** — managed [Flowseal/tg-ws-proxy](https://github.com/Flowseal/tg-ws-proxy) sidecar (`TgWsProxy_windows.exe`), pack under app data, secret ACL, Updates channel, Sites entry, Quit cleanup. Toggle is **independent** of Bypass.
- **First-start `tg://` CTA** — offer Open / Copy link after the first successful proxy start (can disable in Settings).
- **Richer Smart check scoring** — per-service scores, host details, soft score, median/max latency; collapsed `Score · Response` row with expand.
- **Report a bug** on About — in-app form opens a prefilled GitHub Issue (`bug` label). Error toasts link to About. Issue template added.

### Changed

- Updates model includes a fourth channel: Telegram proxy.
- Settings: auto-start Telegram proxy (off by default), offer link on first start.

### Known limitations

- Probes are still **TCP/HTTPS**, not a real Discord client. **Voice (UDP)** can fail while Sites cards look green.
- Telegram proxy is **not** part of Smart check pass criteria; DPI Bypass ≠ MTProto proxy.
- Flowseal’s binary may still show its own tray icon; zapp does not embed a headless-only build.
- Engine / Telegram packs download from GitHub on first use; not bundled in the installer.
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
0BC0311C1B76711DA030558F75DB70B3E67CAB87216AFFF5856006F22B91229A

zapp_0.2.6_x64_en-US.msi
C875949FF6AA07D5A31D610E2B9A614E57F03BA092C8340AC01CA87108330796
```

### Verify

```powershell
Get-FileHash .\zapp_0.2.6_x64-setup.exe -Algorithm SHA256
Get-FileHash .\zapp_0.2.6_x64_en-US.msi -Algorithm SHA256
```
