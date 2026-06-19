Montana — узел (genesis bootstrap, без сетевого слоя)
======================================================

Узел установлен через install-local-mac.sh и запущен через launchd:

  • работает в фоне 24/7
  • переживает закрытие Terminal, logout, перезагрузку Mac
  • автоматически рестартует при падении
  • логи пишутся в data/logs/montana.log + montana.err.log

Identity сгенерирован при установке. 24-словная мнемоника была
выведена в Terminal во время install — backup которой ваша
ответственность (мнемоника нигде на диске НЕ сохраняется).

ДВЕ КНОПКИ В FINDER
-------------------

  1. Запуск и логи узла   — launchctl load -w + статус + tail -F логов
                            (если узел уже запущен, просто показывает
                            статус и логи в реальном времени)
  2. Остановить узел      — launchctl unload -w

Это всё что нужно. Узел запущен через launchd и работает сам;
кнопка 1 нужна только чтобы посмотреть что он делает,
кнопка 2 — чтобы корректно остановить.

ЖИЗНЕННЫЙ ЦИКЛ УЗЛА
-------------------

Этот узел = genesis bootstrap node своей локальной сети. Per spec
Genesis Decree, bootstrap_node_pubkey активируется через genesis state
без Candidate SSHA — узел стартует сразу в Active phase.

  Phase Active: каждое окно (~60 сек, D = 252_000_000 SHA-256):
    1. TimeChain SSHA compute (T_r = SHA-256^D(prev))
    2. SshaReveal с endpoint = compute_endpoint(T_r, cba, node_id, W)
    3. BundledConfirmation с reveal_hash, подпись node_sk
    4. is_cemented quorum check (genesis узел сам себя cement: 1/1)
    5. weighted_ticket_node lottery, determine_winner argmin
    6. ProposalHeader с правильным state_root, подпись node_sk
    7. mt_account::apply_proposal canonical pipeline:
         — Step 2 apply_emission (13 Ɉ оператору)
         — Step 3.5 chain_length++ для cemented confirmers
         — Step 3.6 checkpoint rotation
         — Step 4 state_root recompute
    8. state_root self-verify через compute_state_root recompute
    9. archive_proposal через FsStore::archive_proposal

  Каждое τ₂ = 20160 окон (≈14 дней): mt_timechain::next_d adaptive D.

Эмиссия: 13 Ɉ оператору per окно (winner_W-1 — за окно W). На W=1
эмиссии нет (apply_emission early-return на genesis окно). С W=2:
balance растёт линейно ~13 Ɉ × N окон.

ОСТАЛЬНЫЕ УЗЛЫ (когда добавится сетевой слой M6+)
--------------------------------------------------

Узлы которые присоединяются к существующей сети проходят полный
canonical путь:

  1. fast-sync TimeChain до текущего W (от genesis узла, verify-only)
  2. Candidate SSHA (~10 часов на M-class Mac, τ₂ × D итераций SHA-256)
  3. NodeRegistration через apply_noderegistrations_batch
  4. selection event на следующем W % 336 == 0 → активация
  5. Active phase

В текущей версии montana-node (без M6) этот путь существует в коде
для будущего использования, но не выполняется (один genesis узел).

ВОССТАНОВЛЕНИЕ ИЗ МНЕМОНИКИ
---------------------------

24 слова — единственный надёжный backup. Мнемоника в файлах НЕ
сохраняется (стандартная практика безопасности крипто-кошельков).

Полная переустановка с нуля (INSTALL_DIR — путь установки, по умолчанию
~/Applications/Montana, но может быть любой через env var INSTALL_DIR):

  launchctl unload -w ~/Library/LaunchAgents/org.montana.node.plist
  rm -rf "$INSTALL_DIR/data"
  bash <путь>/scripts/install-local-mac.sh

Installer создаст новую identity. Запишите 24 слова до нажатия Enter.

LAUNCHD CONTROL (продвинутое)
-----------------------------

LaunchAgent: ~/Library/LaunchAgents/org.montana.node.plist

Прямые команды:
  launchctl list org.montana.node
  launchctl unload ~/Library/LaunchAgents/org.montana.node.plist
  launchctl load -w ~/Library/LaunchAgents/org.montana.node.plist

Полное удаление ($INSTALL_DIR — текущий путь установки):
  launchctl unload -w ~/Library/LaunchAgents/org.montana.node.plist
  rm ~/Library/LaunchAgents/org.montana.node.plist
  rm -rf "$INSTALL_DIR"

ЕСЛИ macOS БЛОКИРУЕТ ЗАПУСК .command
------------------------------------

  Правый клик (Control + клик) на .command → «Открыть» → «Открыть»

Либо через терминал ($INSTALL_DIR — путь установки):
  xattr -dr com.apple.quarantine "$INSTALL_DIR"

ЧТО ВНУТРИ
----------

montana-node                    — бинарь узла
1. Запуск и логи узла.command    — launchctl load + status + tail -F
2. Остановить узел.command       — launchctl unload -w
data/                            — состояние узла
  identity.bin                   — ключи (mode 0600)
  accounts.bin / nodes.bin /     — таблицы state
  candidates.bin
  logs/montana.log               — stdout (per-window события)
  logs/montana.err.log           — stderr
  meta/current_window.bin        — текущее окно (u64)
  meta/timechain.bin            — T_r + D + last_window
  meta/node_state.bin            — phase lifecycle
  proposals/                     — archived ProposalHeaders 3722B каждый

КРИПТОГРАФИЯ
------------

ML-DSA-65   (FIPS 204)  — постквантовая подпись
ML-KEM-768  (FIPS 203)  — постквантовый обмен ключами
SHA-256     (FIPS 180-4) — хеширование, SSHA
PBKDF2 + HKDF (RFC 5869) — деривация ключей

Все примитивы через OpenSSL 3.5 LTS (статически линкованный),
51 NIST KAT vectors verified byte-exact.
