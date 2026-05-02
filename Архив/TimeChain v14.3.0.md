# TimeChain — Спецификация протокола Montana

**Версия:** 14.3.0 (2026-04-01)

## Определение

Децентрализованная сеть. Время — единственная реальная валюта. 1 секунда присутствия узла в сети = 1 Ɉ.

Консенсус: **Proof of Time (PoT)** — последовательное SHA-256 хэширование (VDF) как доказательство прошедшего времени. Временные окна возникают из вычислений.

Генезис: 09.01.2026 00:00:00 MSK.

---

## Криптография

Два примитива с разделёнными ролями:

- **SHA-256** — консенсус (Beacon VDF, Service VDF), адреса, Merkle-деревья, хэширование
- **FN-DSA-512** (selected NIST candidate, forthcoming FIPS 206) — подписи транзакционных блоков

SHA-256 обеспечивает квантовую устойчивость консенсуса: алгоритм Гровера сокращает безопасность с 256 до 128 бит. FN-DSA-512 обеспечивает математическую постквантовую устойчивость подписей на основе NTRU-решёток.

### Подписи — FN-DSA-512

Подпись на NTRU-решётках (Falcon-512). Stateless, многоразовая. Публичный ключ закрепляется за аккаунтом при создании и используется для всех последующих блоков.

| Компонент | Размер |
|-----------|--------|
| Приватный ключ | 1 281B |
| Публичный ключ | 897B |
| Подпись (padded) | 666B |

Поле suite_id в формате блока обеспечивает миграцию подписи без изменения модели состояния. Активация новой схемы требует protocol upgrade. Активная схема на момент запуска: FN-DSA-512.

### Адреса

Формат: `mt` + Base58(account_id + checksum).

Account_id = SHA-256("mt-account" || suite_id || pubkey). Стабильный идентификатор аккаунта. Смена ключа или схемы подписи выполняется через ChangeKey блок без изменения account_id — для этого account_id привязан к первому pubkey, а текущий ключ хранится в состоянии аккаунта.

---

## Account Chain (Block Lattice)

Каждый аккаунт имеет собственную цепочку блоков. Перевод — один блок в цепочке отправителя. Зачисление получателю — детерминированно после финализации proposal. Цепочки аккаунтов полностью независимы.

### Типы блоков

**OpenAccount** — создание аккаунта (один раз):

```
type           1B
suite_id       2B
account_id    32B
pubkey       897B     <- FN-DSA-512, публикуется единожды
pending_root  32B     <- Merkle root всех захваченных pending entries
balance        8B     <- детерминированно = сумма захваченных entries (>= ACCOUNT_RESERVE)
signature    666B
Итого:     ~1 638B
```

**StateBlock** — перевод:

```
prev_hash     32B     <- хэш предыдущего блока в цепочке аккаунта
account_id    32B
link          32B     <- account_id получателя
link_amount    8B     <- сумма перевода получателю
balance        8B     <- абсолютный баланс отправителя после операции
flags          1B
signature    666B
Итого:       ~779B
```

**ChangeKey** — смена ключа или схемы подписи:

```
prev_hash     32B
account_id    32B
new_suite_id   2B
new_pubkey   897B     <- новый публичный ключ
signature    666B     <- подписано старым ключом
Итого:     ~1 629B
```

### Верификация баланса

Баланс в StateBlock абсолютный. StateBlock содержит поле balance (новый баланс отправителя после операции). Перевод содержит сумму в поле link_amount (8B, добавляется в формат StateBlock).

```
fee = prev_balance - new_balance - link_amount
```

Каждый узел проверяет: new_balance >= 0, link_amount > 0, fee >= min_fee.

### Комиссия

Комиссия вычисляется из трёх известных величин: prev_balance (из Account Table), new_balance и link_amount (из StateBlock). Минимум 1 mɈ. Размер min_fee адаптивный — рассчитывается по формуле на основе заполненности окон τ₁. Пользователь знает комиссию до отправки.

Победитель окна τ₂ получает сумму всех комиссий блоков в окне.

### Pending Transfers и OpenAccount

Отправка на несуществующий account_id создаёт pending entry в Pending Transfer Table:

```
Pending entry:
  source_block_hash   32B
  from_account_id     32B    <- отправитель (для refund при expiry)
  to_account_id       32B
  amount               8B
  expiry_τ₂            4B    <- дедлайн: N окон τ₂ после финализации
```

Баланс отправителя уменьшается сразу при финализации StateBlock. Средства хранятся в pending entry до открытия аккаунта получателем.

OpenAccount захватывает все pending entries адресованные данному account_id. Canonical rule: множество захваченных entries = все записи в Pending Transfer Table где `to_account_id == account_id`. Выбора нет — захватываются все или ни одна. pending_root в OpenAccount = Merkle root этого множества, отсортированного по (source_block_hash). balance = сумма amount всех захваченных entries.

Верификация OpenAccount в state transition:

1. Вычислить множество entries из Pending Transfer Table по to_account_id
2. Вычислить Merkle root множества -> сравнить с pending_root в блоке
3. Вычислить сумму amount -> сравнить с balance в блоке
4. Проверить balance >= ACCOUNT_RESERVE
5. Удалить все matching entries из Pending Transfer Table
6. Создать запись в Account Table

