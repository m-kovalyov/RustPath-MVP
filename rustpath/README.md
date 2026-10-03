# RustPath v0.2

Русскоязычная платформа практического изучения Rust. **Это развивающийся учебный продукт, не законченный курс до middle и не готовая публичная платформа исполнения недоверенного кода.**

## Что нового

- **44 урока / 11 модулей / 3 независимых маршрута**: «С нуля» (12), «Основы Rust» (16), «Продвинутый Rust» (16).
- Новые вводные темы: программа и main, let/mut, типы, shadowing, bool, const, объявление функций, аргументы, результат и композиция.
- Продвинутые темы: generics, associated types, dyn Trait, lifetimes, Cow, структуры со ссылками, Result/?, checked arithmetic, FnMut, type-state, потоки, Arc/Mutex, каналы и Atomic.
- **3 серверных квиза** с объяснениями. Остальные 41 урок — код с rustc/unit-тестами. Квизы работают без compiler runner.
- Серверные закладки и раздел повторения. Сохраняются с гостевой сессией в PostgreSQL.
- Поиск тем внутри маршрута, компактные раскрываемые модули и прогресс выбранного маршрута.
- Уроки разделены на «Разобраться», «Практика», «Материалы»: меньше информации одновременно.
- Учебная ownership-лаборатория: пошаговые move/borrow/drop, лёгкий CSS-объём и переходы; reduced-motion поддерживается.
- Материалы для чтения в каждом уроке: Metanit, Brown University, Stanford CS110L, The Rust Book. Задания и тексты написаны отдельно; университеты не связаны с этим проектом.

**Все 16 старых ID и XP сохранены.** Вводные уроки не закрывают прежний маршрут. Пройденные уроки доступны для повторения независимо от prerequisites. Общий процент может уменьшиться из-за добавленных уроков — это не потеря прогресса. Максимум нового курса: 2330 XP, повтор не добавляет опыт.

## Обновление работающей версии на Windows

Читайте **`docs/UPDATE-WINDOWS.md`**. Обновляйте **в той же папке**, из которой раньше запускали Docker Compose. Оставьте старый `.env`, тот же адрес `http://localhost:8080` и cookie браузера. Новые секреты создавать не нужно. Не используйте `docker compose down -v`.

## Первый локальный запуск

Нужны Docker Engine/Docker Desktop с рабочими cgroup CPU/memory/pids, Docker Compose v2, минимум 4 ГБ RAM и около 8 ГБ свободного диска. На Windows нужен WSL 2; после установки Docker Desktop он должен быть запущен.

```bash
cp .env.example .env
```

В `.env` замените два placeholder независимыми случайными hex-секретами. Например, `openssl rand -hex 32` для каждого. Не публикуйте `.env`. Остальные значения по умолчанию подходят для локального запуска.

```bash
docker pull rust:1.99.0-slim-bookworm
docker compose up --build -d
```

Откройте **http://localhost:8080**. Первая сборка занимает несколько минут. Миграции применяются автоматически; v0.2 добавляет таблицу bookmarks и не удаляет прежние данные.

```bash
curl http://localhost:8080/api/health
docker compose ps
docker compose logs --tail=80 web api runner
```

Origin в `.env` должен точно совпадать с адресом браузера (`localhost` и `127.0.0.1` различаются). Остановка: `docker compose down`; следующий запуск: `docker compose up -d`. `down -v` удаляет прогресс.

## Стек

Vue 3 + TypeScript, Pinia, Vue Router, Vite; CodeMirror загружается только при открытии кодовой практики. Rust edition 2024 + Axum/Tokio/SQLx; PostgreSQL; приватный Python runner управляет одноразовыми Rust-контейнерами. Публичный API и бизнес-логика на Rust. Traits, композиция, Repository, Adapter, DI и DTO описаны в `docs/ARCHITECTURE.md`.

## Проверки и разработка

Без контейнеров приложения: PostgreSQL + `DATABASE_URL`, `RUNNER_URL`, `RUNNER_SECRET`, `APP_ORIGIN=http://localhost:5173`, `COOKIE_SECURE=false`; `cargo run --locked` в backend и `npm ci && npm run dev` в frontend. Vite проксирует /api в 127.0.0.1:3000. Не заменяйте runner прямым запуском learner code на API-хосте.

```bash
cd backend
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
# Только выделенная тестовая БД:
INTEGRATION_DATABASE_URL=postgresql://USER:PASSWORD@localhost/rustpath_test cargo test --locked -- --ignored
cd ../frontend
npm ci && npm run build && npm test
cd ..
python3 tests/check_curriculum.py
pip3 install -r runner/requirements.txt
python3 tests/test_runner.py
```

Для UI contract тестов запустите API/frontend, затем из frontend:

```bash
npx playwright install chromium
npm run test:ui
```

По умолчанию UI тест использует http://localhost:5173; `APP_BASE_URL` задаёт другой адрес. Проверяются реальные quiz/bookmark/draft API + PostgreSQL. Ответы compiler-error/success в UI тесте явно подменены — это не доказательство Docker sandbox. По желанию `QA_CAPTURE_DIR` сохраняет статические снимки DOM; `BROWSER_EXECUTABLE` задаёт локальный Chromium.

Полный локальный deployment smoke:

```bash
python3 tests/smoke.py
```

Проверяет все 44 урока и XP/закладки; требует рабочего Docker sandbox, занимает около 5 минут из-за лимита попыток. Не запускайте smoke на публичной production-инсталляции.

## Структура

- `backend/`: API, домен, адаптеры, миграции, course.json.
- `frontend/`: интерфейс, store, маршруты, редактор, ownership-модель.
- `runner/`: приватный сервис исполнения.
- `tests/`: curriculum generator, immutable legacy fixture, API/runner/UI/smoke проверки.
- `docs/`: архитектура, источники, обновление Windows, безопасность, QA и планы.

## Известные ограничения

- Гостевой доступ зависит от cookie. Нет постоянных аккаунтов, восстановления и синхронизации между устройствами.
- Sandbox поддерживает std, не произвольные Cargo crates. Async/Tokio/backend/DB/capstone уроки ещё в плане.
- Unit-тесты проверяют поведение, но не всегда использование нужного приёма (например, shadowing или spawn). Это явно отмечено в уроках.
- Обычные контейнеры и Docker socket у development runner не подходят как окончательная публичная security boundary. Нужны dedicated execution node и gVisor/microVM, audit, ingress quotas и очередь.
- Полный Docker E2E v0.2 здесь не подтверждён; пользователь подтвердил локальный запуск предыдущей v0.1 на Windows. GitHub Actions workflow подготовлен, не запускался в этой среде.

Перед публикацией выполняйте `docs/SECURITY.md` и `docs/DEPLOYMENT.md`. Подробные результаты — `docs/QA.md`, учебная/дизайн-методика — `docs/SOURCES.md`.
