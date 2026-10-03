# Дальнейшее развитие после v0.2

## Создано

44 урока, 3 независимых маршрута, quiz/code, серверные закладки, повторение, поиск, пошаговый интерфейс и ownership-модель. Новые темы помогают освоить variables/functions и intermediate/advanced основы языка, но не представляют собой полный production backend курс.

## Следующая содержательная версия

1. Блок async: Future, async/await, Tokio, задачи и отмена. Потребуется управляемый allowlist crates и новая схема sandbox build, а не загрузка произвольных dependencies пользователя.
2. Проектный практикум: CLI → HTTP API → PostgreSQL transactions/auth. Раздельные проекты и rubric review, не только функция в одном файле.
3. Аккаунты/OIDC, восстановление/удаление данных, синхронизация между устройствами.
4. Версионируемый CMS/review процесса, миграция при пересмотре IDs и тестов, пользовательские UX-тесты вводного блока.
5. Интервальное повторение: новый feature после накопления learning events, а не выдача сегодняшних bookmarks за spaced repetition.
6. Dedicated execution node, stronger isolation, queue/quotas/metrics и подтверждённые OOM/network/timeout/cleanup tests.

## Проекты для оценки самостоятельности

Собственная библиотека с API/rustdoc → bounded очередь задач → многопользовательский REST API → deployed сервис с транзакциями, нагрузочным отчётом и runbook. Внешнее ревью, исправление замечаний и защита компромиссов обязательны.

## Definition of Done публичной beta

Наблюдения за новичками; полнота условий/тестов; безопасность sandbox на целевом хосте; аккаунты/privacy; TLS/секреты/бэкапы с restore drill; centralized abuse controls; реальная нагрузка и доступность. Курс сам по себе не гарантирует должность middle или работу.
