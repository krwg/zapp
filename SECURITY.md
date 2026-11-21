# Security policy

## Supported versions

Security fixes land on the latest released version of zapp (currently **0.2.x**). Older releases are not patched separately.

## What zapp is (threat model in one paragraph)

zapp runs an elevated helper (`zapp-helper.exe`) that can start `winws` / WinDivert. The UI is unelevated and talks to the helper over a local named pipe with a per-machine token. The token file is readable by your Windows user. zapp is not a VPN and does not protect against malware running as you.

## Please report

- Privilege escalation from a normal user to the elevated helper without the token / UAC
- Cross-user access to the helper pipe or token
- Path allowlist bypass that lets the helper start arbitrary binaries
- Supply-chain issues in release artifacts (tampered installer vs published SHA256 / minisign `.sig`)
- Credential or token leakage off the machine

## Please do not use this channel for

- “SmartScreen blocked the installer” (unsigned Authenticode — see [docs/CODE_SIGNING_POLICY.md](docs/CODE_SIGNING_POLICY.md))
- Antivirus false positives on WinDivert
- Strategy / ISP bypass not working (use a bug report or Discussions)

## How to report

Open a **private** vulnerability report if available on the repo, or email / contact the maintainer via GitHub (@krwg) without posting exploit details in a public issue.

Include:

- zapp version and OS build
- Whether the helper was elevated
- Steps to reproduce
- Impact (what an attacker would gain)

Please give a reasonable window before public disclosure.

## Release integrity

- Installer SHA256 is published on each [Release](https://github.com/krwg/zapp/releases)
- In-app Application updates are verified with minisign (pubkey in `src-tauri/tauri.conf.json`)
- Installers are currently **not** Authenticode-signed
