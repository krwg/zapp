# Code signing policy

Required by [SignPath Foundation](https://signpath.org/) for open-source Authenticode signing.

Windows installers for **zapp** are intended to be Authenticode-signed via the SignPath Foundation. Until signing is active, GitHub Releases may still ship unsigned builds.

## Downloads

- Installers: [GitHub Releases](https://github.com/krwg/zapp/releases)
- Recommended file: `zapp_*_x64-setup.exe` (NSIS)
- This project uses the **SignPath Foundation** for code signing of Windows release artifacts

## Details

- **Project:** [krwg/zapp](https://github.com/krwg/zapp)
- **What is signed:** Windows NSIS installer (`*-setup.exe`), optionally MSI
- **Who signs:** SignPath Foundation (when the org/project is approved and CI secrets are set)
- **Build origin:** GitHub Actions workflow `Release Windows` on this repository only
- **How to verify:** installer Properties → Digital Signatures, or `Get-AuthenticodeSignature .\zapp_*_x64-setup.exe`
- **In-app updates:** separate minisign key (`plugins.updater.pubkey` in `tauri.conf.json`); after Authenticode signing, updater `.sig` files are regenerated so both trust layers match

Maintainer contact: GitHub Issues on the repository above.
