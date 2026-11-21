# Privacy

zapp has **no telemetry**, **no accounts**, and **no analytics SDK**.

## What leaves your machine

| Traffic | When | Why |
|---------|------|-----|
| GitHub API / release assets | Update check, first engine download, Application/Engine/Strategies update | Fetch versions and packs |
| YouTube / Discord / related hosts (and hosts you add) | Smart check; silent probes on Home while bypass is on; watchdog while enabled | See if a mode works |
| Nothing else by default | — | — |

There is no crash reporter phone-home. Bug reports are opened by you (GitHub issue / in-app form); you choose what to paste.

## What stays local

| Path | Contents |
|------|----------|
| `%APPDATA%\app.zapp.desktop\` | Settings, session (including recent working modes), logs, staging, fallback engine |
| `%LOCALAPPDATA%\zapp\` | Helper binary + `helper.token` |
| `%ProgramData%\zapp\` | Preferred engine packs (machine-wide) |

## Token and helper

The helper token is a local secret so only callers that know it can issue Start/Stop over the pipe. The file is readable by your Windows user. zapp does not upload it. See [HOW_IT_WORKS.md](HOW_IT_WORKS.md).

## Not a privacy tool

zapp is a DPI bypass client, not a VPN. Your IP stays visible to sites you visit. Packets are not end-to-end encrypted by zapp. Use it only where that is legal and appropriate.
