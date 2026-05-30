# Montana node — quickstart

Поднять полноценный узел Montana mainnet за одну команду.

## Вариант 1: pre-built docker image (быстрее, ~30 сек)

> **Перед первым использованием:** владелец репозитория должен сделать GHCR
> пакет публичным:
> https://github.com/users/efir369999/packages/container/package/montana-node
> → Package settings → Change visibility → Public.

```bash
docker volume create montana-data
docker run -d \
  --name montana-node \
  --network host \
  --restart unless-stopped \
  -v montana-data:/var/lib/montana \
  ghcr.io/efir369999/montana-node:latest
```

Контейнер:
- Слушает TCP :8444 (Noise_PQ XX → Yamux транспорт).
- Подключается к bootstrap-peer (Moscow `<front>:8445`) + 4 force_active
  узлам (frankfurt/vilnius/armenia/nicosia) из embedded `genesis-manifest.json`.
- Первый запуск — генерирует 24-словную мнемонику и пишет её в
  `/var/lib/montana/mnemonic.txt` (mode 0400). **Сохрани её сразу:**
  ```bash
  docker exec montana-node cat /var/lib/montana/mnemonic.txt
  ```

## Вариант 2: build from source (универсальный, ~5 мин)

```bash
git clone https://github.com/efir369999/Montana.git /opt/montana
cd /opt/montana/Code/docker/runtime
docker compose up -d --build
```

Пересоберёт `montana-node:local` из current `main` branch.

## После запуска

### Проверить, что узел в сети

```bash
# Локально на VPS — current_window растёт каждые ~30 сек:
docker exec montana-node /usr/local/bin/montana-node status --data-dir /var/lib/montana

# В живой сети — узел появится в /api/peers одного из bootstrap-peer узлов:
curl -sk https://efir.org:8443/montana-api/peers
```

После 24 часов sync узел появится в `/api/nodes` как Candidate (фаза подтверждения).
Через ~14 дней последовательной SHA-256 цепочки переходит в Active и начинает участвовать в лотерее.

### Опционально: VPN exit-нода

Если хочешь чтобы узел ещё и работал как Reality VLESS endpoint:

```bash
docker compose up -d xray nginx-decoy
```

`xray` слушает :443 (TLS Reality, маскируется под googletagmanager.com), `nginx-decoy` на :80 — обманный landing.

## Текущее состояние сети

- 5-node mainnet cohort: moscow + frankfurt + vilnius + armenia + nicosia
- Build 26 (sha `b6e79bdc1e8b...`), v1.0.1-build26 tag
- Bootstrap-only proposer (DEV-022/023 rotation disabled — см. `docs/SPEC_DEVIATIONS.md`)
- Multi-confirmer cement (typically bundles=2-3) + multi-winner lottery (DEV-021) распределяет emission
- Cемитированное окно ~`/api/status` грow rate: 1 окно / 30-60 сек

## Поддержка

Открой issue: https://github.com/efir369999/Montana/issues
