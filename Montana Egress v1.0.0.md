# Montana — Egress Layer Specification

**Version:** 1.0.0 (2026-05-26)

**Layer:** Application — sits above the Network layer. Consumes the mesh; defines no consensus state and contributes nothing to any state root.

---

## Introduction

The Egress layer carries a participant's ordinary internet traffic out to the public internet (the clearnet) through a Montana node located in a chosen jurisdiction. A client selects an exit country manually, or delegates the choice to an automatic selector. The transport that carries the client to the mesh, survives per-operator filtering, and re-routes around blocked entry points is supplied entirely by the Network layer; the Egress layer adds only the egress semantics on top.

The Egress layer is an application service offered by node operators who opt in. It is not a consensus mechanism, not a protocol guarantee, and not a protocol-level anonymity system. The consensus state machine is unaware of egress; removing the entire Egress layer changes no state root and halts no clock. This separation is deliberate: anti-censorship reachability is a Network-layer property shared by all traffic (consensus, gossip, messenger, egress), whereas egress is one application built on that foundation.

### Relationship to other specifications

| Concern | Owning specification |
|---------|----------------------|
| Post-quantum transport handshake (Noise_PQ XX), uniform framing | Network — Transport obfuscation |
| Identity-Bound Tunnel (IBT) access levels | Network — Transport obfuscation → IBT |
| Reachable entry discovery, transport profile ladder T0–T4 | Network — Reachability sensing, Transport profile ladder |
| Multi-hop transit | Network — NAT Traversal → Circuit Relay v2 |
| Account identity, ML-DSA-65 / ML-KEM-768 sizes | Protocol — Cryptographic primitives |

The Egress layer references these; it redefines none of them.

### Regulatory stance

The Egress layer is an opt-in service of an individual node operator, not a property the protocol guarantees to all participants. The consensus layer neither requires, rewards, nor records egress. An operator enabling egress accepts the operational and jurisdictional consequences of forwarding third-party traffic; an operator who does not enable it participates fully in consensus and messaging regardless. This keeps general-purpose egress out of the protocol's regulatory surface.

---

## Roles

| Role | Definition |
|------|------------|
| **Egress client** | A participant authenticated at IBT level 3 (account keypair) requesting clearnet egress. |
| **Entry node** | Any reachable mesh node the client connects to first. Selected by reachability sensing; carries ciphertext only. |
| **Relay** | A mesh node providing Circuit Relay v2 transit between entry and exit when they are not directly connected. Carries ciphertext only. Entry and relay MAY be the same node. |
| **Exit node** | A mesh node that has opted into clearnet egress, advertises a jurisdiction, and forwards the client's streams to the public internet. |

The entry, relay, and exit are ordinary Montana nodes. A node MAY hold several roles simultaneously. The exit role is configuration-gated and disabled by default.

---

## Architecture

The client establishes two nested Noise_PQ XX sessions:

```
              outer session (Network transport, profile T0..T4)
   client ───────────────────────────────────────────────▶ entry node
                                                              │ Circuit Relay v2
                                                              ▼
   client ════════ inner end-to-end Noise_PQ XX ══════════▶ exit node
              (entry and relay carry ciphertext only)         │ clearnet
                                                              ▼
                                                          destination host
```

1. The **outer session** is an ordinary mesh transport session (Network layer). It gets the client onto the mesh through a reachable entry, with DPI resistance and the transport profile ladder. The entry authenticates the client by IBT.
2. The **inner session** is an end-to-end Noise_PQ XX session between client and exit, tunnelled through the entry (and any relay) by Circuit Relay v2. Entry and relay observe only the AEAD ciphertext of the inner session: they learn the participants' addresses, never the egress control messages or the egress payload — the same trust model as Network-layer relay.

The exit terminates the inner session, authenticates the client by IBT level 3, and forwards the client's streams to the clearnet.

---

## Egress directory

Exit nodes advertise availability in an egress directory. The directory is advisory transport-layer metadata, propagated as a PeerRecord extension over peer exchange, under the existing peer-exchange rate limit (at most one per `τ₁` per peer). It is bounded, ephemeral, and forms no consensus state.

```
EgressDirectoryEntry:
  exit_node_id      32B   node_id of the exit (verifiable by IBT)
  country_code       2B   ISO-3166-1 alpha-2, operator-declared
  capacity_class     1B   0 = best-effort, 1 = standard, 2 = high
  advertised_window u32   cached window_index at advertisement time
```

**Invariants EgressDirectoryEntry:**
- `exit_node_id` is 32 B and resolves to a node present in the address manager; an entry for an unknown node is held unverified until an IBT handshake confirms the node.
- `country_code` is two ASCII letters in the ISO-3166-1 alpha-2 set; any other value drops the entry.
- `capacity_class ∈ {0, 1, 2}`; any other value drops the entry.
- `advertised_window` lies within `[known_window_index − 7 × τ₁, known_window_index]`; a staler value drops the entry, matching the mesh-IBT staleness bound.
- A node retains at most `MAX_DIRECTORY_ENTRIES = 4096` entries, evicting the least-recently-corroborated on overflow.
- A directory entry authorizes no connection by itself; it ranks candidates only. The client confirms the exit by direct IBT handshake.

