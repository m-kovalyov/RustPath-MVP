# Безопасность: граница MVP

## Уже предусмотрено

- HttpOnly, SameSite=Strict cookie, криптографически случайный 256-bit token, SHA-256 hash в БД.
- Exact Origin для каждой мутации; frontend/API same-origin, CORS не открывается.
- Параметризованный SQL, body/code limits, контролируемые prerequisite и XP на сервере.
- До 10 отправок/мин на гостя в одном API процессе; до 2 параллельных проверок.
- Runner не опубликован, проверяет отдельный bearer-secret.
- Контейнеры: non-root UID, read-only rootfs, без сети/capabilities, no-new-privileges, default Docker seccomp, 512 MB RAM, 1 CPU, 64 PID, timeout 20 sec, лимиты временных файлов и логов, удаление контейнера в finally.
- Компиляция и выполнение происходят внутри одного изолированного контейнера; разрешён только std, Cargo dependencies не загружаются.
- API блокирует атрибуты и include-макросы для учебного harness. **Это не средство изоляции.**

## Критический риск Docker socket

Runner в development Compose имеет Docker socket. Это даёт root-эквивалентные возможности на Docker-хосте, хотя API socket не имеет. Сам Docker использует общее ядро; обычный контейнер не является достаточной границей для запуска кода неизвестных пользователей в публичном сервисе.

**Production blocker:** перенести runner на отдельный узел без БД/секретов приложения, применить gVisor или microVM, минимальный проверенный образ, ограничения egress и отдельную сетевую политику. Никогда не монтировать app files, домашние каталоги или секреты в sandbox. Не публиковать Docker API. Текущий Compose — только локальная разработка на машине без ценных данных.

## Что необходимо перед открытием интернета

1. Аудит runner и API; настоящие sandbox acceptance tests на целевой инфраструктуре.
2. Ingress rate limiting для session/draft/submit, глобальный лимит очереди, quotas/IP abuse controls. Новый guest сейчас обходит per-learner limiter.
3. Аккаунты, восстановление/удаление данных, срок хранения гостевых профилей и черновиков; consent/privacy policy.
4. TLS, COOKIE_SECURE=true, точный APP_ORIGIN, секреты из secret manager, ротация, least-privilege DB user.
5. Безопасные образы с immutable digest, CVE scanning, SBOM; npm/Cargo lockfiles обновлять осознанно.
6. Тесты malicious code: infinite loop, memory exhaustion, fork bomb, filesystem reads/writes, network access, excessive output, timeout cleanup.
7. Метрики отказов/таймаутов, аварийный выключатель runner, reap-job для осиротевших контейнеров при crash/restart.
8. Правила проверки решений: учебные unit-тесты не защищены от намеренного обхода harness и не годятся для сертификации. `exit(0)` без итогового harness сообщения отклоняется, но это не anti-cheat.

Не запускайте недоверенный код через shell API-процесса. Не ослабляйте cgroup/seccomp/capability ограничения ради «работающей демки».
