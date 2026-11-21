## zapp 0.2.7

Home now says which of YouTube, Discord web and Discord voice actually works, zapp remembers the modes that worked, and a watchdog keeps an eye on the connection after you close the window. Start also reaches the UAC prompt sooner.

### Upgrading

1. Install `zapp_0.2.7_x64-setup.exe` (NSIS), or let Application auto-update download it and confirm UAC.
2. Quit from the tray if the installer can’t replace `zapp.exe`.
3. The first launch may show a short setup wizard. Approve UAC once on Start, as before.
4. Your current engine pack keeps working (AppData or ProgramData). The next Engine install still prefers `%ProgramData%\zapp\engine\`.

### Added

- **Path verdicts on Home.** YouTube HTTPS, Discord web and Discord voice (UDP) are reported separately. Voice is checked with a light UDP probe, not a real client, so it can still be wrong. The point is that a green web check no longer hides a broken voice.
- **Recent working modes.** The last strategies that passed are kept. Re-apply one in a click; network restore can re-apply a working mode when the network changes.
- **Watchdog.** On by default (every few minutes while bypass is on; turn off in Settings). Soft site recheck and a toast if something breaks. Auto re-check (re-run Smart check) is optional and off by default.
- **First-run wizard.** Engine, UAC, Smart check, in that order.
- **Conflict shield.** Start names competing tools (GoodbyeDPI, other zapret GUIs) before it asks for UAC.
- **Fault messages in your language.** Engine errors follow the interface language, EN or RU.
- **Landing page** at [krwg.github.io/zapp](https://krwg.github.io/zapp/).

### Faster

- **Start reaches UAC sooner.** The helper starts before the heavy preflight, a dead pipe is detected quickly instead of waiting out a long connect timeout, and the helper binary isn’t copied again when size and mtime already match.
- **Smart check resolves DNS in parallel** and caches the result until the network changes.
- **Less idle load.** OS status refresh and conflict scans are throttled, UI polling is lighter, and diagnostics progress no longer triggers a full refresh.

### Fixed

- UAC hang and `UacLaunchFailed`: the pipe now has a Medium-IL ACL so the unelevated UI can talk to the elevated helper. A stuck elevated helper is cleared before staging, and staging retries instead of failing silently on Access Denied.

### Changed

- Installers are not Authenticode-signed. SignPath Foundation declined the application; the [code signing policy](docs/CODE_SIGNING_POLICY.md) is updated. In-app updates are still verified with minisign.

### Known limitations

- Path verdicts and Smart check are TCP/HTTPS plus a light UDP check, not a full Discord or YouTube client. Voice can still fail in edge cases.
- The first engine download needs GitHub. Updates can import a zip, but there is no offline pack browser yet.
- SmartScreen may warn about the unsigned installers. Verify the SHA256 below.
- The MSI has no cleanup page; use the NSIS installer for a clean reinstall.

### Downloads

| File | Notes |
|------|-------|
| `zapp_0.2.7_x64-setup.exe` | Recommended. NSIS installer with the cleanup page. |
| `zapp_0.2.7_x64_en-US.msi` | Optional. No cleanup page. |

SHA256:

```
7E7A0DD5303E486EBAF561F7FD028B403FB93B7E9E2E4BD07C05745887C4870F  zapp_0.2.7_x64-setup.exe
7C010B5091EB70B6F666E51A6F09B6DDCB6E5C1971DAD1E6C948C8FDAC959680  zapp_0.2.7_x64_en-US.msi
```

Check in PowerShell:

```powershell
Get-FileHash .\zapp_0.2.7_x64-setup.exe -Algorithm SHA256
Get-FileHash .\zapp_0.2.7_x64_en-US.msi -Algorithm SHA256
```

**Full changelog:** https://github.com/krwg/zapp/compare/0.2.6...0.2.7
