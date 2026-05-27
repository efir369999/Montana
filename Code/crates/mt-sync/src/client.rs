//! FastSync client — receiver-side reassembly and verification.
//!
//! A follower joining a long-running mesh learns the cemented anchor window
//! `W` and its `state_root` from an honest peer via the standard Proposal
//! propagation path, then requests the snapshot at `W` and feeds each
//! arriving FastSyncResponse chunk here. After every chunk announced by
//! `total_chunks` has arrived, `finalize` reconstructs the Sparse Merkle
//! `state_root` and rejects the stream unless it byte-equals the trusted
//! anchor root — the integrity gate against a peer streaming a forged state.

use crate::response::FastSyncChunk;
use crate::snapshot::{Hash32, Snapshot, SnapshotError, TypedTables};
use crate::SnapshotVerifier;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FastSyncClientError {
    TotalChunksZero,
    TotalChunksMismatch { expected: u32, actual: u32 },
    ChunkIndexOutOfRange { index: u32, total: u32 },
    DuplicateChunk { index: u32 },
    Record(SnapshotError),
    Incomplete { received: u32, total: u32 },
    Verify(SnapshotError),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AcceptOutcome {
    Progress { received: u32, total: u32 },
    Complete,
}

pub struct FastSyncClient {
    expected_state_root: Hash32,
    total_chunks: Option<u32>,
    received: BTreeSet<u32>,
    snapshot: Snapshot,
}

impl FastSyncClient {
    pub fn new(anchor_window: u64, expected_state_root: Hash32) -> Self {
        FastSyncClient {
            expected_state_root,
            total_chunks: None,
            received: BTreeSet::new(),
            snapshot: Snapshot::new(anchor_window),
        }
    }

    pub fn anchor_window(&self) -> u64 {
        self.snapshot.anchor_window
    }

    pub fn accept_chunk(
        &mut self,
        chunk: FastSyncChunk,
    ) -> Result<AcceptOutcome, FastSyncClientError> {
        if chunk.total_chunks == 0 {
            return Err(FastSyncClientError::TotalChunksZero);
        }
        match self.total_chunks {
            None => self.total_chunks = Some(chunk.total_chunks),
            Some(t) if t != chunk.total_chunks => {
                return Err(FastSyncClientError::TotalChunksMismatch {
                    expected: t,
                    actual: chunk.total_chunks,
                });
            },
            Some(_) => {},
        }
        let total = chunk.total_chunks;
        if chunk.chunk_index >= total {
            return Err(FastSyncClientError::ChunkIndexOutOfRange {
                index: chunk.chunk_index,
                total,
            });
        }
        if self.received.contains(&chunk.chunk_index) {
            return Err(FastSyncClientError::DuplicateChunk {
                index: chunk.chunk_index,
            });
        }
        for rec in chunk.records {
            self.snapshot
                .add_record(chunk.table_id, rec)
                .map_err(FastSyncClientError::Record)?;
        }
        self.received.insert(chunk.chunk_index);
        if self.received.len() as u32 == total {
            Ok(AcceptOutcome::Complete)
        } else {
            Ok(AcceptOutcome::Progress {
                received: self.received.len() as u32,
                total,
            })
        }
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.total_chunks, Some(t) if self.received.len() as u32 == t)
    }

    pub fn finalize(self) -> Result<TypedTables, FastSyncClientError> {
        let total = self.total_chunks.unwrap_or(0);
        let received = self.received.len() as u32;
        if total == 0 || received != total {
            return Err(FastSyncClientError::Incomplete { received, total });
        }
        SnapshotVerifier::verify(&self.snapshot, &self.expected_state_root)
            .map_err(FastSyncClientError::Verify)?;
        self.snapshot
            .build_tables()
            .map_err(FastSyncClientError::Verify)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::response::FastSyncTableId;
    use mt_codec::CanonicalEncode;
    use mt_crypto::PUBLIC_KEY_SIZE;
    use mt_state::{compute_state_root, AccountRecord, ACCOUNT_RECORD_SIZE};

    fn acct_bytes(seed: u8) -> Vec<u8> {
        let rec = AccountRecord {
            account_id: [seed; 32],
            balance: 1_000u128.wrapping_add(seed as u128),
            suite_id: 1,
            is_node_operator: seed % 2 == 0,
            frontier_hash: [seed; 32],
            op_height: seed as u32,
            account_chain_length: seed as u32,
            account_chain_length_snapshot: seed as u32,
            current_pubkey: [seed; PUBLIC_KEY_SIZE],
            creation_window: 0,
            last_op_window: 0,
            last_activation_window: 0,
        };
        let mut buf = Vec::with_capacity(ACCOUNT_RECORD_SIZE);
        rec.encode(&mut buf);
        buf
    }

    fn root_of(records: &[Vec<u8>], anchor: u64) -> Hash32 {
        let mut s = Snapshot::new(anchor);
        for r in records {
            s.add_record(FastSyncTableId::Account, r.clone()).unwrap();
        }
        let t = s.build_tables().unwrap();
        compute_state_root(&t.nodes.root(), &t.candidates.root(), &t.accounts.root())
    }

    fn chunk(idx: u32, total: u32, recs: Vec<Vec<u8>>) -> FastSyncChunk {
        FastSyncChunk {
            chunk_index: idx,
            total_chunks: total,
            table_id: FastSyncTableId::Account,
            records: recs,
        }
    }

    #[test]
    fn single_chunk_verifies_and_builds() {
        let recs = vec![acct_bytes(0x11), acct_bytes(0x22)];
        let root = root_of(&recs, 75_850);
        let mut c = FastSyncClient::new(75_850, root);
        assert_eq!(
            c.accept_chunk(chunk(0, 1, recs)).unwrap(),
            AcceptOutcome::Complete
        );
        let tables = c.finalize().expect("finalize");
        assert_eq!(tables.accounts.len(), 2);
    }

    #[test]
    fn multi_chunk_out_of_order_verifies() {
        let r0 = acct_bytes(0x01);
        let r1 = acct_bytes(0x02);
        let r2 = acct_bytes(0x03);
        let root = root_of(&[r0.clone(), r1.clone(), r2.clone()], 9);
        let mut c = FastSyncClient::new(9, root);
        assert!(matches!(
            c.accept_chunk(chunk(2, 3, vec![r2])).unwrap(),
            AcceptOutcome::Progress { .. }
        ));
        assert!(matches!(
            c.accept_chunk(chunk(0, 3, vec![r0])).unwrap(),
            AcceptOutcome::Progress { .. }
        ));
        assert_eq!(
            c.accept_chunk(chunk(1, 3, vec![r1])).unwrap(),
            AcceptOutcome::Complete
        );
        assert_eq!(c.finalize().expect("finalize").accounts.len(), 3);
    }

    #[test]
    fn tampered_record_fails_state_root() {
        let recs = vec![acct_bytes(0x11), acct_bytes(0x22)];
        let root = root_of(&recs, 1);
        let mut bad = recs.clone();
        bad[0][0] ^= 0xFF; // flip a byte → different account_id → different root
        let mut c = FastSyncClient::new(1, root);
        c.accept_chunk(chunk(0, 1, bad)).unwrap();
        let err = c.finalize().err().unwrap();
        assert!(matches!(
            err,
            FastSyncClientError::Verify(SnapshotError::StateRootMismatch { .. })
        ));
    }

    #[test]
    fn duplicate_chunk_rejected() {
        let root = root_of(&[acct_bytes(1)], 0);
        let mut c = FastSyncClient::new(0, root);
        c.accept_chunk(chunk(0, 2, vec![acct_bytes(1)])).unwrap();
        let err = c
            .accept_chunk(chunk(0, 2, vec![acct_bytes(9)]))
            .unwrap_err();
        assert!(matches!(
            err,
            FastSyncClientError::DuplicateChunk { index: 0 }
        ));
    }

    #[test]
    fn total_chunks_mismatch_rejected() {
        let mut c = FastSyncClient::new(0, [0u8; 32]);
        c.accept_chunk(chunk(0, 3, vec![acct_bytes(1)])).unwrap();
        let err = c
            .accept_chunk(chunk(1, 4, vec![acct_bytes(2)]))
            .unwrap_err();
        assert!(matches!(
            err,
            FastSyncClientError::TotalChunksMismatch {
                expected: 3,
                actual: 4
            }
        ));
    }

    #[test]
    fn chunk_index_out_of_range_rejected() {
        let mut c = FastSyncClient::new(0, [0u8; 32]);
        let err = c
            .accept_chunk(chunk(5, 3, vec![acct_bytes(1)]))
            .unwrap_err();
        assert!(matches!(
            err,
            FastSyncClientError::ChunkIndexOutOfRange { index: 5, total: 3 }
        ));
    }

    #[test]
    fn incomplete_finalize_rejected() {
        let root = root_of(&[acct_bytes(1)], 0);
        let mut c = FastSyncClient::new(0, root);
        c.accept_chunk(chunk(0, 2, vec![acct_bytes(1)])).unwrap();
        let err = c.finalize().err().unwrap();
        assert!(matches!(
            err,
            FastSyncClientError::Incomplete {
                received: 1,
                total: 2
            }
        ));
    }
}
