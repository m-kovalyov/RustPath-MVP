# Архитектура и решения

## Поток запросов

```text
Браузер Vue → Nginx (same origin) → Rust Axum API → PostgreSQL
                                          ↓ HTTP + private bearer secret
                               Приватный Python runner
                                          ↓ Docker Engine
                           одноразовый rustc + test контейнер
```

На публичный порт выводится только web. PostgreSQL, API и runner находятся в internal Docker network. Код пользователя не запускается в API-процессе. Клиент не сообщает правильность ответа, XP или свой learner_id: всё вычисляет сервер.

## Принципы ООП в идиоматическом Rust

- **Инкапсуляция:** доменные модели, скрытые ответы, методы `Course::lesson/unlocked`; приватные поля в учебных заданиях.
- **Абстракция:** `LearningRepository` отделяет сценарии обучения от PostgreSQL; `CodeEvaluator` отделяет API от инфраструктуры компиляции.
- **Полиморфизм:** `Arc<dyn LearningRepository>` и `Arc<dyn CodeEvaluator>`, реализации подключаются через dependency injection.
- **Композиция:** AppState собирает зависимости; структуры и traits вместо иерархий наследования.
- **Паттерны:** Repository, Adapter, Strategy (evaluator), DI, DTO. SQLx транзакция даёт атомарность attempt + completion.

Rust не требует классов и не поддерживает обычное наследование классов. «ООП» здесь означает управление поведением и границами, а не перенос Java-подобной структуры.

## Данные

`learners`: гостевые профили и хеш токена; `drafts`: текущий код урока; `submissions`: результат попытки; `completions`: успешные уровни с unique `(learner_id, lesson_id)`.

XP — сумма сохранённых completion, не счётчик в памяти. `ON CONFLICT DO NOTHING` в транзакции защищает от повторного начисления при конкурентных ответах. User code не хранится в логах API или runner. Хранятся только текущие черновики; история исходников не реализована.

Курс хранится в versioned JSON вместе с backend. Ответы и harness не сериализуются в публичные DTO. Добавление урока: обновить генератор `tests/build_course.py`, регенерировать JSON, запустить curriculum QA, проверить условия/Unicode/пустой ввод и миграцию прогресса при изменении ID. Не меняйте уже опубликованные ID без миграции.

## API

| Метод | URL | Назначение |
|---|---|---|
| GET | `/api/health` | Проверка подключения БД (не проверки runner) |
| POST | `/api/session` | Создать/восстановить гостевую сессию |
| PUT | `/api/session` | Ротация токена текущей сессии |
| GET | `/api/course` | Модули, summary уроков и план продолжения |
| GET | `/api/progress` | XP, completion, streak |
| GET | `/api/lessons/:id` | Только доступный урок, без решения/test source |
| GET/PUT | `/api/lessons/:id/draft` | Прочитать/сохранить черновик |
| POST | `/api/lessons/:id/submit` | Проверить код, записать попытку и completion |

Ошибки: JSON `{ "error": "текст" }`; 400 ввод/Origin, 401 сессия, 403 prerequisite, 404 lesson, 413 body limit, 429 attempts, 503 runner. Некоторые ошибки Axum extractor (невалидный JSON/тип) возвращают стандартный текст Axum; клиент обрабатывает их запасным сообщением.

## Масштабирование

API не хранит учебный прогресс в памяти и может масштабироваться за балансировщиком. Текущий per-learner limiter и semaphore локальны экземпляру: перед несколькими репликами заменить limiter на Redis/ingress, добавить общую очередь и worker pool. Нельзя считать этот MVP уже доказанно масштабируемым: нагрузочные измерения ещё не выполнены.

На старте один экземпляр применяет миграции через SQLx; для production вынести миграции в release job. PostgreSQL — managed БД с PITR, индексами и контролем connection budget. Курс JSON → версионируемый CMS с публикацией/ревью после подтверждения авторского процесса. Микросервисы для остальных функций пока не нужны.
