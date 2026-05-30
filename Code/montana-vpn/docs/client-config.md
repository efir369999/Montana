# Client config — формат VLESS URL и параметры

Полная разборка строки `vless://...` которую `install.sh` выдаёт оператору, и как её настроить вручную если нужно.

## Формат URL

```
vless://{UUID}@{HOST}:{PORT}?{params}#{label}
```

Реальный пример с обезличенными значениями:

```
vless://e6d355e2-2d79-4c96-a373-3b0e6b6f4b0d@<exit-de>:443?encryption=none&flow=xtls-rprx-vision&security=reality&sni=www.googletagmanager.com&fp=chrome&pbk=AbCdEfGhIjKlMnOpQrStUvWxYz1234567890&sid=302805bc0c25e504&type=tcp#montana-vpn
```

Разбор полей:

| Поле | Значение | Что делает |
|---|---|---|
| `UUID` | `e6d355e2-2d79-4c96-a373-3b0e6b6f4b0d` | Authentication-токен клиента. Совпадает с `clients[].id` в xray config |
| `HOST` | `<exit-de>` | Public IP сервера (либо domain если есть) |
| `PORT` | `443` | TCP порт сервера |
| `encryption=none` | none | VLESS не имеет своей encryption — она в Reality (TLS 1.3) |
| `flow=xtls-rprx-vision` | vision | Optimization после handshake — без TLS-in-TLS overhead |
| `security=reality` | reality | TLS-stealing handshake (vs `tls` для обычного TLS, `none` для plain) |
| `sni=www.googletagmanager.com` | dest | SNI который клиент шлёт в TLS ClientHello — должен совпадать с `dest` сервера |
| `fp=chrome` | chrome | uTLS fingerprint — точная репликация Chrome 124 ClientHello |
| `pbk=...` | publicKey | Reality public key — клиент использует для derive shared secret |
| `sid=302805bc0c25e504` | shortId | 8-байтовый идентификатор подключения. Серверу разрешено иметь несколько SID, клиент выбирает один |
| `type=tcp` | tcp | Transport-protocol (vs `ws` websocket, `grpc`) |
| `label` | `montana-vpn` | Локальное имя в клиенте (отображается, не передаётся серверу) |

## Что обязательно совпасть между клиентом и сервером

Сервер хранит:
- `clients[].id` (UUID) — должен совпасть с `UUID` клиента
- `realitySettings.privateKey` — sk соответствующий клиентскому `pbk`
- `realitySettings.shortIds[]` — должен содержать клиентский `sid`
- `realitySettings.serverNames[]` — должен содержать клиентский `sni`
- `realitySettings.dest` — куда сервер прокси handshake; **не** видно клиенту, но должно соответствовать `sni`

Любое расхождение → handshake fail, клиент подключиться не сможет.

## Ручная настройка клиента (если URL потерян)

### v2rayN (Windows)

1. Servers → Add → VLESS
2. Address: `<exit-de>`, Port: `443`
3. UUID: твой клиентский UUID
4. Flow: `xtls-rprx-vision`
5. Network: `tcp`
6. Security: `reality`
7. SNI: `www.googletagmanager.com`
8. Fingerprint: `chrome`
9. PublicKey: `pbk` из URL
10. ShortId: `sid` из URL

### v2rayNG (Android)

Меню → Импорт из буфера → вставить VLESS URL. Готово.

### Hiddify (cross-platform)

Импорт по ссылке либо QR-коду. Hiddify сам распарсит URL.

### CLI xray (сервер-как-клиент / Linux)

`/etc/xray/client-config.json`:

```json
{
  "log": { "loglevel": "warning" },
  "inbounds": [
    {
      "tag": "socks-in",
      "port": 1080,
      "listen": "127.0.0.1",
      "protocol": "socks",
      "settings": { "auth": "noauth", "udp": true }
    }
  ],
  "outbounds": [
    {
      "tag": "reality-out",
      "protocol": "vless",
      "settings": {
        "vnext": [{
          "address": "<exit-de>",
          "port": 443,
          "users": [{
            "id": "e6d355e2-2d79-4c96-a373-3b0e6b6f4b0d",
            "encryption": "none",
            "flow": "xtls-rprx-vision"
          }]
        }]
      },
      "streamSettings": {
        "network": "tcp",
        "security": "reality",
        "realitySettings": {
          "show": false,
          "fingerprint": "chrome",
          "serverName": "www.googletagmanager.com",
          "publicKey": "AbCdEfGhIjKlMnOpQrStUvWxYz1234567890",
          "shortId": "302805bc0c25e504",
          "spiderX": ""
        }
      }
    }
  ]
}
```

Запустить:

```bash
xray run -config /etc/xray/client-config.json
```

Поднимет SOCKS5 прокси на `127.0.0.1:1080`. Браузер / приложение конфигурировать на этот SOCKS5.

## QR-код для мобильных

```bash
echo "vless://...полный URL..." | qrencode -t ANSIUTF8
```

(требует `qrencode` пакет: `apt install qrencode`)

Либо онлайн (опасно, URL утечёт): не рекомендуется.

## Безопасность доставки конфига

VLESS URL содержит **полный credential**. Кто его получил → имеет доступ к VPN. Передавать **только** через:

- Signal (E2E шифрование)
- Threema (no-phone-number, E2E)
- ProtonMail с PGP (если получатель имеет PGP ключ)
- Локальная QR-передача (показ экрана при личной встрече)

**Не использовать:**
- Telegram (есть unencrypted history с ключевыми словами)
- WhatsApp (E2E но Meta-controlled, метаданные у Meta)
- Email plaintext
- SMS
- Slack/Discord/любой коммерческий мессенджер с server-side history

## Проверка работы

После настройки клиента и подключения:

```bash
# IP должен показать сервер VPN, не реальный IP клиента
curl https://api.ipify.org

# DNS должен идти через VPN (проверка leak)
curl https://1.1.1.1/cdn-cgi/trace
```

Если оба показывают сервер VPN — работает. Если `curl ipify` показывает VPN-IP, а DNS leak показывает домашнего провайдера — DNS leak, нужно править клиентский config.
