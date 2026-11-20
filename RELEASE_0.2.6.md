## zapp 0.2.6

Stable 0.2.5 engine lifecycle, plus trust hardening, clearer errors, richer Smart check, and ProgramData engine installs. No Telegram proxy.

### Upgrading

1. Install **`zapp_0.2.6_x64-setup.exe`** (NSIS), or let Application auto-update download it and confirm UAC.
2. Quit from the tray if the installer cannot replace `zapp.exe`.
3. Open **Updates** for Markdown notes; use **Report a bug** from a toast or the error on Home if something breaks.

### Fixed

- **Helper `work_dir`** — Flowseal `bin` cwd is accepted (pack root or `bin`); Start no longer fails with a path allowlist refuse.
- **User-facing errors** — toasts and Home show plain language; technical `refused:` / paths stay in logs and bug-report history.
- **Report a bug** — opens the report dialog directly (not just the About page) with most fields prefilled; steps left for you.
- **Release notes Markdown** — headings, lists, fenced code, callouts in Updates.
- **Update check failures** — network / GitHub errors show as Error on the channel.
- **Preferred installer** — Application updates prefer NSIS over MSI.

### Added

- **ProgramData engine + ACL** — Engine installs prefer `%ProgramData%\zapp\engine\` via elevated helper (`InstallEngine`); SYSTEM/Admins full, Users read/execute. Falls back to AppData if UAC is declined.
- **Helper path allowlist** — elevated helper only starts managed `winws` under AppData or ProgramData packs.
- **Safer Application installs** — SHA256 check against published digest / notes before launch; engine zips get a sanity check.
- **Smarter Smart check** — more YouTube/Discord endpoints (www, image CDN, soft STUN/UDP), soft score in ranking, Fast stops on a strong pass, expandable critical/soft host details and short analysis.
- **Report a bug** on About — clearer copy; GitHub Issue template.

### Changed

- Settings no longer show unused advanced toggles that did nothing.
- Continues on the hardened 0.2.5 Bypass / helper / Quit / Cancel lifecycle.

### Known limitations

- Smart check is still **TCP/HTTPS + soft UDP smoke**, not a full Discord/YouTube client. Voice can still fail in edge cases.
- First engine download needs GitHub; no offline pack picker in the UI yet.
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
6281DF8FD003012886E12D84D72C9025BBB6C21DE8A2F987B91A48F87E014927

zapp_0.2.6_x64_en-US.msi
29C13FF242C12C594DB1C7F4B2DBD6D0D87E261BE424BBD3A230F71969F1F413
```

### Verify

```powershell
Get-FileHash .\zapp_0.2.6_x64-setup.exe -Algorithm SHA256
Get-FileHash .\zapp_0.2.6_x64_en-US.msi -Algorithm SHA256
```
