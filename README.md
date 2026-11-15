<div align="center">

<img width="112" height="112" alt="zapp" src="docs/screenshots/icon-128.png" />

# zapp

**A quiet desktop client for [zapret](https://github.com/bol-van/zapret) on Windows.**  
Start and stop DPI-bypass strategies for YouTube and Discord without console windows.

[![Release](https://img.shields.io/github/v/release/krwg/zapp?style=flat-square&labelColor=0a0a0a&color=e8e8e8)](https://github.com/krwg/zapp/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/krwg/zapp/total?style=flat-square&labelColor=0a0a0a&color=e8e8e8)](https://github.com/krwg/zapp/releases)
[![License](https://img.shields.io/badge/license-MIT-555?style=flat-square&labelColor=0a0a0a)](LICENSE)
![Windows 10/11](https://img.shields.io/badge/Windows-10%2F11%20x64-0078D6?style=flat-square&labelColor=0a0a0a)

[English](README.md) · [Русский](README.ru.md) · [Releases](https://github.com/krwg/zapp/releases)

<img src="docs/screenshots/01-overview.png" alt="zapp Home" width="860" />

</div>

## What it is

zapp wraps the zapret engine and the [Flowseal strategy pack](https://github.com/Flowseal/zapret-discord-youtube) in a small native UI (Tauri 2 · React 19 · Rust).

- Runs `winws` hidden. No BAT consoles, one UAC prompt.
- **Smart check** scores strategies against YouTube and Discord (including gateway and `discord.media` ports) and prefers a mode that passes both.
- **Sites** shows reachability for YouTube, Discord and your own hosts.
- Separate update channels for the app, the engine and the strategies.
- EN / RU interface, Light / Dark / System theme, tray mode.

**What it is not:** a VPN, antivirus, or “protection” product. It does not hide your IP or encrypt traffic. It only changes how packets look to DPI on the path to blocked services. Check that using it is legal where you live.

## Install

1. Download `zapp_*_x64-setup.exe` from [Releases](https://github.com/krwg/zapp/releases/latest).
2. Run it and open zapp.
3. Press **Start** and approve the UAC prompt once.
4. Wait for the engine pack to download, then run **Smart check**.

Requirements: Windows 10/11 x64. Administrator rights are needed once to start the helper.

| File | Use |
|------|-----|
| `*_x64-setup.exe` (NSIS) | Recommended. Has a cleanup page for reinstalls. |
| `*_x64_en-US.msi` | Alternative. No cleanup page. |

SHA256 sums are listed in each release.

## Trust and privacy

- **Why UAC:** `winws` uses [WinDivert](https://reqrypt.org/windivert.html), which loads a kernel driver and needs admin rights. zapp starts one elevated helper (`zapp-helper.exe`) and talks to it on `127.0.0.1` over a token-authenticated local TCP port — not over the internet.
- **What is downloaded:** on first run (and when Engine/Strategies update), the Flowseal pack from its GitHub Releases into `%APPDATA%\app.zapp.desktop\engine\`. Application updates come from [krwg/zapp](https://github.com/krwg/zapp/releases). The only other outbound calls are GitHub release checks and the optional site probes you trigger (YouTube, Discord, …).
- **No accounts, no telemetry, no analytics.** Settings, logs and the engine pack stay under `%APPDATA%\app.zapp.desktop\` (helper state under `%LOCALAPPDATA%\zapp\`).
- **Antivirus / SmartScreen:** WinDivert-based tools are often flagged as riskware or “hacking tools”. That is expected for a packet filter driver, not proof of malware. The zapp source is open; verify SHA256 of the installer, or build it yourself.

## Troubleshooting

- **SmartScreen blocks the installer:** *More info → Run anyway*, or verify the SHA256 first.
- **No strategy passes Smart check:** close other zapret / VPN / DPI tools and try again.
- **Update stuck on "Installing…" (0.2.0 / 0.2.1):** run the installer from `%APPDATA%\app.zapp.desktop\downloads\` once (those builds could not request UAC).

To remove zapp completely, uninstall it and accept the cleanup prompt (or delete `%APPDATA%\app.zapp.desktop` and `%LOCALAPPDATA%\zapp` manually).

## Build from source

```bash
npm install
npm run tauri:dev     # desktop app
npm run tauri:build   # NSIS + MSI
```

Needs the Rust toolchain and MSVC Build Tools. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Credits

zapp is only the client UI + core. The engine belongs to its authors.

| Project | Author |
|---------|--------|
| [zapret](https://github.com/bol-van/zapret) | [bol-van](https://github.com/bol-van) |
| [zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube) | [Flowseal](https://github.com/Flowseal) |

zapp does not ship WinDivert or `winws`. Their licenses stay with the upstream projects.

## License

MIT · [krwg](https://github.com/krwg)
