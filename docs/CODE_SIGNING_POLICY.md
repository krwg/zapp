# Code signing policy (0.2.7+)

Windows installers for **zapp** ship from GitHub Releases. Authenticode via SignPath Foundation was applied for and **declined** (Nov 2026) due to insufficient public visibility for the Foundation program. Paid SignPath is not used. Releases therefore ship **without Authenticode** for now; we may reapply to SignPath Foundation later.

## What users get today

- **Installers:** unsigned NSIS `zapp_*_x64-setup.exe` (and MSI). Windows SmartScreen may warn on first run — expected until Authenticode exists.
- **In-app updates:** still integrity-checked with minisign (`plugins.updater.pubkey` in `tauri.conf.json`). That layer does not replace Authenticode and does not silence SmartScreen.

## Downloads

- Installers: [GitHub Releases](https://github.com/krwg/zapp/releases)
- Recommended file: `zapp_*_x64-setup.exe` (NSIS)

## Details

- **Project:** [krwg/zapp](https://github.com/krwg/zapp)
- **What would be Authenticode-signed (if approved later):** Windows NSIS installer (`*-setup.exe`), optionally MSI
- **Build origin:** GitHub Actions workflow `Release Windows` on this repository only
- **CI:** SignPath steps stay optional; they run only when SignPath secrets are present. Until then, builds skip Authenticode and publish unsigned setup + updater `.sig`
- **How to verify updater trust:** matching `.sig` next to the installer / `latest.json` from the same release

Maintainer contact: GitHub Issues on the repository above.
