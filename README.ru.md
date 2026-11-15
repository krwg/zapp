<div align="center">

<img width="112" height="112" alt="zapp" src="docs/screenshots/icon-128.png" />

# zapp

**Тихий десктопный клиент для [zapret](https://github.com/bol-van/zapret) на Windows.**  
Запускайте стратегии обхода DPI для YouTube и Discord без консольных окон.

[English](README.md) · [Русский](README.ru.md) · [Релизы](https://github.com/krwg/zapp/releases)

<img src="docs/screenshots/01-overview.png" alt="zapp, главная" width="860" />

</div>

## Что это

zapp оборачивает движок zapret и [пакет стратегий Flowseal](https://github.com/Flowseal/zapret-discord-youtube) в небольшой нативный интерфейс (Tauri 2 · React 19 · Rust).

- Запускает `winws` скрытно: без BAT-консолей, один запрос UAC.
- **Умная проверка** оценивает стратегии по YouTube и Discord (включая gateway и порты `discord.media`) и предпочитает режим, который проходит оба.
- **Сайты**: проверка доступности YouTube, Discord и ваших хостов.
- Отдельные каналы обновлений для приложения, движка и стратегий.
- Интерфейс RU / EN, темы Light / Dark / System, трей.

**Чем это не является:** это не VPN, не антивирус и не «защита» в смысле security-продукта. zapp не скрывает IP и не шифрует трафик — он меняет вид пакетов для DPI на пути к заблокированным сервисам. Убедитесь, что использование законно там, где вы живёте.

## Установка

1. Скачайте `zapp_*_x64-setup.exe` со страницы [релизов](https://github.com/krwg/zapp/releases/latest).
2. Установите и откройте zapp.
3. Нажмите **Start** и один раз подтвердите UAC.
4. Дождитесь загрузки пакета движка и запустите **Умную проверку**.

Требования: Windows 10/11 x64. Права администратора нужны один раз для запуска helper.

| Файл | Назначение |
|------|------------|
| `*_x64-setup.exe` (NSIS) | Рекомендуется. Есть страница очистки при переустановке. |
| `*_x64_en-US.msi` | Альтернатива. Без страницы очистки. |

SHA256 указаны в каждом релизе.

## Доверие и приватность

- **Зачем UAC:** `winws` использует [WinDivert](https://reqrypt.org/windivert.html), который загружает драйвер ядра и требует права администратора. zapp поднимает один повышенный helper (`zapp-helper.exe`) и общается с ним на `127.0.0.1` по локальному TCP с токеном — не через интернет.
- **Что скачивается:** при первом запуске (и при обновлении Engine/Strategies) пакет Flowseal из его GitHub Releases в `%APPDATA%\app.zapp.desktop\engine\`. Обновления приложения — с [krwg/zapp](https://github.com/krwg/zapp/releases). Другие исходящие запросы — только проверка релизов на GitHub и пробы сайтов, которые вы сами запускаете.
- **Без аккаунтов, телеметрии и аналитики.** Настройки, логи и пакет движка лежат в `%APPDATA%\app.zapp.desktop\` (состояние helper — в `%LOCALAPPDATA%\zapp\`).
- **Антивирус / SmartScreen:** инструменты на WinDivert часто помечают как riskware. Это ожидаемо для пакетного фильтра, а не доказательство вредоносности. Исходники zapp открыты: сверьте SHA256 или соберите сами.

## Если что-то не так

- **SmartScreen блокирует установщик:** *Подробнее → Выполнить в любом случае*, либо сначала сверьте SHA256.
- **Ни одна стратегия не проходит проверку:** закройте другие zapret / VPN / DPI-инструменты и повторите.
- **Обновление зависло на «Installing…» (0.2.0 / 0.2.1):** один раз запустите установщик из `%APPDATA%\app.zapp.desktop\downloads\` (эти сборки не умели запросить UAC).

Чтобы удалить zapp полностью, удалите программу и согласитесь на очистку данных (или вручную удалите `%APPDATA%\app.zapp.desktop` и `%LOCALAPPDATA%\zapp`).

## Сборка из исходников

```bash
npm install
npm run tauri:dev     # десктопное приложение
npm run tauri:build   # NSIS + MSI
```

Нужны Rust и MSVC Build Tools. Архитектура: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Авторы движка

zapp — только UI + core. Движок принадлежит его авторам.

| Проект | Автор |
|--------|-------|
| [zapret](https://github.com/bol-van/zapret) | [bol-van](https://github.com/bol-van) |
| [zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube) | [Flowseal](https://github.com/Flowseal) |

zapp не содержит WinDivert и `winws`. Их лицензии остаются за upstream-проектами.

## Лицензия

MIT · [krwg](https://github.com/krwg)
