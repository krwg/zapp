# Code signing policy

Required by [SignPath Foundation](https://signpath.org/) for open-source Authenticode signing.

- **Project:** [krwg/zapp](https://github.com/krwg/zapp)
- **What is signed:** Windows NSIS installer (`*-setup.exe`), optionally MSI
- **Who signs:** SignPath Foundation (when the org/project is approved and CI secrets are set)
- **Build origin:** GitHub Actions workflow `Release Windows` on this repository only
- **How to verify:** installer Properties → Digital Signatures, or `Get-AuthenticodeSignature .\zapp_*_x64-setup.exe`
- **In-app updates:** separate minisign key (`plugins.updater.pubkey` in `tauri.conf.json`); Authenticode and updater signatures are both required on published installers

Maintainer contact: GitHub Issues on the repository above.
