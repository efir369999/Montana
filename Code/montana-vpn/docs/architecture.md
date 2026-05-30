# Архитектура Montana VPN — как собрана Frankfurt

Этот документ описывает референсное развёртывание Montana VPN на Frankfurt VPS (`<exit-de>`, Ubuntu 24.04). Все компоненты, конфиги и обоснования живые — то есть так оно реально работает на момент `2026-05-02`.

## Слои стека

```
┌──────────────────────────────────────────────────┐
│ клиент (v2rayN / Hiddify / Streisand / sing-box) │
└───────────────────────┬──────────────────────────┘
                        │ TCP :443 (TLS Reality)
                        │ outer SNI = www.googletagmanager.com
                        │ inner = VLESS + xtls-rprx-vision
                        ▼
┌──────────────────────────────────────────────────┐
│ Frankfurt VPS (Ubuntu 24.04, x86_64)              │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │ ufw (default deny)                      │    │
│  │   22/tcp  — SSH                          │    │
│  │   80/tcp  — nginx decoy (камуфляж)      │    │
│  │   443/tcp — xray Reality VPN            │    │
│  └─────────────────────────────────────────┘    │
│                                                  │
│  :80 ──► nginx (sites-enabled/decoy)             │
│            └── /var/www/decoy/index.html         │
│                "It works! Default server page."  │
│                                                  │
│  :443 ──► xray 26.2.6 (xtls/xray-core)           │
│            └── inbound vless+reality+vision      │
│            └── outbound freedom (direct)         │
│                                                  │
│  fail2ban  — ssh brute-force protection          │
│  crowdsec  — IP-reputation feed + auto-ban       │
│  sysctl    — fq_codel + BBR (anti-bufferbloat)   │
│                                                  │
└──────────────────────────────────────────────────┘
                        │ outbound (direct)
                        ▼
                       Internet
```

## Почему именно VLESS + Reality + Vision

**VLESS** — минимальный V2Ray-протокол: только authentication по UUID, без обфускации поверх. Лёгкий, нет накладных расходов.

**Reality** — TLS-stealing handshake. Серверу не нужен собственный домен с валидным сертификатом. Вместо этого Reality на лету проксирует TLS-handshake к **чужому реальному сайту** (`www.googletagmanager.com:443`), и атакующий снаружи видит TLS-fingerprint Google. Только клиент с правильным `private_key`/`short_id` распознаёт что это «свой» сервер и переходит в туннельный режим. Атакующий без ключа — получает реальный ответ Google.

Это закрывает класс атак «active probing»: Russian/Iranian/Chinese DPI-боты **не отличают** Reality-эндпоинт от обычного proxy к Google.

**xtls-rprx-vision flow** — оптимизация для TCP-trafic. После handshake передаёт payload без дополнительного TLS-обёртывания, что устраняет известный fingerprint «TLS-in-TLS» и снижает CPU-нагрузку.

## Frankfurt: реальные параметры

| Параметр | Значение |
|---|---|
| OS | Ubuntu 24.04 LTS, kernel 6.8.0-79-generic |
| Arch | x86_64 |
| RAM | 961 MiB (узел Montana + xray поместятся) |
| Disk | 25 GiB, ~5 GiB used |
| xray | v26.2.6 (commit `12ee51e`, go1.25.7) |
| nginx | 1.24.0 (Ubuntu) |
| Reality dest SNI | `www.googletagmanager.com:443` |
| Listen | `0.0.0.0:443` (TCP) |
| Decoy | `0.0.0.0:80` nginx static |
| Outbound | `freedom` (direct, exit-IP = Frankfurt) |
| Routing | blackhole для bittorrent + private IPs |
| DNS | DoH `1.1.1.1` |
| systemd hardening | `User=nobody`, `NoNewPrivileges=true`, `CapabilityBoundingSet=CAP_NET_BIND_SERVICE,CAP_NET_ADMIN` |
| sysctl | `net.core.default_qdisc = fq_codel` (anti-bufferbloat) |
| ufw | default deny incoming, allow 22/80/443 |
| fail2ban | SSH bantime 1h→1w incremental, maxretry=3 |
| crowdsec | community IP-reputation, auto-ban 4h |

## Почему `www.googletagmanager.com` как dest SNI

Reality dest должен:

