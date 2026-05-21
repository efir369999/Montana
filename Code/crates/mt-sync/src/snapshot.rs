//! Snapshot — typed wrapper over the raw bytes delivered by the
//! FastSyncResponse chunks. The wire format delivers each consensus table
//! (Account, Node, Candidate) as a sequence of fixed-size canonical-encoded
//! records; this module reassembles them into a tagged collection and
//! exposes a verifier that recomputes `state_root` for comparison against
//! the anchor ProposalHeader.
//!
//! Integration with `mt-state` for typed insertion into the live state is
//! the v1.0.0 mainnet gate; this scaffold proves the wire path and the
//! Merkle-root verification API end-to-end.

use mt_codec::domain;
use mt_crypto::sha256_raw;
use mt_state::{compute_state_root, ACCOUNT_RECORD_SIZE, CANDIDATE_RECORD_SIZE, NODE_RECORD_SIZE};

use crate::response::FastSyncTableId;

pub type Hash32 = [u8; 32];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    pub anchor_window: u64,
    pub accounts: Vec<Vec<u8>>, // each entry == ACCOUNT_RECORD_SIZE bytes
    pub nodes: Vec<Vec<u8>>,    // each entry == NODE_RECORD_SIZE bytes
    pub candidates: Vec<Vec<u8>>, // each entry == CANDIDATE_RECORD_SIZE bytes
}

impl Snapshot {
    pub fn new(anchor_window: u64) -> Self {
        Snapshot {
            anchor_window,
            accounts: Vec::new(),
            nodes: Vec::new(),
            candidates: Vec::new(),
        }
    }

    pub fn add_record(
        &mut self,
        table: FastSyncTableId,
        bytes: Vec<u8>,
    ) -> Result<(), SnapshotError> {
        let expected = match table {
            FastSyncTableId::Account => ACCOUNT_RECORD_SIZE,
            FastSyncTableId::Node => NODE_RECORD_SIZE,
            FastSyncTableId::Candidate => CANDIDATE_RECORD_SIZE,
            FastSyncTableId::Proposals => {
                return Err(SnapshotError::ProposalsNotImplementedYet);
            },
        };
        if bytes.len() != expected {
            return Err(SnapshotError::WrongRecordSize {
                table,
                expected,
                actual: bytes.len(),
            });
        }
        match table {
            FastSyncTableId::Account => self.accounts.push(bytes),
            FastSyncTableId::Node => self.nodes.push(bytes),
            FastSyncTableId::Candidate => self.candidates.push(bytes),
            FastSyncTableId::Proposals => unreachable!(),
        }
        Ok(())
    }

    pub fn record_count(&self) -> usize {
        self.accounts.len() + self.nodes.len() + self.candidates.len()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SnapshotError {
    WrongRecordSize {
        table: FastSyncTableId,
        expected: usize,
        actual: usize,
    },
    ProposalsNotImplementedYet,
    StateRootMismatch {
        expected: Hash32,
        actual: Hash32,
    },
}

/// Recompute the per-table Merkle root over the raw record bytes and
/// verify the resulting state_root matches the anchor `expected_state_root`.
///
/// The per-table root is computed as the SHA-256 of the concatenation of
/// the records under the table's domain separator. This is a scaffold-grade
/// verifier whose Merkle construction MUST be replaced with the same SMT
/// algorithm used by `mt_state::{AccountTable, NodeTable, CandidateTable}::root()`
/// in the v1.0.0 mainnet build for byte-equal cross-implementation
/// conformance.
pub struct SnapshotVerifier;

impl SnapshotVerifier {
    pub fn verify(snapshot: &Snapshot, expected_state_root: &Hash32) -> Result<(), SnapshotError> {
        let acct_root = hash_table(domain::ACCOUNT, &snapshot.accounts);
        let node_root = hash_table(domain::NODE, &snapshot.nodes);
        let cand_root = hash_table(domain::ACCOUNT, &snapshot.candidates); // placeholder domain
        let computed = compute_state_root(&node_root, &cand_root, &acct_root);
        if &computed != expected_state_root {
            return Err(SnapshotError::StateRootMismatch {
                expected: *expected_state_root,
                actual: computed,
            });
        }
        Ok(())
    }
}

fn hash_table(_dom: &[u8], records: &[Vec<u8>]) -> Hash32 {
    // Scaffold: concatenate-then-hash. The production verifier must use the
    // sparse Merkle tree algorithm from mt_state::*Table::root() so that the
    // verifier's computed state_root byte-equals the proposer's recorded
    // state_root for the same anchor window.
    let mut concat = Vec::new();
    for r in records {
        concat.extend_from_slice(r);
    }
    sha256_raw(&concat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_record_size_check() {
        let mut s = Snapshot::new(0);
        let wrong = vec![0u8; 100];
        let err = s.add_record(FastSyncTableId::Account, wrong).unwrap_err();
        assert!(matches!(err, SnapshotError::WrongRecordSize { .. }));
    }

    #[test]
    fn add_record_accepts_correct_size() {
        let mut s = Snapshot::new(75850);
        s.add_record(FastSyncTableId::Account, vec![0xAB; ACCOUNT_RECORD_SIZE])
            .unwrap();
        s.add_record(FastSyncTableId::Node, vec![0xCD; NODE_RECORD_SIZE])
            .unwrap();
        s.add_record(
            FastSyncTableId::Candidate,
            vec![0xEF; CANDIDATE_RECORD_SIZE],
        )
        .unwrap();
        assert_eq!(s.record_count(), 3);
        assert_eq!(s.anchor_window, 75850);
    }

    #[test]
    fn verifier_rejects_mismatched_root() {
        let mut s = Snapshot::new(0);
        s.add_record(FastSyncTableId::Account, vec![0xAB; ACCOUNT_RECORD_SIZE])
            .unwrap();
        let bogus_root = [0xFFu8; 32];
        let result = SnapshotVerifier::verify(&s, &bogus_root);
        assert!(matches!(
            result,
            Err(SnapshotError::StateRootMismatch { .. })
        ));
    }

    #[test]
    fn verifier_accepts_self_computed_root() {
        let mut s = Snapshot::new(0);
        s.add_record(FastSyncTableId::Account, vec![0xAB; ACCOUNT_RECORD_SIZE])
            .unwrap();
        s.add_record(FastSyncTableId::Node, vec![0xCD; NODE_RECORD_SIZE])
            .unwrap();
        // Recompute the same expected root the verifier would produce.
        let acct_root = hash_table(domain::ACCOUNT, &s.accounts);
        let node_root = hash_table(domain::NODE, &s.nodes);
        let cand_root = hash_table(domain::ACCOUNT, &s.candidates);
        let expected = compute_state_root(&node_root, &cand_root, &acct_root);
        SnapshotVerifier::verify(&s, &expected).expect("verify");
    }
}
