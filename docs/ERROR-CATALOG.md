# zapp — каталог ошибок и сценариев детекта (0.2.7)

Коды живут в `src-tauri/src/engine/faults.rs`. Preflight — `engine/preflight.rs`.  
Тексты для UI берутся из EN/RU каталогов и следуют `ui_locale` (Settings → язык).

## Lifecycle / Engine

| Код | Сценарий | Как детектим | UX |
| --- | --- | --- | --- |
| `engineNotInstalled` | Нет pack / пустой slot | нет `bin/winws.exe` | Updates → Engine / wizard |
| `winwsBinaryMissing` | Битый pack | preflight path check | переустановить Engine |
| `winDivertDriverMissing` | Нет `WinDivert64.sys` / `.dll` | preflight | переустановить Engine |
| `strategyBatMissing` | Профиль без bat | `root/{name}.bat` | другой профиль |
| `strategyParseFailed` | bat без argv winws | parser | другой профиль / update Strategies |
| `strategyPayloadMissing` | битые hostlist/fake files | parse args → exists() | update Strategies |
| `uacCancelled` | пользователь отклонил UAC | `ShellExecuteExW` cancel | повторить Start |
| `uacLaunchFailed` | UAC / staging / pipe ACL | stage helper, pipe connect, poll | AV / перезапуск zapp / повтор Start |
| `winwsExitedImmediately` | процесс умер сразу | settle poll | конфликт / AV / args |
| `winDivertConflict` | служба/драйвер в странном состоянии | `sc query` + stop leftovers | Stop → Start |
| `accessDenied` | spawn без прав | OS error text | UAC path |
| `conflictingBypass` | GoodbyeDPI / ZapretGUI / EzUnBlock | ToolHelp image names | Conflict shield / закрыть чужой GUI |
| `likelyBlockedByAv` | эвристика AV | текст ошибки / мгновенный exit | исключения Defender |
| `stopFailed` | taskkill не смог | stop status | ручной kill |
| `diagnosticsNeedsAdmin` | diag без elevate path | flag | Start один раз |
| `launchRefused` | helper path/policy refuse | `refused:` в ответе helper | Updates → Engine, повтор Start |

## UI контракт (ошибки)

- Toast / `connection.lastError` — **только** понятный текст (`title — hint`) на языке UI.
- Технические детали (`refused:…`, пути, raw OS) — в лог и в history для bug report.
- «Сообщить об ошибке» из toast или карточки ошибки открывает диалог репорта с prefill (~90%), шаги оставляет пользователю.

## Path verdicts / watchdog (0.2.7)

- Home **path verdict strip** разделяет YouTube HTTPS, Discord web и Discord voice UDP — без единого «всё зелёное».
- Watchdog периодически мягко перепроверяет пути при включённом обходе; toast `watchdog:bypass_stale`; опциональный auto re-check в Settings.
- DNS resolve для diag/watchdog — параллельный + кэш; инвалидация при смене сети.

## Дополнительные сценарии (плановый детект)

1. **WinDivert orphan RUNNING без winws** — `sc query` RUNNING + tasklist empty → auto `net stop`.
2. **Несколько winws PID** — orphan от старых bat → kill tree перед start.
3. **Стратегия с `%GameFilter%` при включённом game filter** — читать `utils/game_filter.enabled`.
4. **Пути с пробелами / кириллицей** — tokenizer + quoted args regression.
5. **Несовместимый Flowseal major** — semver gate на pack.
6. **DNS broken (нет резолва youtube.com)** — preflight DNS отдельно от TCP.
7. **TCP 443 ok, TLS fail** — будущая HTTPS-проба (как Zapret-GUI matrix).
8. **HTTP3/QUIC only block** — soft UDP:443 smoke в Smart check (`protocol=quic`); не гейтит pass alone.
9. **Discord voice UDP fail при web ok** — уже отражается в path verdict / soft UDP; полный RTC всё ещё вне scope.
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
21. **UI poll during long UAC** — status stays `starting`, не error до timeout; UAC теперь раньше в `start_inner`.
22. **Double-click Start** — busy lock (уже есть).
23. **Stop во время starting** — cancel elevate wait.
24. **Sleep/hibernate** — `refresh_status_from_os` demotes dead winws; if `restoreBypassOnResume` + session desired → auto `start` after wake (wall-clock gap).
25. **Single-instance second launch** — focus existing (Zapret-GUI pattern).

## UI контракт (прочее)

- Любая ошибка команды → toast `error` + `connection.lastError` на Overview.
- Никаких unhandled rejection.
- Idle snapshot poll ~4s; OS/conflict refresh throttled on the Rust side.
- `diagnostics:progress` обновляет live-диагностику без полного refresh.
