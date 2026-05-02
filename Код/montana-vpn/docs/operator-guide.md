# Operator guide — Montana VPN

Как поднять свой Reality-эндпоинт на чистом VPS с нуля, выдать клиенту, обслуживать.

## Шаг 0 — выбор VPS

Требования:
- Linux Ubuntu 24.04 / Debian 12 / Fedora 40+ / RHEL-family / Alpine
- Минимум 512 MiB RAM (xray ~60 MiB, nginx ~10 MiB; рекомендуется 1 GiB+ если хостить узел Montana рядом)
- 5 GiB disk (под систему + логи; узел Montana добавит ~50 MiB/год)
- 1 vCPU достаточно
- Public IPv4 со свободным :443 и :80
- Рут-доступ либо sudo

**Хорошие провайдеры (нейтральные юрисдикции):**

- THE.Hosting (Финляндия) — текущий montana-finland
- Hetzner (Финляндия / Германия) — стабильный, недорогой, хорошая network policy
- Mythic Beasts (UK) — privacy-friendly
- 1984 Hosting (Исландия) — nation-level legal protection
- Njalla (no-KYC, разные локации)

**Юрисдикции которые избегать для VPN:** US, UK (5 Eyes core), Russia, China, Iran, любая страна где VPN-операция требует регистрации.

## Шаг 1 — установка одной командой

После git clone и `sudo`:

```bash
sudo bash montana-vpn/install.sh
```

Что произойдёт:
1. apt/dnf/apk install nginx + ufw + curl + jq
2. Скачивание xray через официальный installer XTLS/Xray-install
3. Генерация Reality keypair (X25519), UUID клиента, shortId — всё локально на VPS
4. Сборка `/usr/local/etc/xray/config.json` из шаблона
5. Поднятие nginx :80 с decoy `It works!`
6. Установка systemd unit с hardening
7. Открытие 22/80/443 в ufw, остальное закрытие
8. Включение BBR + fq_codel
9. Запуск xray, печать VLESS URL для клиента

В конце выведется одна строка `vless://...` — это всё что нужно клиенту.

## Шаг 2 — первичная защита SSH

В `install.sh` это **не** делается (опционально, чтобы не сломать доступ оператору). Делается вручную после установки:

```bash
# Включить ключ-only auth
sudo sed -i 's/^#*PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config
sudo systemctl restart ssh

# Поставить fail2ban
sudo apt install -y fail2ban
sudo tee /etc/fail2ban/jail.local >/dev/null <<'JAIL'
[DEFAULT]
bantime = 1h
findtime = 10m
maxretry = 3
backend = auto
bantime.increment = true
bantime.factor = 2
bantime.maxtime = 1w

[sshd]
enabled = true
JAIL
sudo systemctl enable --now fail2ban

# (опционально) crowdsec для IP-reputation feed
curl -s https://install.crowdsec.net | sudo sh
sudo apt install -y crowdsec-firewall-bouncer-iptables
```

## Шаг 3 — выдача клиенту

В выводе `install.sh` строка вида:

```
vless://e6d355e2-2d79-4c96-a373-3b0e6b6f4b0d@91.132.142.42:443?encryption=none&flow=xtls-rprx-vision&security=reality&sni=www.googletagmanager.com&fp=chrome&pbk=AbCd...&sid=302805bc0c25e504&type=tcp#montana-vpn
```

Передать клиенту через **зашифрованный канал** (Signal, ProtonMail с PGP, Threema, не Telegram-чат-историю и не email plaintext).

Клиент импортирует в:
- iOS / macOS: **Streisand** (App Store), **FoXray**, **Hiddify**
- Android: **v2rayNG**, **Hiddify**, **NekoBox**
- Windows: **v2rayN**, **Hiddify**
- Linux: **Hiddify-Next** или CLI `xray run -config /path/to/config.json`
- Router (OpenWrt): **xray-core** + sing-box

## Шаг 4 — мониторинг

```bash
# статус сервиса
systemctl status xray

# логи (live)
journalctl -u xray -f

# access log (кто подключался когда)
sudo tail -f /var/log/xray/access.log

# error log
sudo tail -f /var/log/xray/error.log

# текущие соединения
sudo ss -tlnp | grep :443

# CPU/RAM xray
ps aux | grep -v grep | grep xray
```

## Шаг 5 — обновление xray

xray релизит security patches. Минимум раз в квартал:

