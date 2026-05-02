// spec, раздел "Сетевой уровень → Cross-machine peering" (M8)
//
// Async event loop поверх libp2p Swarm с MontanaBehaviour из mt-net-transport.
// Узел listens на multiaddr, dial-ит peers из GenesisManifest, и через
// request-response обменивается ProtocolMessage envelope-ами.
//
// **Спецификационная роль:** transport уровень. Consensus state sync (sharing
// AccountTable / NodeTable / ProposalHeader через сеть) — отдельный модуль
// M9+. Этот модуль обеспечивает только:
//   1. Стабильную TCP+TLS+Noise связь между genesis-cohort peers
//   2. PeerId pinning (verify peer's libp2p identity == genesis manifest)
//   3. Periodic Ping → Pong heartbeat (proves liveness через journal logs)
//
// Когда узлы готовы к state sync (M9), Behaviour расширяется gossipsub
// или request-response handler для NodeRegistration, BundledConfirmation,
// ProposalHeader.

use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use futures::StreamExt;
use libp2p::request_response::{Event as RrEvent, Message as RrMessage};
use libp2p::swarm::SwarmEvent;
use libp2p::{Multiaddr, PeerId, Swarm};
use mt_genesis::GenesisManifest;
use mt_net::{MsgType, ProtocolMessage};
use mt_net_transport::{
    build_swarm_with_keypair, MontanaBehaviour, MontanaBehaviourEvent, NetworkConfig,
};

use crate::identity::NodeError;

/// Период heartbeat Ping → каждый узел шлёт Ping каждые N секунд всем connected peers.
/// Production: 30 сек = compromise между фон-трафик минимизацией и timeliness.
const HEARTBEAT_PERIOD: Duration = Duration::from_secs(30);

/// Запустить network event loop в текущем tokio runtime. Блокирует tokio task
/// до Ctrl-C (либо ошибки swarm). При корректной остановке — graceful close
/// всех connections.
pub async fn run_network_loop(
    local_keypair: libp2p::identity::Keypair,
    local_peer_id: PeerId,
    manifest: GenesisManifest,
    listen_addr: Multiaddr,
) -> Result<(), NodeError> {
    let cfg = NetworkConfig {
        listen_addrs: vec![listen_addr.clone()],
        max_inbound: 13,
        max_outbound: 24,
    };

    let mut swarm: Swarm<MontanaBehaviour> =
        build_swarm_with_keypair(local_keypair, MontanaBehaviour::new(), &cfg)
            .map_err(|e| NodeError::Network(format!("build swarm: {e}")))?;

    eprintln!(
        "[network] local_peer_id={local_peer_id} listen={listen_addr} \
         peers_to_dial={cnt}",
        cnt = manifest.peers.len() - 1 // вычитаем self
    );

    // Dial peers из manifest (пропуская self)
    let mut dialed: HashMap<PeerId, String> = HashMap::new();
    for peer in &manifest.peers {
        let peer_id_parsed = PeerId::from_str(&peer.peer_id)
            .map_err(|e| NodeError::Network(format!("invalid peer_id {}: {e}", peer.peer_id)))?;
        if peer_id_parsed == local_peer_id {
            continue; // self
        }
        let multiaddr_parsed = Multiaddr::from_str(&peer.multiaddr).map_err(|e| {
            NodeError::Network(format!("invalid multiaddr {}: {e}", peer.multiaddr))
        })?;
        // Compose `<multiaddr>/p2p/<peer_id>` для PeerId pinning по libp2p
        let dial_target: Multiaddr = format!("{}/p2p/{}", multiaddr_parsed, peer_id_parsed)
            .parse()
            .map_err(|e: libp2p::multiaddr::Error| {
                NodeError::Network(format!("compose multiaddr: {e}"))
            })?;
        eprintln!("[network] dialing {} ({})", peer.label, dial_target);
        swarm
            .dial(dial_target.clone())
            .map_err(|e| NodeError::Network(format!("dial {}: {e}", peer.label)))?;
        dialed.insert(peer_id_parsed, peer.label.clone());
    }

    let mut heartbeat = tokio::time::interval(HEARTBEAT_PERIOD);
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut request_id_counter: u64 = 0;
    let mut connected_peers: HashMap<PeerId, String> = HashMap::new();

    loop {
        tokio::select! {
            ev = swarm.select_next_some() => {
                match ev {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        eprintln!("[network] listening on {address}/p2p/{local_peer_id}");
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                        let label = dialed
                            .get(&peer_id)
                            .cloned()
                            .unwrap_or_else(|| "unknown".to_string());
                        eprintln!(
                            "[network] CONNECTION ESTABLISHED peer={peer_id} label={label} \
                             remote={remote}",
                            remote = endpoint.get_remote_address()
                        );
                        connected_peers.insert(peer_id, label);
                    }
                    SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                        let label = connected_peers.remove(&peer_id).unwrap_or_default();
                        eprintln!(
                            "[network] connection closed peer={peer_id} label={label} \
                             cause={cause:?}"
                        );
                    }
                    SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                        eprintln!(
                            "[network] outgoing connection ERROR peer={peer_id:?} error={error}"
                        );
                    }
                    SwarmEvent::Behaviour(MontanaBehaviourEvent::RequestResponse(
                        RrEvent::Message {
                            peer: _,
                            message: RrMessage::Request { request, channel, .. },
                            ..
                        },
                    )) => {
                        // Reply Ping → Pong; другие сообщения консенсуса — defer M9
                        if request.msg_type == MsgType::Ping {
                            let pong = ProtocolMessage::new(MsgType::Pong, request.request_id, Vec::new());
                            if let Err(e) = swarm
                                .behaviour_mut()
                                .request_response
                                .send_response(channel, pong)
                            {
                                eprintln!("[network] send Pong failed: {e:?}");
                            }
                        }
                    }
                    SwarmEvent::Behaviour(MontanaBehaviourEvent::RequestResponse(
                        RrEvent::Message {
                            peer,
                            message: RrMessage::Response { request_id, response },
                            ..
                        },
                    )) => {
                        if response.msg_type == MsgType::Pong {
                            eprintln!(
                                "[network] heartbeat OK peer={peer} request_id={request_id}"
                            );
                        }
                    }
                    _ => {}
                }
            }
            _ = heartbeat.tick() => {
                // Bcast Ping всем connected peers
                let peers: Vec<PeerId> = connected_peers.keys().copied().collect();
                for peer_id in peers {
                    request_id_counter = request_id_counter.wrapping_add(1);
                    let ping = ProtocolMessage::new(
                        MsgType::Ping,
                        request_id_counter,
                        Vec::new(),
                    );
                    swarm.behaviour_mut().request_response.send_request(&peer_id, ping);
                }
            }
            _ = tokio::signal::ctrl_c() => {
                eprintln!("[network] Ctrl-C, выход");
                return Ok(());
            }
        }
    }
}
