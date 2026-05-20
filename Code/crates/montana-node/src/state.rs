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

        // Bootstrap account из Genesis Decree — присутствует во ВСЕХ узлах
        // (нужен для emission target в apply_proposal на receivers).
        let bootstrap_account_id = mt_state::derive_account_id(
            operator.suite_id as u16,
            &params.bootstrap_account_pubkey,
        );
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

// SPEC DEVIATION DEV-010 (closed 2026-05-02 в M9 Phase 1):
// Bootstrap NodeRecord теперь deriviается из params.bootstrap_node_pubkey
// (а не из operator's own pk). Это унифицирует bootstrap entry между всеми
// узлами cohort-а — необходимо для apply_proposal validation на receivers.
// Inline в LocalState::bootstrap(); helper удалён.
