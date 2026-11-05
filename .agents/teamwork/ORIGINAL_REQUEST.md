# Original User Request

## Approved Task Prompt — 2026-11-04T22:47:09Z

Используй множество агентов (минимум 3 разных агента) для создания нового интерфейса, глубокого анализа сети и поиска востребованных функций других лаунчеров и плагинов. Комплексная переработка и оптимизация OmniLauncher (TMlauncher) до уровня полноценного флагманского лаунчера («Steam 2») с высокой производительностью, премиальным интерфейсом по референсам и максимальным набором функций современных лаунчеров.

Working directory: c:\Users\Tmub\Documents\TMlauncher
Integrity mode: development

Референсы визуального стиля:
- Референс 1: C:/Users/Tmub/.gemini/antigravity/brain/a8034bc2-4e98-4db6-ab2e-75b98f1b9cae/.user_uploaded/media_1793830323839_968236a7.jpg (минималистичный ультра-стильный тёмный дизайн "TURA", чистая типографика Cera CY, глубокий графит/чёрный, элегантные карточки).
- Референс 2: C:/Users/Tmub/.gemini/antigravity/brain/a8034bc2-4e98-4db6-ab2e-75b98f1b9cae/.user_uploaded/media_1793830335320_3ec4cbc9.jpg (премиальный футуристичный тёмный интерфейс с акцентами жидкого хрома/серебра, плавными формами, лаконичными видео/фото блоками).

## Requirements

### R1. Редизайн и модульная архитектура интерфейса
Полностью переработать интерфейс OmniLauncher согласно предоставленным референсам, убрав лишние надписи и поля с пояснениями. Добавить модульность интерфейсу с возможностью добавления любых тем оформления и изменения визуальных элементов без ограничений.

### R2. Интеграция сторонних платформ (Steam, Epic Games, Roblox)
Добавить возможность привязки аккаунта Steam и перехода в профиль пользователя, починку достижений Steam и функционала Steam Cloud, а также полноценное модальное окно свойств игры как в Steam. Реализовать подтягивание незагруженных игр из Epic Games в библиотеку и отображение недавно сыгранных игр из Roblox.

### R3. Магазин, загрузчик, каталог эмуляторов и сетевые настройки
Добавить бесконечную ленту рекомендаций в магазин как в Steam и возможность переключения валюты магазина (USD, тенге). Реализовать встроенный выбор диска и директории загрузки непосредственно в интерфейсе лаунчера (без открытия системного диалога Steam). Добавить возможность скачивания open-source эмуляторов напрямую с GitHub в библиотеку. Настроить загрузку изображений и файлов через защищённые серверы Cloudflare и предоставить в настройках лаунчера выбор надежных DNS (Cloudflare, Google и др.) и прокси-соединений.

### R4. Внутренняя структура файлов и программа запуска
Реорганизовать внутреннюю структуру папки с файлами проекта для ясности и удобства навигации. Переделать программу запуска так, чтобы лаунчер запускался чисто и презентабельно, без всплывающих окон командных строк и терминалов.

### R5. Исследование рынка и внедрение востребованных возможностей
Задействовать минимум 3 разных агента для проведения глубокого анализа сети: найти минимум 20 идей и востребованных функций/плагинов, которых не хватает пользователям современных игровых лаунчеров в интерфейсе и функционале. Самостоятельно отобрать и внедрить наиболее полезные и подходящие из них без промежуточных согласований с пользователем, предоставив подробный итоговый отчёт о проделанной работе.

## Acceptance Criteria

### Сборка и запуск
- [ ] Компиляция Rust-бэкенда в `src-tauri` проходит успешно (`cargo check` и `cargo build` выполняются без ошибок).
- [ ] Сборка фронтенда (`npm run build`) и проверка типов TypeScript проходят успешно без ошибок.
- [ ] Лаунчер запускается как единое графическое приложение без всплывающих окон терминала и командной строки.

### Интерфейс и темы
- [ ] Главный экран и компоненты интерфейса переработаны в соответствии с визуальными референсами (тёмный минимализм, акценты хрома/серебра, чистая типографика, отсутствие лишних поясняющих надписей).
- [ ] Реализована расширяемая модульная система тем с возможностью добавления новых тем и переключения оформления.