Автоматический возврат: если pending entry не захвачена OpenAccount до expiry_τ₂, средства возвращаются отправителю протокольно в state transition. Новая подпись отправителя не требуется.

### Account Reserve

ACCOUNT_RESERVE Ɉ — минимальная сумма для открытия аккаунта (параметр протокола). Защита от спама аккаунтов.

### Coinbase

Победитель τ₂ создаёт coinbase-блок в своей цепочке. Баланс увеличивается на сумму эмиссии + комиссии всех блоков окна.

Supply audit при финализации τ₂: суммарная эмиссия coinbase от генезиса сверяется с supply(height) из issuance schedule.

### Двойная трата

Каждый аккаунт имеет одну цепочку. Два блока с одним prev_hash = форк. Форк обнаруживается мгновенно. Разрешается победителем τ₂: canonical rule lower H(block) wins. Второй блок отбрасывается.

---

## Состояние сети

Глобальное состояние = Account Table + Node Table + Pending Transfer Table + Expiry Queue.

```
Account Table (запись на аккаунт):
  account_id       32B
  balance           8B
  frontier_hash    32B    <- хэш последнего блока в цепочке
  block_height      4B
  suite_id          2B
  current_pubkey  897B

Node Table (запись на узел):
  node_id                 32B     <- SHA-256("mt-node" || node_pubkey), верифицируемо
  node_pubkey            897B
  suite_id                 2B
  chain_length             4B     <- суммарное количество подписанных τ₂, weight = chain_length
  pending_invite          32B     <- node_id приглашённого (0x00..00 если нет)
  invite_window            4B     <- окно финализации Invitation (0 если нет)
  invite_expires           4B     <- invite_window + 2116 (0 если нет)
  status                   1B     <- active | suspended

Pending Transfer Table (запись на pending перевод):
  source_block_hash  32B
  from_account_id    32B    <- для refund при expiry
  to_account_id      32B
  amount              8B
  expiry_τ₂           4B

Expiry Queue (индекс pending entries по сроку):
  expiry_τ₂           4B
  entry_hash         32B
```

### State Root

Merkle-дерево глобального состояния. Четыре подкорня, каждый — Merkle root своей таблицы с явным порядком листьев:

```
State Root = SHA-256("mt-merkle-node" || account_root || node_root || pending_root || expiry_root)

Account Table Root:          листья по account_id (лексикографически)
Node Table Root:             листья по node_id (лексикографически)
Pending Transfer Root:       листья по (to_account_id || from_account_id || source_block_hash) (лексикографически)
Expiry Queue Root:           листья по (expiry_τ₂ big-endian 4B || entry_hash) (лексикографически)

active_set_root = Merkle root подмножества Node Table где status = active
                  и chain_length > 0.
                  Детерминировано из Global State. Узлы с weight = 0 не входят в active set.
```

Все sort keys фиксированной длины. Побайтовое лексикографическое сравнение. Две реализации с одинаковыми данными строят одинаковое дерево и получают одинаковый State Root.

State Root коммитится в заголовке каждого финализированного proposal τ₂.

---

## Таймчейн

Три логических двигателя с односторонним потоком зависимостей: Beacon -> Service -> Execution.

### Beacon — глобальные часы протокола

Непрерывная последовательная SHA-256 цепочка. Общий источник случайности и ритм сети:

```
B_r = SHA-256^D(B_{r-1})
```

D — количество последовательных хэшей за один слот τ₁. Beacon продвигается по расписанию финализированных слотов. Для фиксированного индекса r значение B_r совпадает у всех честных узлов. Каждый узел вычисляет Beacon независимо — результат детерминирован.

Beacon не зависит от состояния, транзакций и поведения отдельных узлов.

### Service — персональный VDF узла

Доказательство непрерывного присутствия конкретного node_id. Якорится в Beacon каждый слот:

```
S_{i,s,0}   = SHA-256(S_{i,s-1,m} || B_s || node_id_i)
S_{i,s,j+1} = SHA-256(S_{i,s,j})    для j = 0..m-1
```

Три компонента seed: предыдущий endpoint (непрерывность службы), значение Beacon (протокольное время), node_id (идентичность). m последовательных хэшей за слот — доказательство работы.

Инициализация: для первого слота нового узла предыдущий endpoint отсутствует. Service init привязан к каноническим данным proposal в котором Invitation финализирован:

```
S_{i,0,0} = SHA-256("mt-service-init" || control_root || beacon_value || node_id_i)
```

control_root и beacon_value из proposal header окна финализации Invitation. Оба канонические (не зависят от субъективного user_set). Предвычисление VDF невозможно — beacon_value неизвестен до закрытия окна. Grinding surface = ноль. Верифицируем любым узлом.

Service зависит от Beacon. Beacon не зависит от Service.

### VDF Reveal

После закрытия окна τ₂ каждый узел публикует подписанное reveal-сообщение (VDF endpoint становится известен только после завершения VDF за окно):

```
VDF_Reveal:
  node_id          32B
  window_index      4B     <- индекс τ₂
  endpoint         32B     <- S_{i,final,m}
  signature       666B     <- FN-DSA-512, подписано node_pubkey
Итого:       ~734B
```

Reveal распространяется по P2P. Валидация при получении:

