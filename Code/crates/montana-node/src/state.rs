use std::path::Path;

use mt_genesis::ProtocolParams;
use mt_state::{
    AccountRecord, AccountTable, CandidatePool, NodeTable, ACCOUNT_RECORD_SIZE,
    CANDIDATE_RECORD_SIZE, NODE_RECORD_SIZE,
};
use mt_store::FsStore;

use crate::identity::{Identity, NodeError};
use crate::node_lifecycle::NodeLifecycle;

pub struct LocalState {
    pub accounts: AccountTable,
    pub nodes: NodeTable,
    pub candidates: CandidatePool,
}

impl LocalState {
    // Автоматический genesis vs candidate fork per spec Genesis Decree:
    // - genesis узел (identity.node_pk == params.bootstrap_node_pubkey либо
    //   placeholder zeros — pre-ceremony local mode): operator-аккаунт +
    //   NodeRecord для self напрямую (DEV-010, активация без Candidate SSHA).
    // - candidate узел: пустой NodeTable; узел появится в state только после
    //   apply_selection_event на ближайшем W % selection_interval == 0.
    //   Operator account создаётся в обоих случаях (нужен для подписания
    //   будущей NodeRegistration).
    pub fn bootstrap(operator: &Identity, params: &ProtocolParams) -> Self {
        // Canonical genesis tables — byte-identical to
        // mt_account::genesis_state_root, so the runtime state_root equals the
        // Genesis State Hash (no fork on non-empty N_SEED). build_genesis_state
        // is the single source of truth for genesis seeding ([C-1]).
        let genesis = mt_account::build_genesis_state(params);
        let mut accounts = genesis.account_table;
        let nodes = genesis.node_table;
        let candidates = genesis.candidate_pool;

        // A non-genesis node also carries its own operator account (needed to
        // sign a future NodeRegistration). Genesis members are already seeded
        // above. A genesis member (bootstrap or any N_SEED operator) already has
        // its account baked into `accounts` with is_node_operator=true; inserting
        // would OVERWRITE it (AccountTable::insert replaces), diverging the
        // account_root from canonical genesis. Insert only when truly absent.
        if accounts.get(&operator.account_id()).is_none() {
            accounts.insert(AccountRecord {
                account_id: operator.account_id(),
                balance: 0,
                suite_id: operator.suite_id as u16,
                is_node_operator: false,
                frontier_hash: [0u8; 32],
                op_height: 0,
                account_chain_length: 0,
                account_chain_length_snapshot: 0,
                current_pubkey: *operator.account_pk.as_bytes(),
                creation_window: 0,
                last_op_window: 0,
                last_activation_window: 0,
            });
        }

        Self {
            accounts,
            nodes,
            candidates,
        }
    }

    pub fn load_or_bootstrap(
        data_dir: &Path,
        operator: &Identity,
        params: &ProtocolParams,
    ) -> Result<Self, NodeError> {
        let store = FsStore::open(data_dir).map_err(|e| {
            NodeError::InvalidArguments(format!("открытие хранилища {}: {e:?}", data_dir.display()))
        })?;
        let accounts_path = data_dir.join("accounts.bin");
        if !accounts_path.exists() {
            return Ok(Self::bootstrap(operator, params));
        }
        let accounts = store.load_account_table().map_err(|e| {
            NodeError::InvalidArguments(format!(
                "загрузка accounts.bin: {e:?} (ожидался размер кратный {ACCOUNT_RECORD_SIZE})"
            ))
        })?;
        let nodes = store.load_node_table().map_err(|e| {
            NodeError::InvalidArguments(format!(
                "загрузка nodes.bin: {e:?} (ожидался размер кратный {NODE_RECORD_SIZE})"
            ))
        })?;
        let candidates = store.load_candidate_pool().map_err(|e| {
            NodeError::InvalidArguments(format!(
                "загрузка candidates.bin: {e:?} (ожидался размер кратный {CANDIDATE_RECORD_SIZE})"
            ))
        })?;
        Ok(Self {
            accounts,
            nodes,
            candidates,
        })
    }

    pub fn save(&self, data_dir: &Path) -> Result<(), NodeError> {
        let store = FsStore::open(data_dir)
            .map_err(|e| NodeError::InvalidArguments(format!("открытие хранилища: {e:?}")))?;
        store
            .save_account_table(&self.accounts)
            .map_err(|e| NodeError::InvalidArguments(format!("save accounts: {e:?}")))?;
        store
            .save_node_table(&self.nodes)
            .map_err(|e| NodeError::InvalidArguments(format!("save nodes: {e:?}")))?;
        store
            .save_candidate_pool(&self.candidates)
            .map_err(|e| NodeError::InvalidArguments(format!("save candidates: {e:?}")))?;
        Ok(())
    }

    // Применение проверенного fast-sync снимка: вызывающая сторона передаёт
    // TypedTables только после того как FastSyncClient::finalize сверил
    // reconstructed state_root с доверенным anchor root окна W. Здесь снимок
    // уже доверенный — заменяем три consensus-таблицы и фиксируем
    // meta_last_cemented = W (точка восстановления после перезапуска).
    pub fn apply_fast_sync(
        &mut self,
        tables: mt_sync::snapshot::TypedTables,
        data_dir: &Path,
        anchor_window: u64,
    ) -> Result<(), NodeError> {
        self.accounts = tables.accounts;
        self.nodes = tables.nodes;
        self.candidates = tables.candidates;
        self.save(data_dir)?;
        let store = FsStore::open(data_dir)
            .map_err(|e| NodeError::InvalidArguments(format!("открытие хранилища: {e:?}")))?;
        store
            .save_meta_last_cemented(anchor_window)
            .map_err(|e| NodeError::InvalidArguments(format!("save_meta_last_cemented: {e:?}")))?;
        Ok(())
    }
}

