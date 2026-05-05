# Montana — установка на Mac (десктоп + ноут)

Узел Montana и desktop-приложение ставятся одной командой `install-anywhere.sh`. Поддерживается fresh-установка и recovery на втором Mac из 24-словной мнемоники либо 64-hex master_seed.

## Требования

- macOS 14.0+ (Sonoma) — Apple Silicon или Intel
- Xcode Command Line Tools (`xcode-select --install`)
- ~5 GB свободного места (Rust toolchain + dependencies + сборка)

Rust ставится автоматически через rustup-minimal, если не найден.

## Сценарий 1 — первый Mac (десктоп)

```
bash /path/to/Montana-Protocol/Apps/macOS/install-anywhere.sh
```

Что произойдёт:

1. Сборка `montana-node` release (~1-2 мин cold).
2. Установка binary в `~/Applications/Montana/montana-node`.
3. Генерация **identity** — 24 слова мнемоники выводятся в Terminal.
4. **Запишите 24 слова на бумагу**. Без них узел не восстанавливается.
5. Регистрация launchd-агента `org.montana.node` (auto-restart, переживает logout/reboot).
6. Сборка `Montana.app` (~10 сек) и установка в `/Applications/Montana.app`.
7. Запуск узла (фаза CandidateVdf → ~10 часов VDF → Registered).
8. Запуск Montana.app — на вкладке **Кошелёк** видны QR-код и `account_id`.

## Сценарий 2 — второй Mac (ноут) — recovery той же identity

На ноуте та же команда, но с переданной мнемоникой:

```
INSTALL_MNEMONIC_OR_SEED='word1 word2 ... word24' \
  bash /path/to/Montana-Protocol/Apps/macOS/install-anywhere.sh
```

Альтернативно — через 64-hex master_seed (показывается в приложении на десктопе через **Кошелёк → Backup → Показать master seed**):

```
INSTALL_MNEMONIC_OR_SEED='b0800520902f7439...64-hex' \
  bash /path/to/Montana-Protocol/Apps/macOS/install-anywhere.sh
```

Recovery flag `--force` затрёт identity.bin на ноуте если он был — это намеренно.

## Что работает в текущей сборке (ядро 0.1.0)

- ✅ Узел поднимается, проходит lifecycle Bootstrap → CandidateVdf → Registered → Active
- ✅ launchd-агент переживает logout, reboot, краш узла
- ✅ Montana.app — вкладки **Узел** (фаза, окно, VDF, лог) и **Кошелёк** (QR, идентификация, backup)
- ✅ Recovery identity на втором Mac из мнемоники или master_seed
- ✅ QR-код для получения Ɉ — отправители видят `account_id`

## Что НЕ работает в текущей сборке

- ❌ **Cross-machine синхронизация state.** Десктоп и ноут с одной identity работают независимо в singleton-режиме — балансы и chain не синхронизируются. P2P (M6+) — следующий milestone, требует 3-node genesis ceremony.
- ❌ **Отправка Ɉ.** CLI-команды `transfer` пока нет, в Montana.app кнопка Send показывает honest-сообщение о статусе. Получение через QR работает (отправители запоминают адрес).
- ❌ **Подпись транзакций app-уровня.** Anchor, Nickname, Premium, Auction — операции из spec, ещё не экспонированы в CLI.

## Полезные команды

```bash
# статус узла (фаза, окно, балансы)
~/Applications/Montana/montana-node status --data-dir ~/Applications/Montana/data

# показать identity (account_id, node_id, libp2p_peer_id)
~/Applications/Montana/montana-node inspect --data-dir ~/Applications/Montana/data

# показать master_seed для backup (НЕ ВЫВОДИТЬ при кому-то на экране!)
~/Applications/Montana/montana-node inspect --data-dir ~/Applications/Montana/data --reveal-master-seed

# логи узла realtime
tail -f ~/Applications/Montana/data/logs/montana.log

# остановить узел
launchctl unload ~/Library/LaunchAgents/org.montana.node.plist

# запустить узел
launchctl load -w ~/Library/LaunchAgents/org.montana.node.plist

# полная переустановка (сохраняет identity если есть)
bash /path/to/Montana-Protocol/Apps/macOS/install-anywhere.sh
```

## Безопасность мнемоники

- 24 слова — **единственный backup**. Identity.bin защищён правами 0600, но если потерян (отказ диска, переустановка ОС) — без мнемоники восстановление невозможно.
- Мастер-seed (64 hex) функционально эквивалентен 24 словам.
- **Никогда не передавайте** seed через мессенджеры, облачные заметки, фото экраном смартфона.
- **Записывайте на бумагу.** Две копии в физически разных местах.
- Кто получил seed — получил полный контроль над аккаунтом.

## Удаление

```bash
launchctl unload ~/Library/LaunchAgents/org.montana.node.plist
rm ~/Library/LaunchAgents/org.montana.node.plist
rm -rf ~/Applications/Montana
rm -rf /Applications/Montana.app
```

`identity.bin` удаляется вместе с `~/Applications/Montana/data/`. **Перед удалением убедитесь что мнемоника записана.**