1. Подпись FN-DSA-512 соответствует node_pubkey из Node Table
2. window_index = только что закрытый τ₂
3. node_id существует в Node Table, status = active
4. endpoint верифицируем: пересчёт Service VDF от предыдущего endpoint (или от service_init для первого окна узла)

Два reveal с одинаковыми (node_id, window_index) и одинаковым endpoint = дедупликация. Два reveal с одинаковыми (node_id, window_index) и разным endpoint = reveal_conflict (equivocation).

active_vdf_list = все валидные VDF_Reveal полученные до reveal_cutoff. Порядок: node_id лексикографически. Список каноничен. Свобода победителя над списком: ноль. Пропуск валидного reveal = невалидный proposal = fallback.

### Execution — содержимое блока

Приём, верификация execution objects и формирование набора. Два класса объектов:

**UserObjects** — пользовательские операции:

| Тип | Описание | Валидация |
|-----|----------|-----------|
| StateBlock | Перевод средств | FN-DSA-512 подпись, баланс >= 0, prev_hash, link_amount > 0, fee >= min_fee |
| OpenAccount | Создание аккаунта | FN-DSA-512 подпись, pending_root, balance >= ACCOUNT_RESERVE |
| ChangeKey | Смена ключа | FN-DSA-512 подпись старым ключом, new_pubkey |

**ControlObjects** — объекты управляющие составом сети:

| Тип | Описание | Валидация |
|-----|----------|-----------|
| Invitation | Приглашение нового узла | FN-DSA-512 подпись пригласившего, pending_invite = 0, status = active |
| NodeRegistration | Регистрация узла | FN-DSA-512 подпись, node_id уникален, proof_endpoint верифицируем через VDF, приглашение существует |
| EquivocationProof | Доказательство equivocation | Два конфликтующих подписанных сообщения от одного node_id (proposal_conflict или reveal_conflict) |

Каждый узел валидирует объекты обоих классов локально при получении. Валидные объекты ретранслируются по P2P. ControlObjects дополнительно ретранслируются каждый τ₁ до финализации или expiry.

#### Два набора в proposal

Proposal содержит два набора с разными правилами:

**user_set** = все валидные UserObjects из мемпула победителя до block_cutoff. Определяется мемпулом победителя. UserObjects не вошедшие в proposal переносятся в следующее окно. Пропуск UserObject не является основанием для отклонения proposal.

**control_set** = все валидные ControlObjects полученные по P2P до control_cutoff, не финализированные ранее и не истёкшие. Каноничен — все ControlObjects включены ровно один раз. Пропуск или добавление лишнего ControlObject = невалидный proposal = fallback.

Порядок внутри обоих наборов: H(object) лексикографически.

Форки аккаунтов (два блока с одним prev_hash) разрешаются правилом lower H(block) wins.

#### Три дедлайна в окне

```
|--- окно τ₂ (~600 сек) ---|-- reveal phase (R сек) --|
                            ^                          ^
                      block_cutoff         reveal_cutoff = control_cutoff
                    (C сек до закрытия)      (R сек после закрытия)
```

- **block_cutoff** = C секунд до закрытия окна. UserObjects после block_cutoff переносятся в следующее окно. C достаточен для полной P2P-пропагации (~30 секунд).
- **control_cutoff** = reveal_cutoff = R секунд после закрытия окна. ControlObjects принимаются до этого момента. Дополнительное время + ретрансляция каждый τ₁ обеспечивают полную пропагацию.
- **reveal_cutoff** = R секунд после закрытия окна. VDF_Reveal публикуются после закрытия окна.

После reveal_cutoff: определяется победитель лотереи, победитель собирает proposal.

#### Proposer

Победитель собирает proposal из двух наборов:
- **control_set**: все ControlObjects до control_cutoff (каноничен, свобода = ноль)
- **user_set**: все UserObjects из своего мемпула до block_cutoff

Свобода proposer над обработкой: ноль (порядок = H(object), state transition = детерминирован). Свобода над control_set: ноль (каноничен). Свобода над user_set: ограничена мемпулом (задержка, не раскол).

Proposal с невалидным объектом, пропущенным ControlObject или неверным state_root отклоняется, переход ко второму месту.

#### Финальность proposal

Финальность = подпись победителя на proposal header + независимая верифицируемость.

1. Победитель публикует подписанный proposal header + control_set + user_set + active_vdf_list
2. Каждый узел проверяет каждый блок из списка по правилам валидации
3. Каждый узел применяет state transition детерминированно
4. Каждый узел сравнивает вычисленный state_root с заявленным в proposal
5. Совпадает — proposal принят, state обновлён
6. Не совпадает — proposal отклонён, fallback на второе место

Proposal header:

```
Proposal header:
  prev_proposal_hash    32B
  prev_state_root       32B    <- State Root до применения блоков
  control_root          32B    <- Merkle root control_set (каноничен)
  user_root             32B    <- Merkle root user_set
  vdf_list_root         32B    <- Merkle root active_vdf_list (каноничен)
  new_state_root        32B    <- State Root после применения всех блоков
  active_set_root       32B
  beacon_value          32B
  coinbase_hash         32B
  winner_node_id        32B
  signature            666B    <- FN-DSA-512, подписано node_pubkey победителя
```

Fallback: если proposal победителя не получен в пределах timeout (120 секунд после reveal_cutoff) или отклонён — proposal формирует второе место (следующий min ticket). Каскад до третьего места и далее с тем же timeout. Молчание не наказывается — победитель теряет только coinbase за это окно.

