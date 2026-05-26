// End-to-end integration test of the egress control plane.
// spec: Montana Egress v1.0.0 (Session establishment + Control messages).
//
// Wires the client encode path to the exit decode + ExitSession handling in
// one process: this exercises the full control flow (Auth -> Open -> Data ->
// Close) over the byte codec and the exit state machine. Socket I/O and the
// two Noise_PQ XX sessions are transport glue verified separately by the
// Network layer; this test fixes the application control plane.

use mt_egress::{
    EgressAddr, EgressControl, ExitPolicy, ExitSession, OpenOutcome,
};

/// Drive one control message client -> wire -> exit, returning the exit's
/// optional EgressOpenAck (re-encoded and re-decoded to prove the round trip).
fn deliver(session: &mut ExitSession, msg: &EgressControl) -> Option<EgressControl> {
    let mut wire = Vec::new();
    msg.encode(&mut wire);
    let decoded = EgressControl::decode(&wire).expect("exit decodes client message");
    match decoded {
        EgressControl::Auth { .. } => {
            // The caller verifies the IBT proof against exit_node_id before this;
            // here we model a successful verification.
            session.authenticate();
            None
        },
        EgressControl::Open { stream_id, addr, dest_port, .. } => {
            let outcome = session
                .handle_open(stream_id, &addr, dest_port)
                .expect("authed open");
            let ack = EgressControl::OpenAck { stream_id, status: outcome.status() };
            // round-trip the ack back through the wire to the client
            let mut ackbuf = Vec::new();
            ack.encode(&mut ackbuf);
            Some(EgressControl::decode(&ackbuf).expect("client decodes ack"))
        },
        EgressControl::Data { stream_id, .. } => {
            session.check_data(stream_id).expect("data on open stream");
            None
        },
        EgressControl::Close { stream_id, .. } => {
            session.handle_close(stream_id).expect("close open stream");
            None
        },
        EgressControl::Keepalive => None,
        EgressControl::OpenAck { .. } => None,
    }
}

#[test]
fn full_egress_control_flow() {
    let mut exit = ExitSession::new(ExitPolicy::default_allow());

    // 1. Auth
    deliver(&mut exit, &EgressControl::Auth { proof: vec![0xAB; 64] });
    assert!(exit.is_authenticated());

    // 2. Open a TCP stream to a hostname:443
    let open = EgressControl::Open {
        stream_id: 1,
        protocol: 0,
        addr: EgressAddr::Host(b"example.com".to_vec()),
        dest_port: 443,
    };
    let ack = deliver(&mut exit, &open).expect("ack");
    match ack {
        EgressControl::OpenAck { stream_id, status } => {
            assert_eq!(stream_id, 1);
            assert_eq!(status, OpenOutcome::Open.status());
        },
        _ => panic!("expected OpenAck"),
    }
    assert!(exit.has_stream(1));

    // 3. Data on the open stream
    deliver(&mut exit, &EgressControl::Data { stream_id: 1, payload: vec![1, 2, 3] });

    // 4. Close
    deliver(&mut exit, &EgressControl::Close { stream_id: 1, reason: 0 });
    assert!(!exit.has_stream(1));
    assert_eq!(exit.open_stream_count(), 0);
}

#[test]
fn open_before_auth_is_rejected_end_to_end() {
    let mut exit = ExitSession::new(ExitPolicy::default_allow());
    let open = EgressControl::Open {
        stream_id: 1,
        protocol: 0,
        addr: EgressAddr::V4([1, 1, 1, 1]),
        dest_port: 443,
    };
    let mut wire = Vec::new();
    open.encode(&mut wire);
    let decoded = EgressControl::decode(&wire).unwrap();
    if let EgressControl::Open { stream_id, addr, dest_port, .. } = decoded {
        // exit must refuse Open before a verified Auth (session closes per spec)
        assert!(exit.handle_open(stream_id, &addr, dest_port).is_err());
    } else {
        panic!("decode");
    }
    assert!(!exit.is_authenticated());
}

#[test]
fn policy_deny_blocks_egress_end_to_end() {
    // default-deny exit, only 443 allowed
    let mut exit = ExitSession::new(ExitPolicy::default_deny().with_exception(443));
    exit.authenticate();

    // port 25 refused
    let blocked = EgressControl::Open {
        stream_id: 1,
        protocol: 0,
        addr: EgressAddr::V4([9, 9, 9, 9]),
        dest_port: 25,
    };
    let ack = deliver(&mut exit, &blocked).expect("ack");
    if let EgressControl::OpenAck { status, .. } = ack {
        assert_eq!(status, OpenOutcome::RefusedByPolicy.status());
    } else {
        panic!("expected ack");
    }
    assert!(!exit.has_stream(1));

    // port 443 allowed
    let allowed = EgressControl::Open {
        stream_id: 2,
        protocol: 0,
        addr: EgressAddr::V4([9, 9, 9, 9]),
        dest_port: 443,
    };
    let ack2 = deliver(&mut exit, &allowed).expect("ack");
    if let EgressControl::OpenAck { status, .. } = ack2 {
        assert_eq!(status, OpenOutcome::Open.status());
    } else {
        panic!("expected ack");
    }
    assert!(exit.has_stream(2));
}