The directory keys exit selection; it never enters a state root. `country_code` is operator-declared and advisory — see Threat Model → Country attestation.

---

## Session establishment

```
1. Entry selection.
   The client selects a reachable entry by Network-layer reachability sensing
   and the transport profile ladder, then completes the outer Noise_PQ XX
   session and the IBT level-3 handshake with the entry.

2. Exit selection.
   The client resolves an exit from the egress directory:
     - manual: the first directory entry whose country_code equals the chosen
       country (the entry-hop transport profile is chosen independently by the Network layer);
     - auto:   the reachable entry ranked highest by the reachability map for
       the client's vantage_class, tie-broken by feeler round-trip latency.
   Exit selection is performed by the client, never dictated by the entry.

3. Relayed connection.
   The client opens a Circuit Relay v2 transit through the entry to the chosen
   exit_node_id (Network → NAT Traversal). If the client and exit are directly
   reachable, the relay hop is skipped.

4. Inner handshake.
   Over the relayed connection the client completes an end-to-end Noise_PQ XX
   session with the exit (protocol id /montana/noise-pq-xx/1.0.0) and an IBT
   level-3 proof. The exit verifies the proof and applies its egress policy.

5. Egress.
   The client multiplexes streams over the inner session (control messages
   below). The exit opens the corresponding clearnet sockets and relays bytes
   bidirectionally until close.
```

On loss of the entry mid-session the client re-steers to the next corroborated entry (Network → Auto-steering) and re-establishes the relayed connection to the same exit; the chosen exit, and therefore the egress IP and country, are preserved. One active entry carries a hot reserve.

---

## Control messages

All control and data messages travel over the inner end-to-end Noise_PQ XX AEAD stream (Network → Post-handshake AEAD framing). Each is a length-prefixed application message whose first byte is the message type. Streams are multiplexed over the inner session by the libp2p stream multiplexer (Network → Yamux, transport stack `TCP → Noise_PQ XX → Yamux`), which supplies per-stream flow control; each `stream_id` corresponds to one multiplexer stream, and a client bounds its per-stream send to the multiplexer window.

```
msg_type  1B   0x01 EgressAuth
               0x02 EgressOpen
               0x03 EgressOpenAck
               0x04 EgressData
               0x05 EgressClose
               0x06 EgressKeepalive
```

```
EgressAuth (0x01)  client → exit
  account_proof   variable   IBT level-3 advertisement (separator "mt-tunnel-online",
                             account keypair) as defined by the Network layer.
                             The proof binds server_node_id = exit_node_id — the inner
                             session terminates at the exit — so a proof for one exit
                             is invalid at any other node.
```

```
EgressOpen (0x02)  client → exit
  stream_id        4B   u32, client-assigned, unique within the inner session
  protocol         1B   0 = TCP, 1 = UDP
  addr_type        1B   0 = IPv4 (4B), 1 = IPv6 (16B), 2 = hostname (len-prefixed)
  dest_addr        var  per addr_type; hostname is 1B length + that many bytes
  dest_port        2B   u16 big-endian
```

```
EgressOpenAck (0x03)  exit → client
  stream_id        4B   u32, echoes EgressOpen
  status           1B   0 = open, 1 = refused by policy, 2 = unreachable,
                        3 = rate-limited
```

```
EgressData (0x04)  client ⇄ exit
  stream_id        4B   u32
  payload          var  opaque bytes for the stream (≤ 65 519, the AEAD frame
                        plaintext maximum; larger payloads are fragmented by
                        the caller across successive EgressData messages)
```

```
EgressClose (0x05)  client ⇄ exit
  stream_id        4B   u32
  reason           1B   0 = normal, 1 = error, 2 = policy
```

```
EgressKeepalive (0x06)  client ⇄ exit
  (no body) — resets the idle timer; permits uniform-framing cover on idle streams
```

**Invariants (egress control):**
- `msg_type ∈ {0x01..0x06}`; any other value closes the inner session.
- `EgressAuth` is the first message of the inner session; an `EgressOpen` before a verified `EgressAuth` closes the session.
- `stream_id` in `EgressData` / `EgressClose` references a stream opened by a prior `EgressOpen` and acknowledged `status = 0`; an unknown `stream_id` drops the message.
- `protocol ∈ {0, 1}`, `addr_type ∈ {0, 1, 2}`, `status ∈ {0, 1, 2, 3}`, `reason ∈ {0, 1, 2}`; any other value drops the message.
- The exit honours at most `MAX_STREAMS_PER_SESSION` concurrent open streams; an `EgressOpen` beyond the cap is answered `status = 3`.

---

## Exit node

An exit node is configuration-gated. When enabled, the operator declares:

- `country_code` advertised in the directory;
- an **egress policy** — the set of destination hosts/ports the exit forwards (default-allow or default-deny, operator's choice), and the set of accounts permitted (default: any IBT level-3 account);
- a **bandwidth tier** — egress is a distinct, high-bandwidth resource class, separate from the Network-layer consensus relay (which is capped at the baseline frame rate). The exit forwards egress traffic up to an operator-configured cap per session and per node.

The exit applies its policy on each `EgressOpen`: a destination outside policy is answered `status = 1`; an account outside policy fails the IBT check at step 4 and the inner session is closed. Egress is opt-in volunteer service; the Egress layer defines no protocol-level reward to the exit operator. An incentive mechanism, if introduced, is specified separately in the monetary layer and is out of scope here.

---

## Transport and reachability

The entry hop uses the Network-layer transport profile ladder (T0 direct, T1 TLS mimicry, T2 CDN, T3 pluggable transport, T4 mesh radio); the inner end-to-end session rides inside whichever profile the entry hop negotiated. Entry selection, auto-steering, and the hot reserve are Network-layer mechanisms (Reachability sensing and auto-steering); the Egress layer consumes them unchanged. The exit's own reachability from the entry is established by the same sensing, so the path is built only over corroborated-reachable hops.

---

## Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `MAX_STREAMS_PER_SESSION` | 256 | bounds exit per-session bookkeeping; matches the per-key nonce-set bound of the Network layer |
| `EGRESS_SESSION_IDLE_TIMEOUT` | `4 × τ₁` | an inner session with no `EgressData` or `EgressKeepalive` within the window is closed by the exit |
| `EGRESS_OPEN_TIMEOUT` | `τ₁ / 2` | an `EgressOpen` unanswered within the window is treated by the client as `status = 2` |
| `MAX_DIRECTORY_ENTRIES` | 4096 | bounds the advisory egress directory held by a node; least-recently-corroborated eviction on overflow |
| `EGRESS_BANDWIDTH_CAP_PER_SESSION` | operator-configured | egress high-bandwidth tier; not a consensus parameter |

`τ₁` is the Network-layer window unit. These parameters are local network-stack behaviour on the node's own clock and are outside the scope of consensus state.

---

## Threat model

The trust boundary follows directly from the two-session architecture:

| Party | Learns | Does not learn |
|-------|--------|----------------|
| **Entry / relay** | the addresses of the hops it connects; that egress-class traffic flows | egress control messages, destinations, payload (inner session is end-to-end) |
| **Exit** | the destination hosts and payload it forwards; the client's account identity (IBT) | the client's source IP (the client reaches the exit through the entry/relay) |
| **Passive observer at the client** | a transport-ladder cover session to the entry | destinations, payload, exit identity |
| **Destination host** | the exit's egress IP | the client's identity and IP |

This is the trust boundary of any honest VPN exit and is stated, not eliminated.

**Exit attribution (Sabotage / abuse).** Traffic forwarded by an exit appears on the clearnet under the exit's egress address; abusive third-party traffic is attributable to the exit operator and may trigger blocklisting of the exit address. Closure is operational, not cryptographic: the exit's egress policy (destination allow/deny, per-account rate limits) and the operator's opt-in acceptance of jurisdictional exposure. The Egress layer does not claim to remove this risk; it confines it to operators who deliberately enable the exit role.

**Auto-steering eclipse (Censor / Sybil).** A hostile entry could attempt to steer an auto-selecting client toward a logging or hostile exit. Closure: exit selection is performed by the client from the corroborated reachability map (Network → Quorum and diversity, `REACHABILITY_QUORUM = 3` distinct /16), and the client verifies the exit by its `node_id` through the end-to-end IBT handshake before any egress. A hostile entry cannot substitute an exit identity it does not hold the key for.

**Country attestation (Censor / misrepresentation).** `country_code` in the directory is operator-declared; an exit could advertise jurisdiction A while egressing from jurisdiction B. The directory value is advisory. A client requiring assurance verifies the apparent egress jurisdiction out of band — for example by retrieving its observed egress address through the established exit session and checking its geolocation — before relying on the country. The Egress layer does not bind `country_code` to a verifiable proof; it is a routing hint corroborated by directory quorum, not a guarantee.

**Correlation at a global observer.** A global passive observer correlating entry-side timing with exit-side egress is the same open metadata-correlation problem acknowledged in the Network layer; the Egress layer inherits the Network-layer mitigations (uniform framing, transport randomness) and adds no new claim.

---

## Privacy scope

The Egress layer hides the client's source address from the destination and the destination from the entry/relay. It does not hide, from the exit, the destinations the client visits — an exit is a trusted forwarding point exactly as in any VPN. A participant requiring no trusted forwarder runs an exit on their own infrastructure; the architecture permits a participant's own node to be entry, relay, and exit simultaneously, in which case no third party forwards the traffic. The Egress layer makes no anonymity claim against an adversary controlling the chosen exit.

---

## Conformance

A conforming implementation provides byte-exact encodings of `EgressDirectoryEntry` and the six control messages, the two-session establishment sequence, client-side exit selection (manual and auto), and the exit egress policy gate. Reference test vectors fix the control-message encodings and the directory-entry encoding for cross-implementation verification; the inner and outer Noise_PQ XX sessions reuse the Network-layer conformance vectors unchanged.
