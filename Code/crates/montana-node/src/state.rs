use std::path::Path;

use mt_crypto::PUBLIC_KEY_SIZE;
use mt_genesis::{GenesisPeer, ProtocolParams};
use mt_state::{
    AccountRecord, AccountTable, CandidatePool, NodeRecord, NodeTable, ACCOUNT_RECORD_SIZE,
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
    //   NodeRecord для self напрямую (DEV-010, активация без Candidate VDF).
    // - candidate узел: пустой NodeTable; узел появится в state только после
    //   apply_selection_event на ближайшем W % selection_interval == 0.
    //   Operator account создаётся в обоих случаях (нужен для подписания
    //   будущей NodeRegistration).
    pub fn bootstrap(
        operator: &Identity,
        params: &ProtocolParams,
        extra_actives: &[&GenesisPeer],
    ) -> Self {
        let is_genesis = NodeLifecycle::is_bootstrap_node(operator, params);
        let mut accounts = AccountTable::new();

        // Bootstrap account из Genesis Decree — присутствует во ВСЕХ узлах
        // (нужен для emission target в apply_proposal на receivers).
        let bootstrap_account_id =
            mt_state::derive_account_id(operator.suite_id as u16, &params.bootstrap_account_pubkey);
        accounts.insert(AccountRecord {
            account_id: bootstrap_account_id,
            balance: 0,
            suite_id: operator.suite_id as u16,
            is_node_operator: true,
            frontier_hash: [0u8; 32],
            op_height: 0,
            account_chain_length: 0,
            account_chain_length_snapshot: 0,
            current_pubkey: params.bootstrap_account_pubkey,
            creation_window: 0,
            last_op_window: 0,
            last_activation_window: 0,
        });

        // Operator's own account — отдельная запись если operator != bootstrap
        // (in is_genesis case identity.account_id() == bootstrap_account_id —
        // тот же account_id, insert override без эффекта).
        if !is_genesis {
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

        // Bootstrap NodeRecord — всегда в NodeTable (Genesis Decree), независимо
        // от того bootstrap ли локальный operator. Receivers нужен для validate
        // ProposalHeader.proposer_node_id и apply_emission winner_id lookup.
        let mut nodes = NodeTable::new();
        let bootstrap_node_id = mt_state::derive_node_id(&params.bootstrap_node_pubkey);
        nodes.insert(mt_state::NodeRecord {
            node_id: bootstrap_node_id,
            node_pubkey: params.bootstrap_node_pubkey,
            suite_id: operator.suite_id as u16,
            operator_account_id: bootstrap_account_id,
            start_window: 0,
            chain_length: 1,
            chain_length_snapshot: 1,
            chain_length_checkpoints: [0; 6],
            last_confirmation_window: 0,
        });

        // Test-cohort pre-seed: each peer with `force_active = true` becomes an
        // Active operator at genesis (NodeTable + AccountRecord), bypassing τ₂
        // candidate VDF wait. Production manifest has no such peers — this
        // branch is a no-op in mainnet runs.
        for peer in extra_actives {
            let npk_bytes = hex_to_pubkey(
                peer.node_pubkey_hex
                    .as_deref()
                    .expect("manifest validation guarantees node_pubkey_hex for force_active"),
            );
            let apk_bytes = hex_to_pubkey(
                peer.account_pubkey_hex
                    .as_deref()
                    .expect("manifest validation guarantees account_pubkey_hex for force_active"),
            );
            let suite = operator.suite_id as u16;
            let extra_node_id = mt_state::derive_node_id(&npk_bytes);
            let extra_account_id = mt_state::derive_account_id(suite, &apk_bytes);
            accounts.insert(AccountRecord {
                account_id: extra_account_id,
                balance: 0,
                suite_id: suite,
                is_node_operator: true,
                frontier_hash: [0u8; 32],
                op_height: 0,
                account_chain_length: 0,
                account_chain_length_snapshot: 0,
                current_pubkey: apk_bytes,
                creation_window: 0,
                last_op_window: 0,
                last_activation_window: 0,
            });
            nodes.insert(NodeRecord {
                node_id: extra_node_id,
                node_pubkey: npk_bytes,
                suite_id: suite,
                operator_account_id: extra_account_id,
                start_window: 0,
                chain_length: 1,
                chain_length_snapshot: 1,
                chain_length_checkpoints: [0; 6],
                last_confirmation_window: 0,
            });
        }

        Self {
            accounts,
            nodes,
            candidates: CandidatePool::new(),
        }
    }

    pub fn load_or_bootstrap(
        data_dir: &Path,
        operator: &Identity,
        params: &ProtocolParams,
        extra_actives: &[&GenesisPeer],
    ) -> Result<Self, NodeError> {
        let store = FsStore::open(data_dir).map_err(|e| {
            NodeError::InvalidArguments(format!("открытие хранилища {}: {e:?}", data_dir.display()))
        })?;
        let accounts_path = data_dir.join("accounts.bin");
        if !accounts_path.exists() {
            return Ok(Self::bootstrap(operator, params, extra_actives));
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

fn hex_to_pubkey(h: &str) -> [u8; PUBLIC_KEY_SIZE] {
    let mut out = [0u8; PUBLIC_KEY_SIZE];
    for (i, byte) in out.iter_mut().enumerate() {
        let hi = (h.as_bytes()[2 * i] as char).to_digit(16).expect("hex");
        let lo = (h.as_bytes()[2 * i + 1] as char).to_digit(16).expect("hex");
        *byte = ((hi << 4) | lo) as u8;
    }
    out
}

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
    fn bootstrap_pre_seeds_force_active_extras() {
        use mt_genesis::{genesis_params, GenesisPeer};
        let params = genesis_params();
        let dir = tempdir();
        // build an Identity by writing one fresh
        let id = crate::identity::Identity::from_entropy_ephemeral(&[0x77; 32]).unwrap();
        let mut npk_hex = String::with_capacity(3904);
        for b in [0xAAu8; PUBLIC_KEY_SIZE] {
            npk_hex.push_str(&format!("{b:02x}"));
        }
        let mut apk_hex = String::with_capacity(3904);
        for b in [0xBBu8; PUBLIC_KEY_SIZE] {
            apk_hex.push_str(&format!("{b:02x}"));
        }
        let extra = GenesisPeer {
            label: "vilnius-test".into(),
            multiaddr: "/ip4/0.0.0.0/tcp/8444".into(),
            peer_id: "QmTest".into(),
            account_id_hex: "0".repeat(64),
            node_id_hex: "1".repeat(64),
            bootstrap: false,
            force_active: true,
            node_pubkey_hex: Some(npk_hex),
            account_pubkey_hex: Some(apk_hex),
        };
        let state = LocalState::bootstrap(&id, params, &[&extra]);
        // bootstrap node + extra Active = 2 nodes
        assert_eq!(state.nodes.len(), 2);
        let extra_node_id = mt_state::derive_node_id(&[0xAA; PUBLIC_KEY_SIZE]);
        assert!(state.nodes.contains(&extra_node_id));
        // extra account present, is_node_operator=true
        let extra_account_id = mt_state::derive_account_id(1, &[0xBB; PUBLIC_KEY_SIZE]);
        let rec = state.accounts.get(&extra_account_id).unwrap();
        assert!(rec.is_node_operator);
        assert_eq!(rec.current_pubkey, [0xBB; PUBLIC_KEY_SIZE]);
        fs::remove_dir_all(&dir).ok();
    }
}
