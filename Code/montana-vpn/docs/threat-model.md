# Threat model — Montana VPN (Reality endpoint)

Что закрывает Reality + decoy nginx, а что — нет. Рассматриваются классы атак, не конкретные техники, которые меняются по мере появления новых DPI-плагинов.

## Покрытые атаки

### 1. Пассивный DPI (signature-based)

**Атака.** Провайдер/госрегулятор смотрит TLS ClientHello каждого исходящего соединения. Знакомые VPN-протоколы (WireGuard handshake, OpenVPN, классический Shadowsocks с ASCII-маркерами) определяются по сигнатурам и блокируются.

**Закрытие.** Reality прокидывает реальный TLS handshake к `www.googletagmanager.com`. С точки зрения DPI — это легитимный TLS 1.3 к Google, fingerprint-неотличим от обычного HTTPS-браузера. Дополнительно `xtls-rprx-vision` устраняет «TLS-in-TLS» fingerprint после handshake.

### 2. Active probing — connection replay

**Атака.** Цензор замечает «подозрительный» :443-эндпоинт. Скриптом подключается на тот же IP с тем же ClientHello, что использовал реальный клиент, и смотрит ответ. Если сервер отдаёт паттерн VPN-протокола (короткий handshake, специфический response), цель идентифицирована.

**Закрытие.** Reality верифицирует `short_id` + handshake-secret клиента. Если они не совпадают (а у пробера их нет) — Reality **проксирует** запрос к настоящему `www.googletagmanager.com:443` и отдаёт пробру реальный Google response. Пробер видит только Google.

### 3. Active probing — port scan

**Атака.** Цензор сканирует все open-ports VPS. Хочет понять «зачем» :443 открыт.

**Закрытие.** :443 → реальный TLS handshake к Google (через Reality fallback). :80 → nginx static decoy «It works!». Всё выглядит как обычный VPS с дефолтной конфигурацией. Никаких VPN-маркеров.

### 4. SNI fingerprinting + ClientHello fingerprinting

**Атака.** DPI смотрит TLS ClientHello extensions, ECH, GREASE, supported groups, signature algorithms. Сравнивает с известными VPN-клиентами.

**Закрытие.** Xray uTLS-фингерпринт `chrome` — точная репликация Chrome 124 ClientHello. Отличить от настоящего Chrome — невозможно без поломки протокола.

### 5. SSH brute-force

**Атака.** Сканеры пробуют password/key brute-force через SSH :22.

**Закрытие.** fail2ban (maxretry=3, ban 1h→1w incremental). Crowdsec (community IP-reputation, auto-ban 4h). Дополнительно рекомендуется `PasswordAuthentication no` в sshd, ключ-only.

### 6. ASN-based blocking

**Атака.** Цензор блокирует целые AS-номера известных VPN-провайдеров (DigitalOcean, Vultr, OVH).

**Закрытие частичное.** Helsinki (THE.Hosting / IPRoute Latvia / etc) — менее известный провайдер чем DigitalOcean. Но если цензор расширяет список AS — рано или поздно VPS будет в списке. Полное закрытие — резидентный IP (домашний) или ASN ротация (выходит за scope этого пакета).

## НЕ покрытые атаки

### 1. Endpoint compromise

Если на клиентском устройстве установлен RAT/spyware — VPN не помогает. Атакующий читает данные **до** шифрования.

**Митигация:** secure boot, hardware-key auth, regular OS patches, минимизация attack surface.

### 2. Global passive adversary

Атакующий который видит **одновременно** трафик клиента (от него к Helsinki) и трафик Helsinki (к target site). Через timing-correlation и packet-size patterns может с высокой вероятностью восстановить связь «client → target», даже если содержимое зашифровано.

**Митигация:** Tor (multi-hop, traffic mixing). Для среднего пользователя global passive adversary — нерелевантная угроза (требует ресурсов NSA-class).

### 3. Server-side compromise

Если VPS-провайдер или хостер скомпрометирован, либо сам оператор VPN злоумышленник — он видит всё содержимое трафика выходящее из exit-node (после расшифровки).

**Митигация:** end-to-end шифрование на уровне приложения (HTTPS, Signal). VPN защищает **on-path** между клиентом и exit-node, не **at-exit**. Это не баг, это фундамент proxy-модели.

### 4. Корреляция через RTT/timing

Если цензор видит весь трафик между клиентом и провайдером, он может через timing latencies восстановить (примерно) «клиент сделал запрос, через 80ms Helsinki сделал такой же запрос наружу». Не криптографическое доказательство, но достаточно для targeted-investigation.

**Митигация:** padding traffic, randomized delays — не реализовано в xray по умолчанию. Доступно как отдельные плагины.

### 5. Long-term traffic-volume analysis

Постоянный исходящий трафик 50 GB/месяц от **одного** клиента **только** на один Helsinki IP — статистически выделяется vs обычного web-серфинга. Цензор может пометить такого пользователя как «VPN-suspect» даже без расшифровки.

**Митигация:** ротация exit-IP (несколько Helsinki/Frankfurt/etc), периодическая смена), смешивание с обычным трафиком.

## Лестница атакующего по сложности

| Уровень | Кто | Что может |
|---|---|---|
| 1 | Местный провайдер | Базовый DPI signature → закрыто Reality |
| 2 | Госрегулятор (РФ ТСПУ, Иран) | Active probing + ASN-banlist → закрыто Reality + decoy + (потенциально) ASN-rotation |
| 3 | Сильный регулятор (Китай GFW) | ML-based timing/volume detection → требует traffic padding (не покрыто) |
| 4 | NSA-class global adversary | Global passive observation + targeted-implant → требует Tor/multi-hop (не покрыто) |

Этот пакет закрывает уровни 1-2, частично 3. Уровни 3-4 требуют отдельных архитектурных решений.

## Failure modes — когда стек ломается

1. **Цензор блокирует dest SNI (`www.googletagmanager.com`).** Вероятность низкая — Google domain слишком важен. Митигация: смена `DECOY_HOST` на `www.cloudflare.com` или другой.
2. **Уязвимость в xray.** Уровень атак на Go-binary с pure-software TLS — низкий (xray активно поддерживается, security-issues закрываются ≤30 дней).
3. **Compromise xray credentials.** `state.env` mode 0600, читается только root. Если root scompromised → утрачен и VPN, и узел Montana, и весь VPS.
4. **Reality-protocol weakness обнаружен.** Reality введён 2022, активно изучается. Известные проблемы исправляются upstream. Старые xray-версии могут содержать уязвимости — требуется auto-update (`unattended-upgrades` для Ubuntu или периодический ручной `xray update`).

## Рекомендации оператору

- Использовать VPN **в дополнение** к threat model приложения, не как замену.
- Хранить мнемонику узла Montana и VPN-state.env в **разных** местах (компрометация одного не даёт второго).
- Регулярно обновлять xray (`bash <(curl -L https://github.com/XTLS/Xray-install/raw/main/install-release.sh) @ install`).
- Мониторить `journalctl -u xray --since today` на аномальные failures.
- Если VPS из «горячей» юрисдикции (US, RU, CN) — рассматривать как **компрометированный по умолчанию** и не доверять ему хранение секретов длительного срока.