1. Разрешать TLS 1.3 + ECH (он разрешает оба)
2. Иметь `X25519` key exchange (Reality использует X25519)
3. Возвращать valid TLS handshake без strict-SNI блокировок
4. Быть глобально доступным (Google CDN)
5. Иметь высокий traffic baseline — чтобы появление новой связи не выделялось

`www.googletagmanager.com` подходит по всем пунктам. Альтернативы: `www.cloudflare.com`, `www.microsoft.com`, `www.icloud.com`. Не использовать сайты которые могут быть заблокированы в стране клиента (например Twitter).

## Почему nginx :80 с decoy

Активный пробер шлёт первым делом HTTP GET на :80. Если сервер отвечает 503 / connection refused / SSH banner — это **аномалия**, флаг для последующего более глубокого зондирования :443.

Decoy `index.html` со стандартным «It works!» от Apache/nginx — выглядит как **неактивированный VPS дефолтной конфигурации**. Не привлекает внимание ботов, ищущих «брошенные» VPS под web-defacement.

## Почему `freedom` outbound, а не каскад

В первоначальной концепции (моя память) Frankfurt был **front** для Frankfurt origin — `nginx stream-proxy :443 → <exit-de>:443`. На практике Frankfurt был перестроен в **самостоятельный exit-node** с прямым `freedom` outbound: проще, быстрее (один RTT вместо двух), exit-IP финский (хорошая юрисдикция).

Каскадная схема имеет смысл когда:
- Нужно скрыть **реальный** IP origin от клиента (zero-trust trust front)
- Front в дружественной юрисдикции, origin в любой
- Клиент компрометирован → видит только front-IP

Прямой exit-node имеет смысл когда:
- Юрисдикция уже хорошая (Финляндия)
- Caller хочет минимальную latency
- Нет требования двойной анонимизации

Frankfurt — второе.

## Почему статический Xray `User=nobody`, не root

Xray должен слушать `:443` (privileged port). По старой Unix-модели это требует root. Современная модель: запускать как `nobody` + `AmbientCapabilities=CAP_NET_BIND_SERVICE`. Принцип least-privilege: компрометация xray-процесса даёт `nobody`, а не root.

Drop-in `/etc/systemd/system/xray.service.d/10-donot_touch_single_conf.conf` — стандартный paranoid-блок официального xray installer. Перезаписывает `ExecStart` чтобы гарантировать что запускается **именно** `/usr/local/etc/xray/config.json`, а не глобальный `/etc/xray/...` или конфиг из current-dir.

## Почему `fq_codel` + BBR

VPN-трафик идёт через шифрованный TCP. Default Linux qdisc (`pfifo_fast`) при перегрузке буфера вызывает bufferbloat — RTT прыгает с 30ms до 500ms+. `fq_codel` (Fair Queue + Controlled Delay) держит buffer drain time на уровне ~5ms даже под нагрузкой.

`tcp_congestion_control=bbr` — Google's BBR. На lossy-каналах (Wi-Fi клиента, сотовая связь) BBR даёт 2-5× throughput vs CUBIC default.

Для VPN с ~10-20 одновременными клиентами это разница «работает идеально» vs «ютуб тормозит».

## Что НЕ делает этот стек

- **Не маскирует timing-correlation.** Если глобальный наблюдатель видит трафик клиента → Frankfurt + трафик Frankfurt → target — он может корреляцией восстановить «кто куда ходил».
- **Не защищает от компрометации клиентского устройства.** Если RAT на клиенте — VPN бесполезен.
- **Не log-free.** xray пишет access.log + error.log. Можно отключить (`"loglevel": "none"`) — оставлено для оператора.
- **Не двойной hop.** Clean exit-IP, не Tor.

## Связь с протоколом Montana

Узел Montana (singleton mode без сетевого слоя) **не использует** этот VPN. Узел тикает VDF локально и пишет state в `/var/lib/montana/`, портов наружу не открывает.

VPN и узел Montana — два **независимых** systemd-сервиса на одном хосте. Можно поднять оба (`scripts/install-vps-full.sh`), либо только один (`montana-vpn/install.sh` или `scripts/install-vps.sh`). Конфликтов по ресурсам нет — узел single-thread, xray async-IO.

После M6 (когда у узла появится сетевой слой) — узел получит свой порт (например `:9501`), и можно будет (опционально) пускать его за тот же Reality-эндпоинт через xray inbound на отдельном порту. Сейчас это не предусмотрено архитектурно.
