# Contributing

Thanks for looking at zapp. The project is a small Windows-only Tauri client around Flowseal / zapret — keep changes focused.

## Before you start

- Read [README.md](README.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- Search [issues](https://github.com/krwg/zapp/issues) for duplicates
- For security issues, use [SECURITY.md](SECURITY.md) instead of a public bug

## Dev setup (Windows)

```bash
npm install
npm run tauri:dev
```

You need:

- Node 22+ (or current LTS used in CI)
- Rust stable + MSVC Build Tools
- Windows 10/11 x64

UI-only mock (no native shell):

```bash
npm run dev
```

Release-shaped build:

```bash
npm run tauri:build
```

## Scope that fits well

- UI / UX polish that matches the existing soft-minimal look
- Reliability of Start / Stop / helper / Smart check
- Clearer errors, docs, and probes
- Tests or small refactors that make the above safer

## Scope to avoid unless discussed first

- Non-Windows ports
- Bundling WinDivert / `winws` in this repo
- Telemetry or cloud accounts
- Large redesigns or new product surfaces without an issue

## Pull requests

1. One concern per PR when possible
2. Describe **why**, not only what
3. Note how you tested (`tauri:dev`, Smart check, Start/Stop, UAC path)
4. Do not commit secrets, `keys/zapp.key`, or local engine packs
5. Match existing code style; don’t mass-reformat unrelated files

Use the PR template. Link the issue if there is one.

## Issues

- Bugs: use the Bug report template (or About → Report a bug in the app)
- Ideas: use the Feature request template
- Questions / “doesn’t work on my ISP”: prefer a clear bug with environment, or a discussion if enabled

## License

By contributing, you agree your changes are licensed under the MIT License (same as the project).
