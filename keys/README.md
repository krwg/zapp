# Updater signing keys

- `zapp.key.pub` — public key (also embedded in `src-tauri/tauri.conf.json`).
- `zapp.key` — **private**, never commit. Keep offline / in CI secrets.

## Local / CI env

```text
TAURI_SIGNING_PRIVATE_KEY      # file contents or path (for tauri build)
TAURI_SIGNING_PRIVATE_KEY_PASSWORD
```

Generate a replacement pair only if you rotate keys (existing installs need the same pubkey):

```powershell
npx tauri signer generate -w keys/zapp.key --ci -p "YOUR_PASSWORD" -f
```

Then paste the new pubkey into `tauri.conf.json` → `plugins.updater.pubkey`.
