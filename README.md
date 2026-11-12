# zapp

```
╔══════════════════════════════════════════╗
║  Soft-minimal desktop client for zapret  ║
║  React + TypeScript UI · Rust / Tauri    ║
╚══════════════════════════════════════════╝
```

**v0.2** — zapp drives `winws` itself: no Flowseal consoles, clean stop, real strategy diagnostics, independent Application / Engine / Strategies updates.

## Stack

| Layer | Tech |
| --- | --- |
| UI | React 19, TypeScript, CSS Modules, Lucide |
| Shell | Tauri 2 |
| Backend | Rust (`src-tauri`) |
| Browser demo | Mock API (`npm run dev`) |

## Develop

```bash
npm install
npm run dev          # UI + mock
npm run tauri:dev    # full desktop
npm run build
npm run tauri:build
```

## UX

1. Open **zapp** → **Start** (one UAC for the background helper).
2. Engine runs hidden — original zapret UI never appears.
3. **Stop** kills `winws` cleanly.
4. **Diagnostics** scores profiles, applies the best, enables it.
5. Engine updates come from Flowseal releases, independent of the app.

## Architecture

```
UI
  → ZappApi (tauriClient | mockClient)
  → Rust commands
      → AppState
      → ManagedEngine   (hidden winws)
      → elevated helper (one UAC, then silent IPC)
      → UpdateManager
      → Diagnostics
```

## Engine pack

Upstream: [Flowseal/zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube)

Resolution order: `ZAPP_ENGINE_DIR` → app data `engine/current` → local `vendor/zapret` (dev) → sibling Flowseal folders.

## Design

Monochrome Soft Minimalism. Light / Dark / System. Status colors only. No emoji, no neon.

## License

MIT · [krwg](https://github.com/krwg)
