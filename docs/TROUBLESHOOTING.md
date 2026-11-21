# Troubleshooting

## SmartScreen / antivirus

Installers are **not Authenticode-signed** ([policy](CODE_SIGNING_POLICY.md)). SmartScreen may say “Windows protected your PC”.

1. Prefer the SHA256 on the [release page](https://github.com/krwg/zapp/releases/latest) (or `SHA256SUMS` asset when present).
2. *More info → Run anyway* if the hash matches.
3. WinDivert-based tools are often flagged — that alone doesn’t mean the build is trojaned; compare hashes or build from source.

## UAC / `UacLaunchFailed` / helper

- Approve the UAC prompt for `zapp-helper` (not only for the installer).
- If you declined UAC, Start again.
- Quit zapp from the tray, ensure no stuck `zapp-helper.exe` / `winws.exe`, reopen, Start once.
- Corporate policies that block elevation will prevent bypass.

## Smart check finds nothing

- Close GoodbyeDPI, other zapret GUIs, and overlapping VPN/DPI tools (Conflict shield names common ones).
- Stop leftover `winws.exe` from a Flowseal `.bat`.
- Need GitHub reachability for the first engine download (or import a zip on Updates).
- Try **Thorough** if Fast’s early stop didn’t land a good mode.
- Voice can fail while web probes look fine — check path verdicts on Home; try another mode.

## Voice works in browser but not in Discord (or the reverse)

Path verdicts and Smart check use TCP/HTTPS plus a **light UDP** smoke test, not Discord’s real voice stack. Treat voice as best-effort. Re-run Smart check or pick a recent working mode manually.

## Installer can’t replace `zapp.exe`

Quit from the **tray** (not only close the window). Quit stops `winws` and the helper.

## Bypass dies after sleep or network change

Settings → restore on resume / network change. Watchdog (on by default) soft-checks while bypass is desired; enable auto re-check if you want Smart check to run again automatically.

## Clean uninstall

Use the NSIS uninstaller cleanup page when possible. By hand delete:

- `%APPDATA%\app.zapp.desktop`
- `%LOCALAPPDATA%\zapp`
- `%ProgramData%\zapp` (if present)

## Still stuck

About → **Report a bug** (prefills versions and recent errors), or open a GitHub issue with the Bug report template. Don’t paste `helper.token` or full strategy hostlists.
