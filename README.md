<div align="center">

<img width="112" height="112" alt="zapp" src="docs/screenshots/icon-128.png" />

# zapp

**Desktop client for [zapret](https://github.com/bol-van/zapret) on Windows.**  
Run a DPI-bypass strategy for YouTube and Discord without a console window.

[![Release](https://img.shields.io/github/v/release/krwg/zapp?style=flat-square&labelColor=0a0a0a&color=e8e8e8)](https://github.com/krwg/zapp/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/krwg/zapp/total?style=flat-square&labelColor=0a0a0a&color=e8e8e8)](https://github.com/krwg/zapp/releases)
[![License](https://img.shields.io/badge/license-MIT-555?style=flat-square&labelColor=0a0a0a)](LICENSE)
![Windows 10/11](https://img.shields.io/badge/Windows-10%2F11%20x64-0078D6?style=flat-square&labelColor=0a0a0a)

[English](README.md) · [Русский](README.ru.md) · [Site](https://krwg.github.io/zapp/) · [Releases](https://github.com/krwg/zapp/releases)

<img src="docs/screenshots/01-overview.png" alt="zapp Home" width="860" />

</div>

I made zapp because I was tired of launching BAT files with a console window you shouldn’t close, guessing which Flowseal strategy fits your ISP, and babysitting `winws` by hand.

zapp wraps zapret and the [Flowseal strategy pack](https://github.com/Flowseal/zapret-discord-youtube) in a small Tauri app. Press Start on the **Bypass** card, approve UAC once, and `winws` runs hidden in the background.

**0.2.7** is the product leap after the 0.2.6 trust slice: **path verdicts** (YouTube / Discord web / Discord voice called out separately), **recent working modes**, a **quiet watchdog**, a **first-run wizard**, **conflict shield**, EN/RU fault text that matches the UI language, a faster Start→UAC path, and a minimal [landing page](https://krwg.github.io/zapp/). Smart check stays Fast-by-default with Thorough optional; Home stays honest when voice UDP still fails.

The app, engine and strategies update separately. English and Russian UI, light/dark themes, tray support.

zapp is not a VPN. Your IP stays visible and nothing is encrypted; the only thing that changes is how your packets look to DPI. Check that this is legal where you live.

## Install

1. Download `zapp_*_x64-setup.exe` from [Releases](https://github.com/krwg/zapp/releases/latest) and install it.
2. Open zapp (first run may show a short wizard), press **Start** on Bypass, approve the UAC prompt.
3. Wait while the engine pack downloads, then run **Smart check**.

Windows 10/11 x64. Admin rights are needed once, to start the helper.

The NSIS `setup.exe` is the one I recommend: on a reinstall it can wipe the old app data. The `.msi` works too but has no cleanup page. SHA256 sums are in each release. Installers are currently **not Authenticode-signed** (see [code signing policy](docs/CODE_SIGNING_POLICY.md)); SmartScreen may warn — verify the hash or build from source.

## What it does on your machine

`winws` uses [WinDivert](https://reqrypt.org/windivert.html), which loads a kernel driver, so it needs admin rights. zapp starts one elevated helper, `zapp-helper.exe`, and talks to it over a token-authenticated **named pipe** (`\\.\pipe\zapp-helper`) with a Medium-IL ACL so the unelevated UI can still connect. That stays on the local machine — no TCP listen port.

On first run, and when Engine or Strategies update, zapp downloads the Flowseal pack from GitHub. New installs prefer `%ProgramData%\zapp\engine\` (elevated helper + locked ACL); older / fallback packs stay under `%APPDATA%\app.zapp.desktop\engine\`. App updates come from [krwg/zapp](https://github.com/krwg/zapp/releases) and are checked with minisign. Apart from GitHub release checks and the site probes you start yourself, zapp makes no other requests. There is no telemetry. Settings and logs stay in `%APPDATA%\app.zapp.desktop\`, helper state in `%LOCALAPPDATA%\zapp\`.

Antivirus and SmartScreen sometimes flag WinDivert-based tools, because a packet-filter driver looks suspicious by nature. The source is open, so you can check the installer's SHA256 or build it yourself.

## Known rough edges

Short version before you install:

- Smart check and path verdicts use **TCP/HTTPS plus soft UDP smoke**, not a real Discord/YouTube client — voice can still fail while web looks green (0.2.7 surfaces that gap on purpose).
- First launch needs GitHub access to download the engine pack (Updates also has zip import). There is no full offline pack browser yet.
- Closing the window hides to the tray by default; the process is still running. **Quit from the tray** before reinstalling (Quit also stops `winws`/helper), or the setup cannot replace `zapp.exe`.
- Windows 10/11 x64 only. No macOS or Linux build.

## If something doesn't work

If SmartScreen blocks the installer, choose *More info → Run anyway*, or verify the SHA256 first. If no strategy passes Smart check, close other zapret, VPN or DPI tools (Conflict shield will name common ones) and run it again. If a Flowseal `.bat` left `winws.exe` running, stop it first — Smart check will tell you.

To remove zapp completely, uninstall it and accept the cleanup prompt. By hand, delete `%APPDATA%\app.zapp.desktop`, `%LOCALAPPDATA%\zapp`, and optionally `%ProgramData%\zapp`.

## Build from source

```bash
npm install
npm run tauri:dev     # desktop app
npm run tauri:build   # NSIS + MSI
```

You need the Rust toolchain and MSVC Build Tools. Architecture notes are in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Release notes: [RELEASE_0.2.7.md](RELEASE_0.2.7.md).

## Credits

The engine isn't mine. zapp is the UI and the glue around it.

| Project | Author |
|---------|--------|
| [zapret](https://github.com/bol-van/zapret) | [bol-van](https://github.com/bol-van) |
| [zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube) | [Flowseal](https://github.com/Flowseal) |

zapp doesn't ship WinDivert or `winws`; their licenses stay with the upstream projects.

MIT · [krwg](https://github.com/krwg)
