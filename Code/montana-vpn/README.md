# Montana VPN

Self-hosted Reality-based VPN endpoint. Запускается рядом с узлом Montana либо отдельно. Не зависит от узла, не использует консенсус Montana — это отдельный слой.

Реализован по образцу production-эндпоинта в Финляндии (`montana-finland`, `<exit-removed>`).

## Что внутри

```
montana-vpn/
├── README.md                                — этот файл
├── install.sh                                — установка одной командой на чистый VPS
├── config-template/
│   ├── xray-config.json.template             — xray Reality config с {{плейсхолдерами}}
│   ├── nginx-decoy.conf                      — :80 камуфляж (active-probing defense)
│   ├── decoy-index.html                      — "It works!" страница
│   ├── xray.service                          — systemd unit
│   ├── xray.service.d/
│   │   └── 10-donot_touch_single_conf.conf   — xray installer drop-in
│   └── sysctl-bbr.conf                       — fq_codel + BBR (anti-bufferbloat)
└── docs/
    ├── architecture.md                       — как именно собрана Helsinki, детально
    ├── threat-model.md                       — что закрывает Reality + decoy, что нет
    ├── operator-guide.md                     — пошаговый развёртывание + обслуживание
    └── client-config.md                      — формат VLESS URL + настройка клиентов
```

## Быстрый старт (оператор)

На чистом Ubuntu 24.04 / Debian 12 / Fedora / Alpine VPS:

```bash
git clone https://github.com/montana-protocol/montana.git /opt/montana
cd /opt/montana
sudo bash Протокол/Code/montana-vpn/install.sh
```

В конце скрипт выведет `vless://...` URL — это всё что клиент импортирует в свой Hiddify/v2rayN/Streisand.

## Установка вместе с узлом Montana

Один VPS — узел Montana + VPN рядом, два независимых systemd-сервиса:

```bash
sudo bash Протокол/Code/scripts/install-vps-full.sh
```

Поднимет:
- `montana-node.service` — узел консенсуса (singleton, без сетевого слоя M6)
- `xray.service` — VPN endpoint
- `nginx.service` — :80 decoy

Узел и VPN **не зависят** друг от друга. Можно остановить любой, второй продолжит работать.

## Только VPN (без узла Montana)

```bash
sudo bash Протокол/Code/montana-vpn/install.sh
```

## Только узел Montana (без VPN)

```bash
sudo bash Протокол/Code/scripts/install-vps.sh
```

## Что использует Montana VPN

- **xray-core** v26+ (xtls/xray-core, MIT) — VLESS + Reality + xtls-rprx-vision
- **nginx** 1.24+ — :80 decoy
- **ufw** — default deny + allow 22/80/443
- **systemd** hardening — `User=nobody`, `NoNewPrivileges`, `CapabilityBoundingSet`

Всё компоненты open-source, audit-trail известны, активно поддерживаются.

## Юрисдикция

VPN-узел берёт характеристики юрисдикции хостера. Helsinki-референс — Финляндия (нейтральная, нет mass-surveillance законов). Список нейтральных юрисдикций с провайдерами в `docs/operator-guide.md` шаг 0.

## Связь с протоколом Montana

**Никакой структурной зависимости.** VPN не использует консенсус, ключи Montana, identity-файл узла, mt-account, AccountTable, ничего из протокольного слоя.

Они **сосуществуют** на одном хосте если оператор хочет и хостить узел, и иметь exit-эндпоинт. Это удобно (один VPS, две функции), но не обязательно.

После M6 (когда у узла появится сетевой слой) **возможно** будет опциональная маршрутизация трафика узла через тот же Reality-эндпоинт — но это будущее, сейчас не предусмотрено.

## License

Конфиги, скрипты и документация в этом каталоге — публичное достояние (CC0 / Unlicense). Используемые компоненты (xray, nginx) — каждый под своей лицензией (MIT / BSD-2-Clause).
