# RustPath

Интерактивная платформа изучения Rust на русском языке. **Это MVP, не готовый production-сервис и не полный курс до middle.**

## Уже реализовано

- Vue 3 + TypeScript, Composition API, Pinia и Vue Router.
- Адаптивный интерфейс, светлая/тёмная тема по настройкам системы.
- 4 учебных модуля, 16 оригинальных уровней, теория, задачи и пошаговые подсказки.
- CodeMirror 6: подсветка Rust, номера строк, горячая клавиша Ctrl/⌘ + Enter.
- Rust API: Axum, Tokio, SQLx; PostgreSQL и миграции.
- Гостевая сессия: случайный токен в HttpOnly cookie, в БД только SHA-256 хеш.
- Серверные черновики, прогресс, XP, последовательное открытие уровней, streak по дням UTC.
- Проверка Rust-кода через приватный runner: rustc + unit-тесты в отдельном контейнере.
- Транзакционное начисление XP: повторные успешные решения не дают дополнительный опыт.
- Docker Compose, unit/integration тесты и подготовленная GitHub Actions CI.

Runner — вспомогательный Python-сервис управления Docker; бизнес-логика и публичный backend написаны на Rust. Docker-сокет не доступен API.

## Быстрый локальный запуск

Требования: Docker Engine с работающими cgroup memory/pids/CPU, Docker Compose v2, минимум 4 ГБ RAM и около 8 ГБ свободного диска. Docker Desktop на Windows/macOS подходит при включённой виртуализации.

```bash
cp .env.example .env
```

В `.env` замените оба значения `replace_with_64_random_hex_characters` независимыми случайными hex-секретами. Их можно сгенерировать командой `openssl rand -hex 32`. **Не публикуйте `.env`.** Пароль БД должен быть hex, чтобы URL подключения не требовал экранирования.

```bash
docker pull rust:1.99.0-slim-bookworm
docker compose up --build -d
docker compose logs -f api
```

Откройте **http://localhost:8080**. Первая сборка скачивает зависимости и может занять несколько минут. Миграции применяются при старте API. `APP_ORIGIN` должен точно совпадать с адресом в браузере: `localhost` и `127.0.0.1` не взаимозаменяемы.

```bash
curl http://localhost:8080/api/health
# {"status":"ok"}
python3 tests/smoke.py
# Полная проверка 16 уровней через локальный API/runner (~2 минуты).
```

Остановка: `docker compose down`. Данные PostgreSQL сохраняются в volume. `docker compose down -v` **удаляет весь прогресс**.

Если возникает cgroup/memory-limit ошибка, исправьте/обновите Docker-хост. Не отключайте изоляцию и не запускайте пользовательский код непосредственно на API-хосте.

## Разработка без контейнеров приложения

Поднимите собственную PostgreSQL и приватный runner на подходящем Docker-хосте. Создайте пустую БД `rustpath`. API не требует заранее применять миграции.

```bash
cd backend
export DATABASE_URL=postgresql://USER:PASSWORD@localhost:5432/rustpath
export RUNNER_URL=http://localhost:4000
export RUNNER_SECRET=YOUR_PRIVATE_RUNNER_SECRET
export APP_ORIGIN=http://localhost:5173
export COOKIE_SECURE=false
cargo run --locked
```

В другом терминале:

```bash
cd frontend
npm ci
npm run dev
```

Vite проксирует `/api` в `127.0.0.1:3000`. Открывайте `http://localhost:5173`. Нельзя безопасно заменить runner вызовом `rustc` внутри основного API.

## Проверки

```bash
cd backend
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
# Используйте только выделенную тестовую БД:
INTEGRATION_DATABASE_URL=postgresql://USER:PASSWORD@localhost/rustpath_test cargo test --locked -- --ignored
cd ../frontend
npm ci && npm run build && npm test
cd ..
python3 tests/check_curriculum.py
pip3 install -r runner/requirements.txt
python3 tests/test_runner.py
```

`check_curriculum.py` запускает только доверенные встроенные эталонные решения. Никогда не используйте этот скрипт для произвольного пользовательского кода.

## Структура

```text
backend/       Rust API, доменные модели, репозиторий, runner adapter, курс, миграции
frontend/      Vue интерфейс, Pinia store, типы API, компоненты, CodeMirror
runner/        Приватный HTTP сервис контейнерной компиляции
tests/         curriculum QA, runner tests, deployment smoke
docs/          архитектура, безопасность, roadmap, результаты QA
compose.yaml   Локальная конфигурация запуска
```

## Что пока не входит

Аккаунты и восстановление доступа, синхронизация между устройствами, email/пароли/OIDC, CMS, интервальное повторение, продвинутые уроки 05–09, проекты с ревью, anti-cheat, дипломы, платежи, очереди выполнения, мониторинг и production-развёртывание. Они не имитируются в интерфейсе.

Гостевой прогресс хранится в БД, но доступ к нему связан с cookie в конкретном браузере. При очистке cookie восстановление не реализовано. Серия дней считается по UTC, что явно указано в интерфейсе.

## Достоверность проверки

Смотрите `docs/QA.md`: frontend/Rust сборка, API с PostgreSQL и эталонные решения проверены. В среде создания проекта Docker-контейнеры не запускаются из-за cgroup-ограничения; полный контейнерный запуск и CI pipeline здесь **не подтверждены**. Успешное состояние интерфейса отдельно проверено с подменённым HTTP-ответом, а не выдаётся за реальный запуск sandbox.

## Публичный запуск

Не открывайте текущий development Compose в интернет как готовую платформу исполнения кода. Сначала выполните `docs/SECURITY.md`, `docs/DEPLOYMENT.md` и настоящий `tests/smoke.py` на целевом хосте. Нужны отдельный execution-хост, усиленная изоляция (gVisor/microVM), TLS, аккаунты, rate limiting на ingress и аудит.

## Референсы

- Rustlings: https://github.com/rust-lang/rustlings — маленькие практические упражнения.
- 100 exercises to learn Rust: https://github.com/mainmatter/100-exercises-to-learn-rust — последовательное learning-by-doing. Его контент имеет CC BY-NC 4.0; в этот проект он не копировался.
- The Rust Programming Language: https://doc.rust-lang.org/book/ — техническая основа маршрута.
- Vue: https://vuejs.org/guide/introduction.html — Composition API и SFC.
- Axum: https://docs.rs/axum/ — API на Rust.

Учебные задания здесь написаны заново. Не использованы чужие логотипы, иллюстрации или тексты уроков.