#### Async confirmations (для light clients)

После принятия proposal узлы публикуют confirmation:

```
Confirmation:
  node_id          32B
  proposal_hash    32B
  signature       666B
```

Confirmations не участвуют в консенсусе. Они предоставляют light clients аудиторский след: независимые узлы подтвердили корректность state_root. Light client взвешивает confirmations по chain_length из prev_state_root (состояние на начало окна, зафиксировано в proposal header). Не по количеству node_id. Порог доверия определяется light client.

#### State transition

При финализации proposal state transition применяется атомарно в фиксированном порядке:

```
apply_proposal(state, proposal) -> state':

  Шаг 1: применить control_set в порядке H(object) лексикографически.
    Invitation:       записать pending_invite, invite_window и invite_expires в Node Table пригласившего.
    NodeRegistration: проверить приглашение, создать запись в Node Table
                      (chain_length=1, status=active). Очистить pending_invite пригласившего.
    EquivocationProof: применить санкцию к node_id в Node Table
                       (proposal_conflict или reveal_conflict: status=suspended, chain_length=0).

  Шаг 2: применить user_set в порядке H(object) лексикографически.
    StateBlock:   обновить баланс отправителя.
                  Получатель существует -> кредитовать баланс.
                  Получатель не существует -> создать pending entry.
    OpenAccount:  захватить все pending entries для account_id
                  (включая созданные ранее на этом же шаге),
                  создать запись в Account Table.
    ChangeKey:    обновить pubkey и suite_id в Account Table.

  Шаг 3: применить coinbase победителя.

  Шаг 4: обновить chain_length.
    Для каждого node_id из active_vdf_list в proposal:
    chain_length += 1 в Node Table.

  Шаг 5: обработать expiry.
    Pending transfers: все entries где expiry <= current_window
    и не захваченные OpenAccount на шаге 2 ->
    вернуть amount на баланс from_account_id,
    удалить entry из Pending Transfer Table и Expiry Queue.
    Приглашения: все записи Node Table где invite_expires <= current_window
    и invite_expires > 0 -> очистить pending_invite, invite_window и invite_expires.

  Шаг 6: вычислить new_state_root.
```

Порядок детерминирован. Control_set первым, user_set вторым, coinbase третьим, chain_length четвёртым, expiry последним. Каждый узел применяет одну и ту же последовательность шагов к одним и тем же наборам и получает один и тот же new_state_root.

Execution зависит от Beacon и Service. Обратных зависимостей нет.

С ростом TPS сети дополнительные ядра подключаются для верификации блоков. Минимум для валидатора: 3 логических ядра (Beacon + Service + Execution). Верификация блоков аккаунтов полностью параллелизуется — цепочки аккаунтов независимы.

### Приглашение и регистрация узла

Вход в сеть — по приглашению. Каждый зарегистрированный узел может пригласить одного нового. Пока приглашённый в процессе регистрации, пригласивший не может приглашать других. Одновременно одно приглашение на узел.

Генезис: 12 узлов в разных локациях (hardcoded, аналог bootstrap nodes в Bitcoin).

#### Invitation

Зарегистрированный узел публикует приглашение:

```
Invitation:
  inviter_node_id    32B
  invited_pubkey    897B     <- публичный ключ нового узла
  signature         666B     <- подписано inviter node_pubkey
Итого:          ~1 595B
```

Invitation не содержит start_window — он определяется при финализации.

Валидация Invitation:

1. Подпись валидна для inviter node_pubkey из Node Table
2. inviter status = active
3. inviter pending_invite = 0x00..00 (нет активного приглашения)
4. invited node_id = SHA-256("mt-node" || invited_pubkey) не существует в Node Table

При финализации в proposal P окна W:
- inviter pending_invite = invited node_id
- inviter invite_window = W
- inviter invite_expires = W + 2116 (2016 VDF + 100 окон запас)

#### Привязка VDF к моменту приглашения

VDF seed приглашённого узла привязан к хэшу proposal в котором Invitation финализирован:

```
service_init = SHA-256("mt-service-init" || control_root || beacon_value || node_id)
```

control_root и beacon_value — канонические поля из proposal header окна финализации. Не зависят от субъективного user_set победителя. Предвычисление VDF невозможно: beacon_value неизвестен до закрытия окна.

Приглашённый узел узнаёт control_root и beacon_value только увидев финализированный proposal → вычисляет service_init → начинает VDF с окна W+1.

#### Регистрация узла

Приглашённый узел после финализации Invitation:

1. Наблюдает proposal с Invitation → получает control_root и beacon_value
2. Вычисляет service_init = SHA-256("mt-service-init" || control_root || beacon_value || node_id)
3. Непрерывно вычисляет Service VDF: 2016 окон подряд (от W+1 до W+2016), каждое якорится в соответствующий Beacon
4. Через ~14 дней получает proof_endpoint = S_{i,2015,m}
5. Публикует NodeRegistration

```
NodeRegistration:
  type              1B
  suite_id          2B
  node_pubkey     897B     <- FN-DSA-512 ключ узла
  inviter_node_id  32B     <- кто пригласил
  proof_endpoint   32B     <- S_{i,2015,m} (endpoint после 2016 окон VDF)
  signature       666B     <- подписано node_pubkey
Итого:        ~1 630B
```

