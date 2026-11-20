<div align="center">

<img width="112" height="112" alt="zapp" src="docs/screenshots/icon-128.png" />

# zapp

**Desktop client for [zapret](https://github.com/bol-van/zapret) on Windows.**  
Run a DPI-bypass strategy for YouTube and Discord without a console window.

[![Release](https://img.shields.io/github/v/release/krwg/zapp?style=flat-square&labelColor=0a0a0a&color=e8e8e8)](https://github.com/krwg/zapp/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/krwg/zapp/total?style=flat-square&labelColor=0a0a0a&color=e8e8e8)](https://github.com/krwg/zapp/releases)
[![License](https://img.shields.io/badge/license-MIT-555?style=flat-square&labelColor=0a0a0a)](LICENSE)
![Windows 10/11](https://img.shields.io/badge/Windows-10%2F11%20x64-0078D6?style=flat-square&labelColor=0a0a0a)

[English](README.md) · [Русский](README.ru.md) · [Releases](https://github.com/krwg/zapp/releases)

<img src="docs/screenshots/01-overview.png" alt="zapp Home" width="860" />

</div>

I made zapp because I was tired of launching BAT files with a console window you shouldn’t close, guessing which Flowseal strategy fits your ISP, and babysitting `winws` by hand.

zapp wraps zapret and the [Flowseal strategy pack](https://github.com/Flowseal/zapret-discord-youtube) in a small Tauri app. Press Start on the **Bypass** card, approve UAC once, and `winws` runs hidden in the background.

**Smart check** (Fast by default, Thorough optional) tries strategies against YouTube and Discord — gateway plus `discord.media` ports — and turns on one that passes both. From **0.2.4** the loop is much faster; **0.2.5** hardens the engine lifecycle (stop-before-restart, Quit stops `winws`/helper, Cancel clears queued restarts), keeps Bypass status owned-only, and speeds DNS/probes further. Home keeps site cards grey until a fresh probe; stale “Available” from days ago no longer looks like live access.

The app, engine and strategies update separately. English and Russian UI, light/dark themes, tray support.

zapp is not a VPN. Your IP stays visible and nothing is encrypted; the only thing that changes is how your packets look to DPI. Check that this is legal where you live.

## Install

1. Download `zapp_*_x64-setup.exe` from [Releases](https://github.com/krwg/zapp/releases/latest) and install it.
2. Open zapp, press **Start** on Bypass, approve the UAC prompt.
3. Wait while the engine pack downloads, then run **Smart check**.

Windows 10/11 x64. Admin rights are needed once, to start the helper.

The NSIS `setup.exe` is the one I recommend: on a reinstall it can wipe the old app data. The `.msi` works too but has no cleanup page. SHA256 sums are in each release.

## What it does on your machine

`winws` uses [WinDivert](https://reqrypt.org/windivert.html), which loads a kernel driver, so it needs admin rights. zapp starts one elevated helper, `zapp-helper.exe`, and talks to it over a token-authenticated TCP port on `127.0.0.1`. Nothing there is reachable from the network.

On first run, and when Engine or Strategies update, zapp downloads the Flowseal pack from its GitHub Releases into `%APPDATA%\app.zapp.desktop\engine\`. App updates come from [krwg/zapp](https://github.com/krwg/zapp/releases). Apart from GitHub release checks and the site probes you start yourself, zapp makes no other requests. There is no telemetry. Settings and logs stay in `%APPDATA%\app.zapp.desktop\`, helper state in `%LOCALAPPDATA%\zapp\`.

Antivirus and SmartScreen sometimes flag WinDivert-based tools, because a packet-filter driver looks suspicious by nature. The source is open, so you can check the installer's SHA256 or build it yourself.

## Known rough edges

Short version before you install:

- As of **0.2.3+**, Smart check requires both YouTube and Discord (gateway + `discord.media` on Flowseal ports). It still only does **TCP/HTTPS probes**, not a real client — **Discord voice (UDP)** can fail while the Sites cards stay green.
- From **0.2.4+** the check is much faster and steadier; if something still feels off afterwards, open **Modes** and pick a strategy by hand.
- First launch needs GitHub access to download the engine pack. There is no offline pack picker in the UI yet.
- Closing the window hides to the tray by default; the process is still running. **Quit from the tray** before reinstalling (0.2.5 also stops `winws`/helper on Quit), or the setup cannot replace `zapp.exe`.
- Windows 10/11 x64 only. No macOS or Linux build.

## If something doesn't work

If SmartScreen blocks the installer, choose *More info → Run anyway*, or verify the SHA256 first. If no strategy passes Smart check, close other zapret, VPN or DPI tools and run it again. If a Flowseal `.bat` left `winws.exe` running, stop it first — Smart check will tell you.

To remove zapp completely, uninstall it and accept the cleanup prompt. By hand, delete `%APPDATA%\app.zapp.desktop` and `%LOCALAPPDATA%\zapp`.

## Build from source

```bash
npm install
npm run tauri:dev     # desktop app
npm run tauri:build   # NSIS + MSI
```

You need the Rust toolchain and MSVC Build Tools. Architecture notes are in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Credits

The engine isn't mine. zapp is the UI and the glue around it.

| Project | Author |
|---------|--------|
| [zapret](https://github.com/bol-van/zapret) | [bol-van](https://github.com/bol-van) |
| [zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube) | [Flowseal](https://github.com/Flowseal) |

zapp doesn't ship WinDivert or `winws`; their licenses stay with the upstream projects.

MIT · [krwg](https://github.com/krwg)