### Платформы и библиотека
- [ ] Доступен переход в профиль Steam и привязка аккаунта Steam.
- [ ] Реализовано отображение достижений Steam и синхронизация Steam Cloud.
- [ ] Реализовано окно «Свойства» для игр с функционалом параметров запуска и управления файлами как в Steam.
- [ ] В библиотеку корректно подтягиваются незагруженные игры из Epic Games.
- [ ] В библиотеку отображаются недавно запущенные игры из Roblox.
- [ ] Доступен каталог и прямое скачивание open-source эмуляторов с GitHub в библиотеку.

### Магазин, сеть и загрузки
- [ ] В магазине работает бесконечная лента рекомендаций игр.
- [ ] Реализовано переключение валюты магазина (USD, тенге) с пересчётом цен.
- [ ] Выбор диска и каталога загрузки осуществляется через собственный встроенный селектор лаунчера без диалогового окна Steam.
- [ ] Загрузка файлов и медиа настроена через серверы Cloudflare; в настройках доступен выбор DNS (Cloudflare, Google и др.) и прокси.

### Анализ рынка и отчёт
- [ ] Проведён глубокий анализ сети с участием минимум 3 агентов: найдено не менее 20 идей и функций современных игровых лаунчеров.
- [ ] Отобраны и внедрены наиболее полезные и подходящие функции в лаунчер без необходимости промежуточных подтверждений.
- [ ] Сформирован и предоставлен подробный итоговый отчёт о проделанной работе со списком найденных идей, внедрённых функций и архитектурных изменений.


## Follow-up — 2026-11-05T04:51:03Z

User directive received:
"Да, всё устраивает. Поищи ещё минимум 20 идей в интернете и добавь только те, которые могут подойти и принести пользу. Не спрашивай меня согласен ли я. предоставляй лишь отчёт о проделанной работе."

User fully approves current work and autonomously directs:
1. Research 20+ additional launcher features from the community (Playnite extensions, Steam discussions, Reddit r/pcgaming, Heroic, GOG Galaxy, Lutris).
2. Filter strictly for those that provide genuine utility and architectural elegance without bloat.
3. Automatically adopt and integrate the top fitting features into Milestone 4 (alongside Zero-Footprint mode, Save Time Machine, HLTB, Game Roulette, and Shader Cache reclaimer).
4. No approval questions—proceed directly and provide a structured final report upon completion.

Top vetted features to weave into M4:
- Steam Review Sentiment Badges & Metacritic ratings (% positive, Overwhelmingly Positive pill) on GameCard and details.
- Personal RPG / Walkthrough Notes tab with local storage autosave in GamePropertiesModal.
- Smart Dynamic Filter Pills (Installed, Unplayed Backlog, Favorites, Steam, Epic, Roblox, Emulators).
- Direct Folder Openers (Game files, Save directory, Mod directory).
- Handheld & Laptop Eco / Battery Saver Mode (throttling UI render rate & background polling).
- Custom Cover Art / Capsule replacer in GamePropertiesModal.
- Visual Disk Footprint & Storage Breakdown per game.

Proceed with Milestone 3 certification and dispatch Milestone 4!


## Follow-up — 2026-11-05T05:31:12Z

User directive received:
"с этого момента попробуй упростить и ускорить реализацию задачи немного"

Process acceleration instructions:
1. M4: Do not run new explorer rounds. After worker_m4 handoff, run ONE compact gate (1 reviewer + 1 auditor) without challengers or repeated rechecks unless critical bugs exist.
2. Combine M5 (folder structure reorganization, #![windows_subsystem = "windows"]) and M6 (final verification & release build) into a single unified step: 1 worker, 1 final test pass (cargo test / tsc / npm run build / tests/runner.cjs), 1 audit.
3. Keep secondary items in clean minimal working versions.
4. Crucial requirements preserved: clean GUI launch without terminal/CMD window popup, organized project folder structure, all previously accepted features fully functional with genuine implementations.
5. Final summary report to user: one comprehensive report upon completion.
