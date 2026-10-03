# Проверка RustPath 0.2.0

Дата: 2 октября 2026. Результаты относятся к этой сборке, а не к будущему production-развёртыванию.

| Проверка | Результат | Граница доказательства |
|---|---|---|
| Rust build, fmt, clippy -D warnings | PASS | Rust 1.99, все targets |
| Rust unit tests | 8 PASS | Курс, prerequisites, приватные ответы, streak и ограничения |
| API + PostgreSQL integration | 1 PASS | Настоящая PostgreSQL 16; evaluator подменён |
| Curriculum | 41 решений / 111 проверок PASS; 3 quiz contracts PASS | Только доверенные эталонные решения через rustc |
| Отрицательные starter-решения | Все 41 отклонены | Задания не проходят без изменений |
| Frontend typecheck / production build | PASS | Lazy routes и отдельная загрузка редактора |
| Frontend progression tests | 6 PASS | Независимые маршруты и совместимость старого прогресса |
| npm audit | 0 vulnerabilities на момент проверки | Не заменяет security audit |
| Runner tests | 7 PASS | Mocked Docker; лимиты, авторизация и cleanup |
| Compose config --quiet | PASS | Проверка конфигурации, не запуск контейнеров |
| Python compile | PASS | Runner и тестовые скрипты |
| Browser UI | 24 состояния, 1440/390 px | Нет JS errors и горизонтального переполнения |
| Визуальная проверка | PASS после исправлений | Снимки шаблонов desktop/mobile, dark, ошибки и успех |
| Полный Docker execution 0.2 | NOT VERIFIED HERE | В этой среде недоступен запуск с требуемыми cgroup-лимитами |
| GitHub Actions | NOT RUN HERE | CI подготовлена, не запускалась на GitHub |
| Нагрузка / penetration / WCAG certification | NOT PERFORMED | Не заявляются как выполненные |

## Что проверено реально

HTTP API и PostgreSQL: cookie/Origin, приватность черновиков и закладок между гостями, закрытые уроки, скрытие решений и ответов квизов, неверная попытка без XP, правильная попытка, повтор без повторного XP, открытие следующего урока. Миграция закладок добавляет таблицу, не удаляет старые данные. Исходная миграция 001 и ID/тесты/XP 16 прежних уроков сохранены.

В браузере: квиз, сохранение/удаление закладки, сохранение кода и восстановление после перезагрузки через настоящий API/PostgreSQL. Выбор маршрута, поиск, аккордеоны, вкладки, лаборатория и reduced-motion. Успех и ошибка **компилятора в UI — явные HTTP fixtures**, не доказательство работы sandbox. Подробности: `ui-qa.json`, `curriculum-qa.json`.

Интерфейс стал компактнее: на главной раскрыт один модуль; теория, практика и источники разделены. Исправлены контраст неактивных состояний лаборатории, отображение radio в экспортированных превью и gutter редактора. Полная screen-reader проверка не выполнялась.

Пользователь сообщил об успешном локальном запуске 0.1 на Windows. Это не заменяет повторный E2E-тест 0.2 на его компьютере.

## Повторение проверок

```bash
cd backend
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
# Перед этой командой задайте INTEGRATION_DATABASE_URL для отдельной тестовой PostgreSQL:
cargo test -- --ignored
cd ../frontend
npm ci
npm test
npm run build
cd ..
python tests/check_curriculum.py
```

UI: запустить API с тестовой БД и frontend dev server, затем `npm run test:ui` в frontend. Требуется Playwright Chromium (`npx playwright install chromium` на собственном компьютере). Проверка создаёт гостевые сессии в БД; не запускать на production.

Полный smoke на целевой Windows Docker Desktop / Linux:

```bash
docker pull rust:1.99.0-slim-bookworm
docker compose up --build -d
python tests/smoke.py
```

Smoke занимает несколько минут из-за ограничения частоты запросов и проверяет все 44 урока. Далее отдельно проверить infinite loop, OOM, сеть/файловую систему, лимиты output, cleanup, нагрузку и восстановление backup. Не отключать изоляцию ради успешного теста. Перед публичным запуском выполнить `SECURITY.md` и `DEPLOYMENT.md`.
