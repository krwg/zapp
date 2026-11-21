# How zapp works

Short companion to the README. For the developer map, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Bypass in three steps

1. You press **Start** on Bypass.
2. If the elevated helper isn’t running, Windows shows **UAC** once. zapp stages `zapp-helper.exe` under `%LOCALAPPDATA%\zapp\` and launches it elevated.
3. The helper starts Flowseal’s `winws` (WinDivert) with the active strategy. Later Start/Stop calls go over the local pipe — no second UAC until the helper dies or you Quit.

## Helper and pipe

- Pipe name: `\\.\pipe\zapp-helper` (no TCP listen)
- Every request carries a token from `%LOCALAPPDATA%\zapp\helper.token`
- The pipe ACL allows your normal (Medium IL) user to connect to the elevated helper
- The helper only starts `winws` from managed pack paths (AppData or ProgramData)

Any process running as your Windows user that can read the token can talk to the helper. That is intentional for the unelevated UI; it is not a cross-user boundary. Details: [PRIVACY.md](PRIVACY.md) and [SECURITY.md](../SECURITY.md).

## Where the engine lives

Preferred (new / elevated installs):

`%ProgramData%\zapp\engine\` — ACL locked down (admins/SYSTEM write, users read/execute).

Fallback / older installs:

`%APPDATA%\app.zapp.desktop\engine\`

Resolve order and env overrides: [ARCHITECTURE.md](ARCHITECTURE.md).

## Updates

| Channel | Source | Trust |
|---------|--------|--------|
| Application | [krwg/zapp releases](https://github.com/krwg/zapp/releases) | minisign `.sig` (not Authenticode) |
| Engine | Flowseal GitHub zip | sanity checks; prefer ProgramData install via helper |
| Strategies | Same Flowseal pack | profiles from `general*.bat` |

Offline: Updates → import zip for Engine. Signing policy: [CODE_SIGNING_POLICY.md](CODE_SIGNING_POLICY.md).

## Smart check and Home

- **Smart check** starts strategies one by one, probes YouTube/Discord targets, scores them. Fast stops early on a strong score; Thorough tests all and ranks.
- **Path verdicts** split YouTube HTTPS, Discord web, and Discord voice (light UDP) so a green web path doesn’t hide broken voice.
- **Home** with bypass on may run silent site probes; the **watchdog** (optional, on by default) soft-checks while bypass is desired.

These probes are not a real browser or Discord client.

## Quit vs close

Closing the window hides to the tray by default. **Quit** from the tray stops `winws` and the helper so an installer can replace `zapp.exe`.
