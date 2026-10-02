# Результаты проверки

Дата: 2 октября 2026. Доказательства ниже относятся к этой сборке, а не к будущему production-развёртыванию.

| Проверка | Результат | Граница доказательства |
|---|---|---|
| Rust API cargo check/build | PASS | Настоящая сборка Rust 1.99.0 |
| cargo fmt --check, clippy -D warnings | PASS | Все targets, без предупреждений |
| Rust unit tests | 7 PASS | Курс, скрытие ответов, prerequisites, streak, лимиты кода, cookie |
| API + PostgreSQL integration | 1 PASS | Настоящая PostgreSQL 16.14; evaluator внедрён как fake |
| Curriculum rustc tests | 16 решений PASS / 45 cases | Запускаются только доверенные встроенные reference sources |
| Отрицательные starter решения | 16 отклонены | Проверяет, что задания не проходят «из коробки» |
| Frontend typecheck + production build | PASS | Vue/TypeScript/Vite, lazy route loading |
| Frontend progression unit tests | 3 PASS | Prerequisites, next lesson, unknown IDs |
| npm install audit | 0 vulnerabilities на момент установки | Не заменяет полный security audit |
| Runner HTTP/configuration tests | 7 PASS | Docker client mocked; параметры isolation и cleanup проверены |
| Compose v2.39.4 config --quiet | PASS | Конфигурация валидна, сервисы не запускались |
| Python compile | PASS | Runner и тестовые скрипты |
| Browser UI checks | 10 состояний, PASS | 1440/390 px, нет page JS errors и horizontal overflow |
| Ручная визуальная проверка | PASS после исправлений | Главная, урок, roadmap; mobile; errors; success fixture; dark mode |
| Полный Docker execution E2E | NOT VERIFIED | Docker Engine 25.0.16 не может создать контейнер из-за ограничений cgroup этой среды |
| GitHub Actions pipeline | NOT RUN HERE | Workflow подготовлен, запуск на GitHub ещё не выполнен |
| Нагрузка / penetration test / WCAG certification | NOT PERFORMED | Не заявляются как готовые |

## Интеграционный API тест

Реально проверены с PostgreSQL: Origin обязателен для мутации; без cookie 401; закрытый уровень 403; public lesson не отдаёт solution/tests; черновики изолированы между двумя гостями; неверная попытка не открывает уровень; первая успешная выдаёт 50 XP; повторная даёт 0 XP; следующая тема открывается только владельцу результата. Правильность решения здесь задаёт тестовый evaluator, а не Docker.

## Визуальные проверки

10 состояний: главная desktop/mobile, урок desktop/mobile, roadmap desktop/mobile, недоступный runner, success fixture, тёмная главная, ошибка связи/повтор. Исправлены вылезавший логотип и отсутствующие glyph icons: использованы векторные UI-иконки. Проверены отступы и отсутствие наложений/горизонтального переполнения. Полная screen-reader проверка не выполнялась.

Автосохранение черновика проверено в браузере: изменён код, дождались подтверждения сохранения, перезагрузили страницу и сравнили восстановленный текст. Это реальное сохранение через API/PostgreSQL.

Success state проверялся явным mocked HTTP ответом и не выдаётся за настоящее прохождение задания. Недоступный runner был проверен реальным запросом к API: показывается понятное сообщение 503, XP не начисляется.

## Как закрыть оставшийся пробел

На Linux/Docker Desktop с корректной поддержкой cgroup:

```bash
docker pull rust:1.99.0-slim-bookworm
docker compose up --build -d
python3 tests/smoke.py
```

После этого отдельно проверить infinite loop, OOM, сетевые/файловые попытки, переполнение output и cleanup. Не отключать изоляцию для достижения зелёного результата. Готовая CI содержит Docker smoke шаг, но он здесь не запускался.
