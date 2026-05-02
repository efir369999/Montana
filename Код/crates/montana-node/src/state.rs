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
    //   NodeRecord для self напрямую (DEV-010, активация без Candidate VDF).
    // - candidate узел: пустой NodeTable; узел появится в state только после
    //   apply_selection_event на ближайшем W % selection_interval == 0.
    //   Operator account создаётся в обоих случаях (нужен для подписания
    //   будущей NodeRegistration).
    pub fn bootstrap(operator: &Identity, params: &ProtocolParams) -> Self {
        let is_genesis = NodeLifecycle::is_bootstrap_node(operator, params);
        let mut accounts = AccountTable::new();
        accounts.insert(AccountRecord {
            // SPEC DEVIATION DEV-010
            account_id: operator.account_id(),
            balance: 0,
            suite_id: operator.suite_id as u16,
            is_node_operator: is_genesis,
            frontier_hash: [0u8; 32],
            op_height: 0,
            account_chain_length: 0,
            account_chain_length_snapshot: 0,
            current_pubkey: *operator.account_pk.as_bytes(),
            creation_window: 0,
            last_op_window: 0,
            last_activation_window: 0,
        });

        let mut nodes = NodeTable::new();
        if is_genesis {
            nodes.insert(genesis_bootstrap_node_record(operator)); // SPEC DEVIATION DEV-010
        }
        // Candidate узел: пустой NodeTable. CandidatePool тоже пустой —
        // узел добавится через apply_noderegistrations_batch когда vdf_chain_length
        // достигнет τ₂; затем активируется через apply_selection_event.

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
}

// SPEC DEVIATION DEV-010: genesis bootstrap NodeRecord — узел = bootstrap_node_pubkey
// своей локальной сети, активирован через genesis state без Candidate VDF + selection.
// Per spec: chain_length=1 (= результат selection event activation), start_window=0.
// Acknowledged автором 2026-04-28. См. docs/SPEC_DEVIATIONS.md DEV-010.
fn genesis_bootstrap_node_record(operator: &Identity) -> mt_state::NodeRecord {
    mt_state::NodeRecord {
        node_id: operator.node_id(),
        node_pubkey: *operator.node_pk.as_bytes(),
        suite_id: operator.suite_id as u16,
        operator_account_id: operator.account_id(),
        start_window: 0,
        chain_length: 1,
        chain_length_snapshot: 1,
        chain_length_checkpoints: [0; 6],
        last_confirmation_window: 0,
    }
}