Валидация NodeRegistration:

1. Подпись FN-DSA-512 валидна для node_pubkey
2. node_id уникален (не существует в Node Table)
3. inviter_node_id существует в Node Table, pending_invite = node_id
4. invite_window + 2016 < текущее окно (VDF завершён)
5. Восстановить control_root и beacon_value из proposal окна invite_window
6. Вычислить service_init = SHA-256("mt-service-init" || control_root || beacon_value || node_id) из proposal окна invite_window
7. proof_endpoint верифицируем: пересчёт VDF от service_init через 2016 окон с якорением в Beacon значения от invite_window+1

Верификация: 2016 сегментов VDF проверяются параллельно. На C ядрах: ~(2016/C) × t_segment.

При финализации: создать запись в Node Table (chain_length = 1, status = active). Очистить pending_invite, invite_window и invite_expires у пригласившего.

#### Истечение приглашения

Если NodeRegistration не финализирован до invite_expires (invite_window + 2116) — приглашённый не завершил VDF. При обработке state transition: pending_invite, invite_window, invite_expires пригласившего очищаются автоматически. Узел может приглашать снова.

#### Скорость роста сети

Каждый узел производит максимум одно приглашение за ~14 дней. Рост ограничен текущим размером сети:

```
Генезис:      12 узлов
14 дней:      24
28 дней:      48
1 год:        до 12 × 2^26 (~800M, теоретический максимум)
```

Аппаратный бюджет сверх количества приглашений бесполезен. 1000 узлов = максимум 1000 новых за 14 дней, независимо от количества ядер.

---

## Потоковая модель

Блоки аккаунтов текут непрерывно. Узел получает блок -> проверяет подпись FN-DSA-512 и баланс -> передаёт в P2P gossip. Время подтверждения определяется скоростью P2P-распространения (~2-5 секунд).

Блок включается в канонический набор τ₂ при включении победителем лотереи в proposal. Блоки не ожидают появления τ₁ или τ₂ — окна являются точками финализации, а не условиями приёма.

Цепочки аккаунтов полностью независимы. Блоки разных аккаунтов обрабатываются параллельно без конфликтов.

---

## Временные слои (τ)

```
τ₁ (≈60с) → τ₂ (10 × τ₁) → τ₃ (2016 × τ₂)
```

### τ₁ — Слот (≈60 секунд)

- Beacon продвигается на D хэшей
- Service VDF продвигается на m хэшей с якорем в текущем B_s
- Узлы валидируют и ретранслируют блоки аккаунтов по P2P

### τ₂ — Финализация и эмиссия (10 × τ₁ ≈ 10 минут)

- Active set (состав и веса участников) фиксируется в начале τ₂
- control_set: все валидные ControlObjects до control_cutoff (каноничен)
- user_set: все валидные UserObjects из мемпула победителя до block_cutoff
- Узлы раскрывают Service VDF endpoint
- Лотерея: `ticket_i = -ln(S_{i,final,m} / 2^256) / weight_i`, победитель = min(ticket_i) среди допущенных
- Победитель публикует подписанный proposal:

```
Proposal header:
  prev_proposal_hash    32B
  prev_state_root       32B
  control_root          32B
  user_root             32B
  vdf_list_root         32B
  new_state_root        32B
  active_set_root       32B
  beacon_value          32B
  coinbase_hash         32B
  winner_node_id        32B
  signature            666B
```

- Финальность: подпись победителя на proposal header. Каждый валидатор применяет блоки детерминированно и проверяет new_state_root
- При финализации: state transition по фиксированному порядку (см. раздел «State transition» в Execution)
- Coinbase: вся эмиссия + все комиссии → победителю
- Supply audit: суммарная эмиссия coinbase от генезиса сверяется с supply(height) из issuance schedule
- Разрешение форков: приоритет ветки с наибольшим суммарным Beacon-доказательством

Beacon safety: компрометация значения Beacon требует нарушения свойства последовательности SHA-256 VDF.

Beacon liveness: задержка продвижения Beacon невозможна — Beacon вычисляется каждым узлом независимо.

### τ₃ — Адаптация (2016 × τ₂ ≈ 14 дней)

- Калибровка D и m: медиана слотовых интервалов сверяется с целевыми, цель τ₁ ≈ 60 секунд
- State Root коммитится в каждом τ₂ (в proposal header). State Root покрывает весь Global State: Account Table, Node Table, Pending Transfer Table, Expiry Queue. τ₃ фиксирует канонический State Root для Fast Sync
- Криптографическая амнезия: подписанные proposals сохраняются навсегда — верифицируемая цепочка state commitments. Тела блоков аккаунтов, подписи FN-DSA-512 и данные execution objects удаляются после 8 × τ₃ (112 дней). Proposals доказывают что конкретное состояние было закоммичено победителем; восстановление содержимого состояния требует snapshot или архива. Equivocation proofs по объектам старше 8 × τ₃ не принимаются — все споры разрешаются внутри окна
- Пересчёт параметров окна τ₁

---

## Консенсус — Proof of Time (PoT)

### Три двигателя

**Beacon** — глобальные часы. Чистая VDF-цепочка `B_r = SHA-256^D(B_{r-1})`. Источник случайности для лотереи. Продвигается по расписанию финализированных слотов.

