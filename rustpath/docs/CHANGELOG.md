# Changelog

## 0.2.0

- +28 уроков: 12 вводных (9 code + 3 quiz) и 16 продвинутых. Всего 44/11/3.
- Сохранены 16 прежних ID, XP, tests и цепочка внутри core; completed всегда можно повторить.
- Явные prerequisites, независимые входы в intro/core/advanced.
- Quiz answer endpoint: серверная проверка, объяснение после ответа, idempotent XP, общий per-learner attempt limiter.
- Bookmarks API + add-only SQL migration 002. Повторение по закладкам и completion.
- Поиск, accordion, разбивка урока на understanding/practice/materials и outcome/common-mistake.
- Лёгкая пошаговая ownership-лаборатория: CSS-переходы и небольшой объём; reduced-motion и текстовая альтернатива.
- Материалы Metanit/Brown/Stanford/The Rust Book; оригинальные уроки, без affiliations.
- CodeMirror динамически загружается только в code practice.
- Real API/PostgreSQL integration + UI tests; compiler fixture честно отделён от execution proof.
- Windows update guide сохраняет папку проекта, volume, cookie и .env.

## 0.1.0

Первый MVP: Vue/Rust/PostgreSQL, 16 code lessons, guest/drafts/XP, Docker runner scaffold. Пользователь сообщил об успешном локальном запуске на Windows; это не приравнивается к нашему подтверждённому E2E v0.2.
