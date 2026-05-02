// Manual Validation Gate scenario 7: proposal exchange e2e test.
//
// Spec ROADMAP «Критерий закрытия M6: 2 узла на разных machines обмениваются
// proposals через network».
//
// Этот test использует synthetic Proposal envelope (msg_type=0x22) для
// проверки full transport chain. Реальный Proposal construction от
// mt-consensus apply_proposal — separate integration с M8 montana-node
// binary (cross-process / cross-machine pairing).

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
async fn proposal_envelope_round_trip() {
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

    let server_addr = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = server.select_next_some().await {
            break address;
        }
    };
    let server_peer_id = *server.local_peer_id();
    let server_dial: Multiaddr = format!("{server_addr}/p2p/{server_peer_id}")
        .parse()
        .unwrap();
    client.dial(server_dial).expect("client dial");

    // Synthetic Proposal payload — структура согласно spec section "Proposal":
    // header bytes (упрощённо для transport-level e2e). Реальный proposal
    // construction — mt-consensus integration.
    let synthetic_proposal_payload: Vec<u8> = (0..512).map(|i| (i & 0xFF) as u8).collect();
    let proposal_request = ProtocolMessage::new(
        MsgType::Proposal,
        0x1234_5678_9ABC_DEF0,
        synthetic_proposal_payload.clone(),
    );
    let ack_response = ProtocolMessage::new(MsgType::Pong, 0x1234_5678_9ABC_DEF0, vec![]);

    let mut req_id = None;
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
                    assert_eq!(req.msg_type, MsgType::Proposal);
                    assert_eq!(req.payload.len(), 512);
                    assert_eq!(req.payload, synthetic_proposal_payload);
                    server.behaviour_mut().request_response.send_response(channel, ack_response.clone()).expect("send response");
                }
            }
            ev = client.select_next_some() => {
                match ev {
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        let id = client.behaviour_mut().request_response.send_request(&peer_id, proposal_request.clone());
                        req_id = Some(id);
                    }
                    SwarmEvent::Behaviour(MontanaBehaviourEvent::RequestResponse(
                        RrEvent::Message {
                            message: RrMessage::Response { request_id: rid, response },
                            ..
                        },
                    )) => {
                        assert_eq!(Some(rid), req_id);
                        assert_eq!(response, ack_response);
                        return;
                    }
                    _ => {}
                }
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn large_payload_near_max_limit() {
    // Boundary test: 512 KB payload (half of MAX_PROTOCOL_PAYLOAD_BYTES = 1 MiB)
    // exercises wire-format roundtrip без срабатывания backpressure reject.
    let listen: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
    let mut server = build_swarm(
        MontanaBehaviour::new(),
        &NetworkConfig {
            listen_addrs: vec![listen],
            max_inbound: 13,
            max_outbound: 24,
        },
    )
    .expect("server swarm");
    let mut client = build_swarm(
        MontanaBehaviour::new(),
        &NetworkConfig {
            listen_addrs: vec![],
            max_inbound: 13,
            max_outbound: 24,
        },
    )
    .expect("client swarm");

    let server_addr = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = server.select_next_some().await {
            break address;
        }
    };
    let server_peer_id = *server.local_peer_id();
    let dial: Multiaddr = format!("{server_addr}/p2p/{server_peer_id}")
        .parse()
        .unwrap();
    client.dial(dial).unwrap();

    let big = vec![0xCD; 512 * 1024];
    let req = ProtocolMessage::new(MsgType::FastSyncResponse, 1, big.clone());
    let resp = ProtocolMessage::new(MsgType::Pong, 1, vec![]);
    let mut id = None;
    let timeout = tokio::time::sleep(Duration::from_secs(20));
    tokio::pin!(timeout);
    loop {
        tokio::select! {
            _ = &mut timeout => panic!("timeout"),
            ev = server.select_next_some() => {
                if let SwarmEvent::Behaviour(MontanaBehaviourEvent::RequestResponse(
                    RrEvent::Message { message: RrMessage::Request { request, channel, .. }, .. }
                )) = ev {
                    assert_eq!(request.payload.len(), 512 * 1024);
                    server.behaviour_mut().request_response.send_response(channel, resp.clone()).unwrap();
                }
            }
            ev = client.select_next_some() => {
                match ev {
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        id = Some(client.behaviour_mut().request_response.send_request(&peer_id, req.clone()));
                    }
                    SwarmEvent::Behaviour(MontanaBehaviourEvent::RequestResponse(
                        RrEvent::Message { message: RrMessage::Response { request_id: r, response }, .. }
                    )) => {
                        assert_eq!(Some(r), id);
                        assert_eq!(response, resp);
                        return;
                    }
                    _ => {}
                }
            }
        }
    }
}
