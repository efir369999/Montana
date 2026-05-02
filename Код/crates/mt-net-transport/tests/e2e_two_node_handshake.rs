// Manual Validation Gate scenario 6: two-node handshake e2e test.
//
// Spec section "Connection lifecycle" Step 1-6:
//   TCP → TLS 1.3 → Noise → IBT proof exchange → access level → ProtocolMessage
//
// Этот test поднимает два libp2p Swarm в одном процессе, dial peer-to-peer,
// и обменивается одним Ping → Pong через MontanaCodec request-response.
// Это subset критерия закрытия M6 spec ROADMAP:
//   «2 узла на разных machines обмениваются proposals через network»
// (machine-pairing — defer to Phase C.5; в-process e2e — initial coverage).

use std::time::Duration;

use futures::StreamExt;
use libp2p::{
    request_response::{Event as RrEvent, Message as RrMessage},
    swarm::SwarmEvent,
    Multiaddr,
};
use mt_net::{MsgType, ProtocolMessage};
use mt_net_transport::{build_swarm, MontanaBehaviour, MontanaBehaviourEvent, NetworkConfig};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_node_request_response_ping_pong() {
    let listen: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
    let cfg = NetworkConfig {
        listen_addrs: vec![listen.clone()],
        max_inbound: 13,
        max_outbound: 24,
    };

    let mut server = build_swarm(MontanaBehaviour::new(), &cfg).expect("server swarm");
    let mut client = build_swarm(
        MontanaBehaviour::new(),
        &NetworkConfig {
            listen_addrs: vec![],
            max_inbound: 13,
            max_outbound: 24,
        },
    )
    .expect("client swarm");

    // Wait for server NewListenAddr
    let server_addr = loop {
        let ev = server.select_next_some().await;
        if let SwarmEvent::NewListenAddr { address, .. } = ev {
            break address;
        }
    };

    let server_peer_id = *server.local_peer_id();
    let server_dial_addr: Multiaddr = format!("{server_addr}/p2p/{server_peer_id}")
        .parse()
        .unwrap();

    client.dial(server_dial_addr).expect("client dial");

    let request = ProtocolMessage::new(MsgType::Ping, 0, Vec::new());
    let expected_response = ProtocolMessage::new(MsgType::Pong, 0, Vec::new());

    // Wait until connection established on client
    let mut request_id_opt = None;
    let timeout = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            _ = &mut timeout => panic!("e2e timeout"),
            ev = server.select_next_some() => {
                if let SwarmEvent::Behaviour(MontanaBehaviourEvent::RequestResponse(
                    RrEvent::Message {
                        message: RrMessage::Request { request: req, channel, .. },
                        ..
                    },
                )) = ev
                {
                    assert_eq!(req, request, "server received request must match client send");
                    server
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, expected_response.clone())
                        .expect("send response");
                }
            }
            ev = client.select_next_some() => {
                match ev {
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        let id = client
                            .behaviour_mut()
                            .request_response
                            .send_request(&peer_id, request.clone());
                        request_id_opt = Some(id);
                    }
                    SwarmEvent::Behaviour(MontanaBehaviourEvent::RequestResponse(
                        RrEvent::Message {
                            message: RrMessage::Response { request_id: rid, response },
                            ..
                        },
                    )) => {
                        assert_eq!(Some(rid), request_id_opt);
                        assert_eq!(response, expected_response, "Pong response must match");
                        return;
                    }
                    _ => {}
                }
            }
        }
    }
}