```bash
sudo bash <(curl -L https://github.com/XTLS/Xray-install/raw/main/install-release.sh) @ install
sudo systemctl restart xray
```

Конфиг в `/usr/local/etc/xray/config.json` сохраняется, ключи в `/etc/montana-vpn/state.env` тоже.

## Шаг 6 — ротация ключей (если подозрение на компрометацию)

```bash
sudo rm /etc/montana-vpn/state.env
sudo bash montana-vpn/install.sh
```

Сгенерируются новые UUID + Reality keypair + shortId. Старые клиенты перестанут работать — нужно раздать новый VLESS URL.

## Шаг 7 — несколько клиентов

`install.sh` создаёт **одного** клиента в массиве `inbounds[0].settings.clients`. Чтобы добавить второго:

```bash
sudo nano /usr/local/etc/xray/config.json
```

В блоке `clients`:

```json
"clients": [
  {
    "id": "e6d355e2-2d79-4c96-a373-3b0e6b6f4b0d",
    "email": "alice",
    "flow": "xtls-rprx-vision"
  },
  {
    "id": "ВТОРОЙ-UUID-СГЕНЕРИРОВАТЬ-ЧЕРЕЗ-xray-uuid",
    "email": "bob",
    "flow": "xtls-rprx-vision"
  }
]
```

```bash
sudo xray uuid                    # сгенерить UUID для bob
sudo systemctl restart xray
```

Reality keypair (`pbk`/`sid`) **общий** для всех клиентов одного эндпоинта. Различается только UUID. Это нормально и архитектурно правильно.

## Шаг 8 — отключение VPN (но узел Montana остаётся)

```bash
sudo systemctl stop xray
sudo systemctl disable xray
sudo ufw delete allow 443/tcp
sudo ufw delete allow 80/tcp
sudo systemctl stop nginx
sudo systemctl disable nginx
```

## Шаг 9 — полное удаление

```bash
sudo systemctl stop xray
sudo systemctl disable xray
sudo bash <(curl -L https://github.com/XTLS/Xray-install/raw/main/install-release.sh) @ remove
sudo rm -rf /etc/montana-vpn /usr/local/etc/xray /var/log/xray /var/www/decoy
sudo apt remove -y nginx
```

Узел Montana (если установлен) — отдельная команда:

```bash
sudo systemctl stop montana-node
sudo systemctl disable montana-node
sudo rm /etc/systemd/system/montana-node.service
sudo systemctl daemon-reload
sudo rm -rf /var/lib/montana /usr/local/bin/montana-node /opt/montana
sudo userdel montana
```

## Troubleshooting

**Клиент не подключается, no error.**
Проверь `pbk` (public key) — это **публичный** ключ из `state.env`. Если случайно вставил `private_key` — будет тихий fail.

**Клиент подключается, ping есть, но websites не открываются.**
DNS leak. В клиенте принудительно через VPN: `1.1.1.1`. Либо проверь что в `xray-config.json` outbound `freedom` действительно direct, не proxy.

**xray падает с `failed to listen on 0.0.0.0:443: bind: permission denied`.**
`AmbientCapabilities=CAP_NET_BIND_SERVICE` не сработал. Проверь systemd-version (≥232) и что drop-in присутствует. На старых Linux: `setcap 'cap_net_bind_service=+ep' /usr/local/bin/xray`.

**Reality handshake fail.**
`dest` сайт перестал поддерживать TLS 1.3 + X25519 либо стал недоступен. Смени `DECOY_HOST=www.cloudflare.com` и переустанови.

**ufw блокирует :443.**
`sudo ufw status verbose`. Если правил нет — `sudo ufw allow 443/tcp && sudo ufw reload`.

**RAM кончается.**
Helsinki 961 MiB достаточно для xray-only. Если вместе с узлом Montana и swap нет — добавить swapfile:

```bash
sudo fallocate -l 1G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```

## Backup

Что бэкапить (для пере-деплоя без потери ключей):

```bash
# одной командой — на локальную машину
ssh montana-finland 'sudo tar czf - /etc/montana-vpn /usr/local/etc/xray /var/lib/montana 2>/dev/null' \
  > montana-finland-backup-$(date +%Y%m%d).tar.gz
gpg -c montana-finland-backup-*.tar.gz   # зашифровать паролем
```

Хранить **отдельно** от мнемоники узла Montana. Backup VPN ≠ backup узла.