**Service** — персональный VDF. Якорится в Beacon каждый слот. Доказывает присутствие node_id. Определяет лотерейный билет. Раскрытие endpoint при закрытии τ₂ = подпись окна = +1 к весу.

**Execution** — содержимое блока. Два набора: control_set (каноничен, все ControlObjects) + user_set (из мемпула победителя). Порядок и обработка детерминированы.

Зависимости односторонние: Beacon -> Service -> Execution. Отказ в Execution не заражает случайность. Отказ конкретного узла в Service не заражает общий ритм.

### Стаж и вес

#### Определение

Вес узла — суммарное количество подписанных τ₂:

```
weight_i = chain_length_i
```

Вес — единственная мера влияния узла в протоколе. Определяется только количеством подписанных окон.

#### Как растёт

Подписал окно — плюс один. Не подписал — ничего не произошло. Equivocation — chain_length = 0.

Узел раскрывает VDF_Reveal после закрытия τ₂ (см. раздел «VDF Reveal»). active_vdf_list каноничен: все валидные reveal до reveal_cutoff, порядок по node_id. Свобода победителя: ноль. State transition: chain_length += 1 для каждого node_id из списка. Пропуск окна не наказывается — узел просто не получает +1.


#### На что влияет вес

Вес определяет две вещи:

**1. Лотерея.** Вероятность победы в τ₂ строго пропорциональна весу:

```
ticket_i = -ln(S_{i,final,m} / 2^256) / weight_i
winner = min(ticket_i)
P(node_i) = weight_i / Σ weight(all_nodes)
```

Доказательство: S_{i,final,m} / 2^256 приближает U[0,1]. -ln(U) ~ Exp(1). -ln(U)/w ~ Exp(w). Минимум независимых Exp(w_i): P(i wins) = w_i / Σw_j. Точно пропорционально при любых весах.

Узел с weight = 0 не участвует в лотерее.

**2. Допуск.** weight = 0 означает: узел участвует в сети (валидация, ретрансляция), но не участвует в лотерее и не может быть победителем τ₂.

### Победитель τ₂

Победитель определяется после закрытия окна τ₂. Каждый узел раскрывает свой Service VDF endpoint. Минимальный ticket среди допущенных — победитель.

Победитель публикует подписанный proposal: control_set (каноничен) + user_set (из мемпула) + coinbase + active_vdf_list (каноничен). Порядок и обработка детерминированы. Валидация: control_set полон, все объекты валидны, state_root корректен. UserObjects не вошедшие в proposal переносятся в следующее окно.

Финальность — подпись победителя на proposal header. Верификация — независимый пересчёт state_root.

### Верификация

Победитель публикует: `{node_id, Service VDF endpoints[], proposal}`.

Верификация Service VDF: пересчёт K сегментов параллельно. K ≈ 960. На C ядрах: ~(K/C) × t_segment секунд. 8 ядер → ~7.5 сек. 64 ядра → ~1 сек.

Верификация proposal: независимое применение блоков из canonical set и сравнение state_root.

### Устойчивость

- **Proposer grinding** исключён: порядок = H(object) лексикографически, state transition детерминирован, свобода над обработкой = ноль
- **Committee grinding** исключён: Beacon не зависит от состояния и транзакций, seed лотереи строится из Beacon
- **Node_id гриндинг** исключён: Beacon неизвестен при регистрации
- **Предвычисление** исключено: seed содержит текущее значение Beacon
- **Replay** исключён: Beacon уникален для каждого τ₂
- **Аппаратное преимущество** ограничено: последовательное хэширование масштабируется тактовой частотой и IPC, а не количеством ядер или бюджетом
- **Sybil-барьер**: вход по приглашению (1 инвайт на узел, 1 одновременно) + регистрация = 2016 окон VDF (~14 дней) + Service VDF (физическое ядро) + линейный рост веса. Скорость роста Sybil ограничена размером его текущей сети, а не бюджетом
- **Цензура UserObjects** = задержка, не раскол. Пропущенный блок переносится в следующее окно. В account chain prev_hash резервирует место
- **Цензура ControlObjects** исключена: control_set каноничен, пропуск = невалидный proposal = fallback
- **Liveness halt** исключён: нет порогового голосования, финальность определяется одним победителем с fallback на следующий ticket
- **Fallback cascade**: молчащий победитель теряет coinbase окна. Санкции без подписанного доказательства не применяются

### Разрешение конфликтов и санкции

Два класса нарушений. Пользовательские конфликты разрешаются протокольными правилами без санкций. Валидаторский equivocation — через аннулирование конфликтующих сообщений и санкции.

#### Пользовательские конфликты

**Двойной блок аккаунта** (два блока с одним prev_hash): разрешается правилом lower H(block) wins. Без санкции.

**Невалидный proposal**: валидаторы отклоняют, переход ко второму месту. Без санкции (потерянный coinbase — достаточное наказание).

#### Валидаторские нарушения

Два типа нарушений. Оба доказуемы криптографически — подписанные конфликтующие сообщения. Санкции возникают только из подписанного доказательства, не из отсутствия сообщения.

**Proposal conflict** — победитель публикует два разных proposal для одного τ₂ (одинаковые (node_id, window_index), разный proposal_hash):