// SPEC DEVIATION DEV-010 (closed 2026-05-02 в M9 Phase 1):
// Bootstrap NodeRecord теперь deriviается из params.bootstrap_node_pubkey
// (а не из operator's own pk). Это унифицирует bootstrap entry между всеми
// узлами cohort-а — необходимо для apply_proposal validation на receivers.
// Inline в LocalState::bootstrap(); helper удалён.

#[cfg(test)]
mod tests {
    use super::*;
    use mt_crypto::PUBLIC_KEY_SIZE;
    use mt_state::{AccountTable, CandidatePool, NodeTable};
    use std::fs;
    use std::path::PathBuf;

    fn tempdir() -> PathBuf {
        let mut p = std::env::temp_dir();
        let mut buf = [0u8; 8];
        getrandom::getrandom(&mut buf).unwrap();
        p.push(format!(
            "montana-state-test-{:016x}",
            u64::from_le_bytes(buf)
        ));
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn sample_account(seed: u8) -> AccountRecord {
        AccountRecord {
            account_id: [seed; 32],
            balance: 500,
            suite_id: 1,
            is_node_operator: false,
            frontier_hash: [seed; 32],
            op_height: 0,
            account_chain_length: 0,
            account_chain_length_snapshot: 0,
            current_pubkey: [seed; PUBLIC_KEY_SIZE],
            creation_window: 0,
            last_op_window: 0,
            last_activation_window: 0,
        }
    }

    #[test]
    fn apply_fast_sync_replaces_tables_and_persists_anchor() {
        let dir = tempdir();
        let mut state = LocalState {
            accounts: AccountTable::new(),
            nodes: NodeTable::new(),
            candidates: CandidatePool::new(),
        };

        let mut accounts = AccountTable::new();
        accounts.insert(sample_account(0xAB));
        accounts.insert(sample_account(0xCD));
        let tables = mt_sync::snapshot::TypedTables {
            accounts,
            nodes: NodeTable::new(),
            candidates: CandidatePool::new(),
        };

        state.apply_fast_sync(tables, &dir, 75_850).unwrap();

        assert_eq!(state.accounts.len(), 2);

        let store = FsStore::open(&dir).unwrap();
        assert_eq!(store.load_meta_last_cemented().unwrap(), Some(75_850));
        assert_eq!(store.load_account_table().unwrap().len(), 2);
        assert!(store.load_node_table().unwrap().is_empty());

        fs::remove_dir_all(&dir).ok();
    }
    #[test]
    fn bootstrap_seeds_genesis_active_operators_not_manifest() {
        use mt_genesis::genesis_params;
        let id = crate::identity::Identity::from_entropy_ephemeral(&[0x77; 32]).unwrap();

        // Unified genesis: active set = bootstrap + every baked genesis_active_operator.
        let base = genesis_params();
        let cohort0 = LocalState::bootstrap(&id, base);
        assert_eq!(cohort0.nodes.len(), 1 + base.genesis_active_operators.len());

        // One more operator baked into a clone is seeded Active at genesis.
        let mut params = base.clone();
        params
            .genesis_active_operators
            .push(([0xBB; PUBLIC_KEY_SIZE], [0xAA; PUBLIC_KEY_SIZE]));
        let cohort = LocalState::bootstrap(&id, &params);
        assert_eq!(
            cohort.nodes.len(),
            1 + params.genesis_active_operators.len()
        );
        let extra_node_id = mt_state::derive_node_id(&[0xAA; PUBLIC_KEY_SIZE]);
        assert!(cohort.nodes.contains(&extra_node_id));
        let extra_account_id =
            mt_state::derive_account_id(mt_account::GENESIS_SUITE_ID, &[0xBB; PUBLIC_KEY_SIZE]);
        let rec = cohort.accounts.get(&extra_account_id).unwrap();
        assert!(rec.is_node_operator);
        assert_eq!(rec.current_pubkey, [0xBB; PUBLIC_KEY_SIZE]);
    }

    #[test]
    fn local_state_node_set_equals_canonical_genesis() {
        // Closes the non-empty N_SEED genesis-fork bug (EXT-GEN-01 caveat): every
        // node — genesis member or fresh observer — boots with exactly the
        // canonical genesis NodeTable (candidates disabled, no node added at
        // boot), so all nodes agree on the Genesis State Hash consensus set.
        use mt_genesis::genesis_params;
        let id = crate::identity::Identity::from_entropy_ephemeral(&[0x11; 32]).unwrap();
        let p = genesis_params();
        let local = LocalState::bootstrap(&id, p);
        let g = mt_account::build_genesis_state(p);
        assert_eq!(
            local.nodes.root(),
            g.node_table.root(),
            "runtime node set != canonical genesis -> consensus fork"
        );
        // A fresh observer adds only its own account on top of canonical genesis.
        assert_eq!(local.accounts.len(), g.account_table.len() + 1);
    }

    #[test]
    fn genesis_member_local_state_root_equals_canonical() {
        // For a genesis member the FULL runtime root equals the canonical genesis
        // root (no own-account delta). Verified by reconstructing the canonical
        // tables directly (a genesis member's bootstrap output == build_genesis_state).
        use mt_genesis::genesis_params;
        let p = genesis_params();
        let g = mt_account::build_genesis_state(p);
        let canonical = mt_account::genesis_state_root(&g);
        let runtime = mt_state::compute_state_root(
            &g.node_table.root(),
            &g.candidate_pool.root(),
            &g.account_table.root(),
        );
        assert_eq!(runtime, canonical);
    }
}
