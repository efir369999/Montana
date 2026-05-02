# Montana Network — Live Status

**Updated:** 2026-05-02T16:30:04Z UTC
**Live dashboard:** [efir.org/explorer/](https://efir.org/explorer/) (auto-refresh каждые 60 сек = τ₁)

## Network summary

| Метрика | Значение |
|---|---|
| Активных узлов | **3 / 3** |
| Окно сети (max) | **8** |
| Эпоха τ₂ | 0 (8/20160 окон) |
| Σ supply (closed-form) | **234 Ɉ** |

## Узлы (Genesis-bootstrap singleton mode, M5)

### ✅ Moscow (`local`)

- **Phase:** Active
- **Текущее окно:** `4`
- **D (итераций SHA-256):** `325,000,000`
- **Баланс оператора:** `39.000 Ɉ` (`39,000,000,000 nɈ`)
- **Supply (closed-form):** `65.000 Ɉ`
- **AccountTable:** 1 записей
- **NodeTable:** 1 записей
- **account_id:** `4c290c3d5d63e84b99c30c83fb4d172e04102af4492b4d56d0642711b09e2072`
- **node_id:** `75bfaf9026405c12ef36437f08cc63c040cfe1924773dedcba0abadf8c6928a1`

### ✅ Helsinki (`91.132.142.42`)

- **Phase:** CandidateVdf
- **Текущее окно:** `8`
- **D (итераций SHA-256):** `325,000,000`
- **Баланс оператора:** `0.000 Ɉ` (`0 nɈ`)
- **Supply (closed-form):** `117.000 Ɉ`
- **AccountTable:** 1 записей
- **NodeTable:** 0 записей
- **account_id:** `19edd79c0c13b7164ed5fb00d571ba1fa26726adf1e6ef61a3f21b20fa1b42c4`
- **node_id:** `d63cc60c8367ba6be903e50bc0190d7e2e60f89f30f24d3a10dceb92613a5901`

### ✅ Frankfurt (`89.19.208.158`)

- **Phase:** CandidateVdf
- **Текущее окно:** `3`
- **D (итераций SHA-256):** `325,000,000`
- **Баланс оператора:** `0.000 Ɉ` (`0 nɈ`)
- **Supply (closed-form):** `52.000 Ɉ`
- **AccountTable:** 2 записей
- **NodeTable:** 1 записей
- **account_id:** `53560626aff44b5f0a88d7b235ef2028a3cf0517fd6fd2aa20b5566345a91e29`
- **node_id:** `5509211b179d69698913e47605d2b0ed24a91702fb6e9d0fbcd3c3c626270aab`

## Архитектура снапшота

Backend: cron на montana-moscow (Moscow node) каждую минуту собирает
`montana-node status` локально + по SSH с Helsinki + Frankfurt → JSON в
`/var/www/efir/explorer/data.json`. Frontend HTML/JS auto-refresh.

Каждый узел в текущей версии — собственный genesis bootstrap
(M5 singleton, без сетевого слоя M6). Эмиссия 13 Ɉ за окно (≈60 сек),
τ₂ = 20160 окон (≈14 дней). 1 Ɉ = 10⁹ nɈ.
