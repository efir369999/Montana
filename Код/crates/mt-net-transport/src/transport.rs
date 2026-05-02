// Transport build helper для libp2p Swarm с TLS 1.3 + Noise + Yamux.
//
// Spec section "Connection lifecycle" Step 1-3:
//   TCP SYN → TLS 1.3 handshake → Noise key agreement
//
// Step 4-5 (IBT proof + access level) — см. ibt_upgrade::classify_proof.
// Step 6 (ProtocolMessage exchange) — caller responsibility поверх данного swarm.

use libp2p::{noise, tcp, tls, yamux, Multiaddr, Swarm, SwarmBuilder};

use crate::error::TransportError;

pub struct NetworkConfig {
    pub listen_addrs: Vec<Multiaddr>,
    pub max_inbound: u32,
    pub max_outbound: u32,
}

impl NetworkConfig {
    pub fn defaults() -> Self {
        // Defaults per spec Network section + critic-fix P-S3
        // (operator-configurable, не Genesis Decree binding).
        NetworkConfig {
            listen_addrs: Vec::new(),
            max_inbound: 13,
            max_outbound: 24,
        }
    }
}

pub fn build_swarm<B>(behaviour: B, config: &NetworkConfig) -> Result<Swarm<B>, TransportError>
where
    B: libp2p::swarm::NetworkBehaviour + Send,
{
    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            // TLS 1.3 (rustls) + Noise — двойное шифрование per spec
            // Transport Obfuscation; TLS hides traffic от passive observer
            // (DPI), Noise authenticates peer identity.
            (tls::Config::new, noise::Config::new),
            yamux::Config::default,
        )
        .map_err(|e| TransportError::Setup(format!("transport upgrade: {e}")))?
        .with_behaviour(|_| behaviour)
        .map_err(|e| TransportError::Setup(format!("behaviour: {e}")))?
        .build();

    for addr in &config.listen_addrs {
        swarm
            .listen_on(addr.clone())
            .map_err(|e| TransportError::Setup(format!("listen_on {addr}: {e}")))?;
    }
    Ok(swarm)
}

/// Сборка swarm-а с фиксированным libp2p Keypair (production-режим).
///
/// В отличие от `build_swarm` (генерит fresh keypair каждый запуск, для тестов),
/// эта функция принимает готовый Ed25519 keypair, derived from operator's
/// identity. PeerId узла стабилен между перезапусками — обязательно для
/// genesis-cohort peer pinning через `GenesisManifest`.
pub fn build_swarm_with_keypair<B>(
    keypair: libp2p::identity::Keypair,
    behaviour: B,
    config: &NetworkConfig,
) -> Result<Swarm<B>, TransportError>
where
    B: libp2p::swarm::NetworkBehaviour + Send,
{
    let mut swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            (tls::Config::new, noise::Config::new),
            yamux::Config::default,
        )
        .map_err(|e| TransportError::Setup(format!("transport upgrade: {e}")))?
        .with_behaviour(|_| behaviour)
        .map_err(|e| TransportError::Setup(format!("behaviour: {e}")))?
        .build();

    for addr in &config.listen_addrs {
        swarm
            .listen_on(addr.clone())
            .map_err(|e| TransportError::Setup(format!("listen_on {addr}: {e}")))?;
    }
    Ok(swarm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let c = NetworkConfig::defaults();
        assert_eq!(c.max_outbound, 24);
        assert_eq!(c.max_inbound, 13);
    }
}
