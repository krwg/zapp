# zapp — каталог ошибок и сценариев детекта

Коды живут в `src-tauri/src/engine/faults.rs`. Preflight — `engine/preflight.rs`.

## Lifecycle / Engine

| Код | Сценарий | Как детектим | UX |
| --- | --- | --- | --- |
| `engineNotInstalled` | Нет pack / пустой slot | нет `bin/winws.exe` | Updates → Engine |
| `winwsBinaryMissing` | Битый pack | preflight path check | переустановить Engine |
| `winDivertDriverMissing` | Нет `WinDivert64.sys` / `.dll` | preflight | переустановить Engine |
| `strategyBatMissing` | Профиль без bat | `root/{name}.bat` | другой профиль |
| `strategyParseFailed` | bat без argv winws | parser | другой профиль / update Strategies |
| `strategyPayloadMissing` | битые hostlist/fake files | parse args → exists() | update Strategies |
| `uacCancelled` | пользователь отклонил UAC | PowerShell exit 1223 | повторить Start |
| `uacLaunchFailed` | UAC ок, winws не появился | poll tasklist ≤12s | AV / WinDivert |
| `winwsExitedImmediately` | процесс умер сразу | settle poll | конфликт / AV / args |
| `winDivertConflict` | служба/драйвер в странном состоянии | `sc query` + stop leftovers | Stop → Start |
| `accessDenied` | spawn без прав | OS error text | UAC path |
| `conflictingBypass` | GoodbyeDPI / ZapretGUI / EzUnBlock | tasklist image names | закрыть чужой GUI |
| `likelyBlockedByAv` | эвристика AV | текст ошибки / мгновенный exit | исключения Defender |
| `stopFailed` | taskkill не смог | stop status | ручной kill |
| `diagnosticsNeedsAdmin` | diag без elevate path | flag | Start один раз |

## Дополнительные сценарии (детект в roadmap)

1. **WinDivert orphan RUNNING без winws** — `sc query` RUNNING + tasklist empty → auto `net stop`.
2. **Несколько winws PID** — orphan от старых bat → kill tree перед start.
3. **Стратегия с `%GameFilter%` при включённом game filter** — читать `utils/game_filter.enabled`.
4. **Пути с пробелами / кириллицей** — tokenizer + quoted args regression.
5. **Несовместимый Flowseal major** — semver gate на pack.
6. **DNS broken (нет резолва youtube.com)** — preflight DNS отдельно от TCP.
7. **TCP 443 ok, TLS fail** — будущая HTTPS-проба (как Zapret-GUI matrix).
8. **HTTP3/QUIC only block** — отдельный score канал.
9. **Discord voice UDP fail при web ok** — UDP probe / stun.
10. **Гигантский hostlist → медленный старт** — timeout budget.
11. **Антивирус карантин WinDivert.sys** — file exists но load fail.
12. **Secure Boot / driver blocklist** — load fail после elevate.
13. **Корпоративный proxy / MITM** — HTTPS probe identity mismatch.
14. **IPv6-only / IPv4-only** — dual-stack probe.
15. **VPN параллельно** — routing asymmetry в probe latency.
16. **Таймзона / clock skew** — не критично, лог.
17. **Disk full при update unzip** — update channel fault.
18. **Partial zip / bad SHA** — update validation.
19. **Engine update while running** — require stop.
20. **Profile switch mid-diagnostics** — cancel or queue.
21. **UI poll during long UAC** — status stays `starting`, не error до timeout.
22. **Double-click Start** — busy lock (уже есть).
23. **Stop во время starting** — cancel elevate wait.
24. **Sleep/hibernate** — refresh_status_from_os на resume.
25. **Single-instance second launch** — focus existing (Zapret-GUI pattern).

## UI контракт

- Любая ошибка команды → toast `error` + `connection.lastError` на Overview.
- Никаких unhandled rejection.
- Poll snapshot не чаще 2s в idle.