```
Сразу: status = suspended, chain_length = 0
```

**Reveal conflict** — узел публикует два разных VDF_Reveal для одного τ₂ (одинаковые (node_id, window_index), разный endpoint):

```
Сразу: status = suspended, chain_length = 0
```

Молчание победителя (proposal не опубликован) не является нарушением. Победитель теряет coinbase окна. chain_length не затрагивается (VDF_Reveal доказывает присутствие). Fallback на следующий ticket без санкций.

Санкции вступают в силу с следующего τ₂. Active set текущего τ₂ зафиксирован в его начале и не меняется до закрытия.

#### Equivocation proof

```
EquivocationProof:
  type             1B     <- proposal_conflict | reveal_conflict
  node_id         32B
  evidence_a             <- первое подписанное сообщение
  evidence_b             <- второе конфликтующее подписанное сообщение
```

Публикует любой узел обнаруживший конфликт. Верификация: оба сообщения подписаны одним node_id с одинаковым window_index и разным содержимым. Подписи FN-DSA-512 криптографически верифицируемы.

Proof включается в канонический набор τ₂. При финализации state transition применяет санкцию к node_id.

---

## Адреса и переводы

### Полный флоу перевода

```
1. Боб: кошелёк создаёт аккаунт -> account_id (постоянный адрес)
2. Боб -> Алисе: "отправь на mt4ZGfe..." (account_id)
3. Алиса формирует StateBlock в своей цепочке:
   prev_hash: хэш её предыдущего блока
   link: account_id Боба
   link_amount: 50 Ɉ
   balance: 49.999 Ɉ (100 - 50 - 0.001 fee)
4. Алиса подписывает FN-DSA-512
5. Алиса рассылает блок узлам сети
6. Каждый узел проверяет:
   FN-DSA-512 подпись валидна для pubkey Алисы
   prev_hash совпадает с frontier Алисы
   fee = 100 - 49.999 - 50 = 0.001 Ɉ >= min_fee
   balance >= 0
   link_amount > 0
7. Блок распространяется через P2P gossip
8. Победитель лотереи включает блок в канонический набор τ₂
9. При финализации proposal:
   Баланс Алисы: 50 Ɉ (из StateBlock)
   Баланс Боба: увеличен на 50 Ɉ (детерминированно)
```

### Баланс

Баланс аккаунта — одно число в таблице аккаунтов. Обновляется при финализации: исходящие переводы (из StateBlock отправителя) и входящие зачисления (детерминированно по финализированным блокам).

Бэкап = seed (для деривации приватного ключа FN-DSA-512).

---

## Эмиссия

### Единица

1 секунда Montana = 1 $MONT = 1 Ɉ = 1 000 mɈ = 1 000 000 μɈ = 1 000 000 000 nɈ

Точность: 9 знаков после запятой (наносекунда). Все расчёты эмиссии в nɈ (целочисленная арифметика, без плавающей точки).

### Issuance schedule

Одна секунда протокольного времени порождает одну монету. С первого блока и навсегда.

| Параметр | Значение |
|----------|----------|
| Генезис | 09.01.2026 00:00:00 MSK |
| REWARD | 600 000 000 000 nɈ (600 Ɉ) |

### Функция награды

```
reward(height) = 600_000_000_000 nɈ
```

Каждое окно τ₂ длится 600 секунд и создаёт ровно 600 Ɉ. Без халвингов, без фаз, без исключений. Одна константа на весь горизонт существования сети.

### Supply audit

```
supply(height) = 600_000_000_000 × (height + 1) nɈ
```

Одно умножение. Проверяемо каждым узлом в каждом τ₂. O(1).

### Инфляция

Supply растёт линейно. Инфляция снижается асимптотически к нулю — константная эмиссия делится на растущий supply:

```
Год 1:     100%
Год 2:      50%
Год 5:      20%
Год 10:     10%
Год 50:      2%
Год 100:     1%
Год 1000:    0.1%
```

### Bootstrap

Ранние участники получают непропорционально большую долю через вероятность лотереи, а не через повышенную награду. При 10 узлах каждый побеждает примерно раз в 100 минут. При 100 000 — раз в 2 года. Bootstrap встроен в математику лотереи, а не в расписание эмиссии.

### Распределение

Победитель τ₂ получает всю эмиссию + все комиссии окна через coinbase-блок в своей цепочке. Одно правило. Неизменно с генезиса.

Базовый бюджет эмиссии постоянный в единицах протокола: 600 Ɉ/τ₂ + комиссии. Реальный бюджет безопасности в покупательной способности зависит от рынка.

1 Ɉ = 1 секунда описывает скорость эмиссии. Не ценовой peg, не гарантия покупательной способности.

---

## Пропускная способность

Размер StateBlock: ~779B.

| Канал узла | TPS |
|-----------|-----|
| 10 Mbps | ~1 620 |
| 100 Mbps | ~16 200 |
| 1 Gbps | ~162 000 |

### Адаптивный размер окна

Пересчёт в τ₃:

- Заполненность > 80% → увеличение размера окна
- Заполненность < 20% → уменьшение размера окна
- Шаг: ±20% за τ₃
- Диапазон: 1 MB — 100 MB

---

## Хранение

### Состояние (Global State)

