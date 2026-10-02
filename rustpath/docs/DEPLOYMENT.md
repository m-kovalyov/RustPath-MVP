# Публикация и эксплуатация

## Локально

Следуйте README. По умолчанию web слушает `127.0.0.1:8080`, все остальные сервисы приватны. Runner image скачивается заранее, поскольку internal network не предоставляет egress и runner не должен скачивать образ на каждом запросе.

## Закрытый staging

1. Выделенная Linux VM с Docker и рабочими cgroup memory/pids/CPU. Не машина с личными секретами.
2. Reverse proxy с TLS перед web; APP_ORIGIN=https://ВАШ-ДОМЕН, COOKIE_SECURE=true.
3. Доступ через VPN/allowlist. Не открывать анонимное исполнение кода публично.
4. Отдельные случайные секреты; managed PostgreSQL или защищённый volume с резервированием.
5. Подтвердить `tests/smoke.py` на локальной/private установке, тесты отказов runner и очистку контейнеров.
6. Проверить свой внешний TLS Origin: нельзя использовать smoke как нагрузку на production.

## Production

Вынести runner на dedicated execution node с gVisor/microVM. Использовать TLS/mTLS/private network и секрет вместо публичного доверия сети. Не давать runner сетевой доступ к PostgreSQL. `compose.yaml` — не готовый production deployment template.

API: immutable image, graceful shutdown, readiness + отдельные метрики runner, autoscaling только после централизованного limiter/очереди. Frontend: CDN/static hosting с reverse proxy на API same-origin. PostgreSQL: PITR, регулярный restore drill, connection pooling, миграции release job.

Бэкап для локальной установки:

```bash
docker compose exec -T db pg_dump -U rustpath rustpath > rustpath.sql
```

Файл содержит прогресс и черновики: хранить приватно, шифровать, применять retention. Проверка восстановления обязательна; «есть pg_dump» не доказывает работоспособность бэкапов.

Публичная регистрация, мониторинг, SLA, нагрузочное тестирование и security review не выполнены и не заявляются как готовые.
