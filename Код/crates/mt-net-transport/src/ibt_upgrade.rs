// IBT proof exchange как custom protocol upgrade поверх Noise.
//
// Spec section "Connection lifecycle" Step 4-5:
//   После Noise key agreement клиент отправляет IBT proof,
//   сервер verify + access level determination
//   (node / candidate / account) per Identity-Bound Tunnel.

use mt_crypto::PublicKey;
use mt_net::ibt::{ibt_online_verify, IbtError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IbtAccessLevel {
    Node,
    Candidate,
    Account,
}

#[derive(Debug, Clone)]
pub struct IbtConfig {
    pub server_node_id: [u8; 32],
    pub current_window: u64,
}

pub fn classify_proof(
    config: &IbtConfig,
    client_pk: &PublicKey,
    proof: &mt_crypto::Signature,
    is_in_node_table: bool,
    is_in_candidate_pool: bool,
    is_in_account_table: bool,
) -> Result<IbtAccessLevel, IbtError> {
    ibt_online_verify(
        client_pk,
        &config.server_node_id,
        config.current_window,
        proof,
    )?;
    if is_in_node_table {
        Ok(IbtAccessLevel::Node)
    } else if is_in_candidate_pool {
        Ok(IbtAccessLevel::Candidate)
    } else if is_in_account_table {
        Ok(IbtAccessLevel::Account)
    } else {
        Err(IbtError::InvalidSignature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mt_crypto::keypair;
    use mt_net::ibt::ibt_online_proof;

    #[test]
    fn classify_node_level() {
        let (pk, sk) = keypair();
        let config = IbtConfig {
            server_node_id: [0x42; 32],
            current_window: 1000,
        };
        let proof = ibt_online_proof(&sk, &config.server_node_id, 1000).unwrap();
        let level = classify_proof(&config, &pk, &proof, true, false, false).unwrap();
        assert_eq!(level, IbtAccessLevel::Node);
    }

    #[test]
    fn classify_candidate_level() {
        let (pk, sk) = keypair();
        let config = IbtConfig {
            server_node_id: [0x42; 32],
            current_window: 1000,
        };
        let proof = ibt_online_proof(&sk, &config.server_node_id, 1000).unwrap();
        let level = classify_proof(&config, &pk, &proof, false, true, false).unwrap();
        assert_eq!(level, IbtAccessLevel::Candidate);
    }

    #[test]
    fn classify_account_level() {
        let (pk, sk) = keypair();
        let config = IbtConfig {
            server_node_id: [0x42; 32],
            current_window: 1000,
        };
        let proof = ibt_online_proof(&sk, &config.server_node_id, 1000).unwrap();
        let level = classify_proof(&config, &pk, &proof, false, false, true).unwrap();
        assert_eq!(level, IbtAccessLevel::Account);
    }

    #[test]
    fn classify_unknown_rejected() {
        let (pk, sk) = keypair();
        let config = IbtConfig {
            server_node_id: [0x42; 32],
            current_window: 1000,
        };
        let proof = ibt_online_proof(&sk, &config.server_node_id, 1000).unwrap();
        let r = classify_proof(&config, &pk, &proof, false, false, false);
        assert_eq!(r, Err(IbtError::InvalidSignature));
    }

    #[test]
    fn classify_invalid_proof_rejected() {
        let (pk, _sk) = keypair();
        let (_pk2, sk2) = keypair();
        let config = IbtConfig {
            server_node_id: [0x42; 32],
            current_window: 1000,
        };
        // proof signed by sk2, classified против pk → fail
        let proof = ibt_online_proof(&sk2, &config.server_node_id, 1000).unwrap();
        let r = classify_proof(&config, &pk, &proof, true, false, false);
        assert_eq!(r, Err(IbtError::InvalidSignature));
    }
}