| Аккаунтов | Размер таблицы |
|-----------|---------------|
| 1M | ~1 GB |
| 10M | ~10 GB |
| 100M | ~100 GB |

### История блоков

| TPS | Архивный (20 лет) | Полный (112 дней) | Pruned | Light |
|-----|-------------------|-------------------|--------|-------|
| 7 | ~3.4 TB | ~42 GB | Account Table | Account Table |
| 100 | ~49 TB | ~600 GB | Account Table | Account Table |
| 1000 | ~486 TB | ~5.9 TB | Account Table | Account Table |

| Тип узла | Содержимое | Лотерея |
|----------|-----------|---------|
| Полный | Скользящее окно 8 × τ₃ + Global State + proposals | weight × 1 |
| Архивный | Полная история от генезиса | weight × 1 |
| Pruned | Global State + proposals | Наблюдатель |
| Light | Global State | Наблюдатель |

Узел самостоятельно выбирает тип. Участие в лотерее: полный или архивный узел. Вес определяется только количеством подписанных окон. Тип узла на вес не влияет. Архивный узел — добровольная роль. Протокол хранит доказательства (подписанные proposals навсегда). Рынок хранит исторические данные (тела блоков). Консенсус не зависит от архивов.

### Fast Sync (новый узел)

1. Цепочка proposals от генезиса — проверка Beacon-цепочки и подписей победителей (мегабайты)
2. State Root из последнего τ₃ (покрывает весь Global State)
3. Global State snapshot от пиров: каноническая сериализация всех листьев Merkle-дерева состояния (Account Table + Pending Transfer Table + Node Table + Expiry Queue). Верификация: пересчёт Merkle root из полученных листьев, сравнение с State Root из proposal τ₃. `MerkleRoot(snapshot_leaves) == state_root_from_proposal`
4. Блоки аккаунтов за последние 112 дней (скользящее окно)
5. Узел синхронизирован и готов к участию

Proposals доказывают цепочку state commitments. Snapshot восстанавливает содержимое состояния через пересчёт Merkle root. Блоки скользящего окна обеспечивают верификацию недавних переходов.

---

## Ключи

```
seed
├── Аккаунт:  FN-DSA-512 keypair → account_id = SHA-256("mt-account" || suite_id || account_pubkey)
└── Узел:     FN-DSA-512 keypair → node_id = SHA-256("mt-node" || node_pubkey)
```

Один seed порождает два FN-DSA-512 keypair: для аккаунта (подпись блоков) и для узла (подпись proposals и Service VDF endpoints). account_id и node_id выводятся из публичных ключей, верифицируемы без знания seed. Бэкап = seed.

---

## Криптографическая реализация

### Primitive layer

Собственная реализация криптографических примитивов запрещена. Только audited библиотеки с constant-time гарантиями и опубликованными test vectors.

| Примитив | Стандарт | Роль |
|----------|----------|------|
| SHA-256 | FIPS 180-4 | Beacon VDF, Service VDF, адреса, Merkle-деревья |
| FN-DSA-512 | Selected NIST candidate, forthcoming FIPS 206 | Подписи блоков аккаунтов и proposals |

### Consensus encoding layer

Консенсусно-критическая поверхность: каноническая сериализация, Merkle layout и domain separation. Разная сериализация одного объекта = разный хэш = форк. Требования:

- Fixed binary encoding для каждого консенсусного объекта
- Length-prefix кодирование полей, фиксированный endianness (little-endian)
- Domain separation для всех хэшей:

| Домен | Контекст |
|-------|----------|
| `mt-block` | Хэширование блоков аккаунтов |
| `mt-header` | Хэширование proposal headers |
| `mt-account` | Деривация account_id |
| `mt-pending` | Хэширование pending entries |
| `mt-merkle-leaf` | Листья Merkle-деревьев |
| `mt-merkle-node` | Внутренние узлы Merkle-деревьев |
| `mt-beacon` | Beacon VDF seed |
| `mt-service` | Service VDF seed |
| `mt-equivocation` | Хэширование equivocation proofs |
| `mt-confirmation` | Хэширование async confirmations |

- Альтернативные сериализации запрещены
- Test vectors для каждого консенсусного объекта
- Cross-implementation conformance tests перед запуском mainnet

### Protocol layer

Собственная реализация поверх криптографического ядра:

| Компонент | Назначение |
|-----------|------------|
| Merkle-деревья | State Root, blocks_root (из SHA-256 вызовов) |
| VDF scheduling | Управление Beacon и Service цепочками |
| State machine | Account Table, Pending Transfers, state transitions |
| P2P gossip | Распространение блоков и proposals |

### Инфраструктура

| Библиотека | Назначение |
|------------|------------|
| RocksDB | Хранение Account Table и блоков |
| libp2p | P2P транспорт |

Production: Rust.

---

## Архитектура

```
┌─────────────────────────────────┐
│  Wallet                         │
│  Кошелёк, баланс, переводы     │
│  FN-DSA-512 keypair            │
└──────────────┬──────────────────┘
               │
┌──────────────┴──────────────────┐
│  TimeChain                      │
│                                 │
│  Beacon ──→ Service ──→ Execution
│  (часы)    (служба)   (состояние)
│                                 │
│  Account Chain (Block Lattice)  │
│  Account Table, Proposals       │
│  SHA-256, FN-DSA-512            │
└─────────────────────────────────┘
```
