# Transport-layer identifier leakage — MTProto vs Montana Noise_PQ XX

**Date:** 2026-05-22.
**Scope:** comparison of two transport designs in adversarial-passive-observer model.
**Audience:** cryptography@metzdowd.com list and the independent reviewer reading the v1.0.0 mainnet tag.

The published claim about the Telegram MTProto wire format is that every MTProto session sends `auth_key_id` — a long-lived 64-bit identifier bound to the client's authorization key — in cleartext at the start of every encrypted message over plain TCP. A passive observer with read-only access to the network path (an ISP, a hotel Wi-Fi operator, a mobile carrier, a transit provider, a state-level adversary on the route) sees `auth_key_id` on the wire and can correlate sessions across IP changes, VPN switches, network handovers, location changes, and application restarts. The encrypted message body protects content, but the leak is structurally below the content layer. Secret Chats do not fix this — the leak is on the outer envelope, not on the message payload.

The exploitation pattern is retrospective correlation. A user connects from many networks over months. Then the user connects once from a hotel Wi-Fi under a real name, or from a corporate network with known mapping, or from a mobile network with a known SIM. That single moment of attribution makes every historical log entry with the same `auth_key_id` retroactively attributable.

This document describes how Montana's production transport — Noise_PQ XX over TCP — does not have this property.

---

## 1. Wire format of a Montana TCP connection — byte by byte

A passive observer on the path between two Montana peers sees the following sequence on a fresh TCP connection.

| Phase | Bytes visible | Long-term identifier visible | Notes |
|-------|---------------|------------------------------|-------|
| TCP three-way handshake | SYN / SYN-ACK / ACK | none | standard for any TCP |
| libp2p multistream-select | `/multistream/1.0.0\n` + `/montana/noise-pq-xx/1.0.0\n` (length-prefixed) | none | protocol name only — same string on every Montana connection, network-wide marker, not per-client |
| Noise_PQ XX msg1 | 1184 bytes, fully opaque to the observer | none | initiator-ephemeral ML-KEM-768 public key + length prefix — ephemeral keypair generated fresh per connection |
| Noise_PQ XX msg2 | 7533 bytes, fully opaque | none | responder-ephemeral pubkey + responder-static ML-DSA-65 pubkey encrypted under the ephemeral shared secret + ML-DSA-65 signature over transcript, encrypted |
| Noise_PQ XX msg3 | 6349 bytes, fully opaque | none | initiator-static ML-DSA-65 pubkey encrypted under the established session key + ML-DSA-65 signature over transcript, encrypted |
| Established session | ChaCha20-Poly1305 framed Yamux streams | none | session keys derived from ephemeral KEM exchange; no static identifier on the wire |

Sources: [`Code/crates/mt-noise-pq/src/xx_handshake.rs`](../Code/crates/mt-noise-pq/src/xx_handshake.rs) (state machine + transcript binding), [`Montana Network v1.1.0.md`](../Montana%20Network%20v1.1.0.md) §«Noise_PQ XX wire format» (normative byte layout).

## 2. What this means for the passive-observer threat model

The passive observer on the path can determine:

1. **A TCP connection occurred** between two IPv4/IPv6 endpoints at a known time and on a known port. This is a property of TCP, not of any transport layer above it.
2. **The protocol is Montana** — the multistream-select preamble names `/montana/noise-pq-xx/1.0.0` in plaintext. This is a network-wide marker (the same string appears in every Montana connection), not a per-client identifier.
3. **The size and timing of subsequent ciphertext frames.** Standard traffic-analysis surface present in any encrypted protocol.

The passive observer cannot determine:

1. **The initiator's long-term identity.** The initiator's static ML-DSA-65 public key — the only stable identifier the peer ever uses across sessions — is sent in msg3, encrypted under the session key derived from the ephemeral ML-KEM-768 exchange in msg1 + msg2. No part of msg3's plaintext is visible to the observer.
2. **The responder's long-term identity, beyond the fact that the responder owns the IP being dialed.** The responder's static ML-DSA-65 public key is sent in msg2, encrypted under the ephemeral shared secret. The IP itself is observable (it's the destination of the TCP SYN), but the peer identity is not bound to a stable byte-string on the wire.
3. **Cross-session correlation by long-term identifier.** Every Montana TCP connection runs a fresh ephemeral ML-KEM-768 keypair on both sides. Two consecutive connections by the same client to the same server produce two completely different ciphertext streams, with no shared bytes that link them as belonging to the same client identity. The retroactive-correlation attack described above is structurally not reachable through the Montana transport.

The contract is stronger than "encrypted from byte N." It is "no long-term identifier exists as a byte-string on the wire."

## 3. Where this property comes from

The property is a consequence of three design decisions in Noise_PQ XX.

**3.1 Ephemeral key encapsulation on both sides.** Both initiator and responder generate a fresh ML-KEM-768 keypair per connection. Neither side has a stable KEM keypair that an observer could fingerprint across sessions.

**3.2 Identity sent post-decapsulation.** The static ML-DSA-65 public key — the only stable per-peer identifier — is sent only after the session key has been derived from the ephemeral KEM exchange. By the time the observer sees any byte that could carry identity information, that byte is already encrypted under the session key.

**3.3 Identity authenticated by signature, not by inclusion in plaintext.** The handshake binds the responder identity in msg2 by signing the transcript hash with the responder's ML-DSA-65 secret key. The initiator binds its identity the same way in msg3. Identity is proved cryptographically without being placed in plaintext at the start of the connection.

The MTProto design places `auth_key_id` at the start of every message in plaintext because, in the original protocol, the server needs to look up which authorization key to use before it can decrypt anything. Noise_PQ XX does not have this constraint: the session key is derived from the in-handshake KEM exchange, and identity verification happens entirely inside the encrypted half of the handshake.

## 4. Out of scope for this property

The Noise_PQ XX property protects against the passive-observer threat described above. It does not protect against:

- **Active man-in-the-middle that controls the network path** — the property protects identity binding, not endpoint reachability. An active MITM that controls the TCP stream can drop, delay, or replay packets, just like for any transport.
- **TCP four-tuple visibility** — `(src_ip, src_port, dst_ip, dst_port)` is observable to any on-path party. If the observer knows the dst_ip corresponds to a Montana mainnet node (the three Genesis IPs are pinned in the public manifest and the Reality VPN front IPs are likewise public), the observer learns that the user contacted "a Montana node." This is identical to what an observer learns when a user contacts any other public-IP server — including Tor relays, Signal, Telegram, etc. — and is treated by the rest of the Montana stack via the VPN frontends and Reality-style obfuscation.
- **Traffic-pattern fingerprinting** — fixed-size handshake bytes (1184 / 7533 / 6349) make a Montana handshake recognizable as Montana. The countermeasure is uniform framing on the established session (Network spec §«Uniform framing for DPI obfuscation»). Frame sizes after the handshake are uniform regardless of payload, removing per-message size fingerprints.

## 5. Independent verification

Anyone with read access to the source tree at the v1.0.0 mainnet tag can reproduce the analysis above.

1. Open [`Code/crates/mt-noise-pq/src/xx_handshake.rs`](../Code/crates/mt-noise-pq/src/xx_handshake.rs). Confirm that the static ML-DSA-65 keypair is never serialized in plaintext.
2. Run `cargo test -p mt-noise-pq --release` and read the KAT vectors: msg1 contains the ephemeral KEM pubkey only; msg2 and msg3 are full ciphertext from the first byte after the length prefix.
3. Capture a TCP session between two Genesis peers with `tcpdump` on either side; the post-multistream bytes are uniformly random ciphertext.

The maintainer commits to acknowledge any finding to the contrary within seven days of submission via GitHub issues at https://github.com/efir369999/Montana/issues with label `mainnet-v1.0.0`, or via plaintext reply on the Metzdowd Cryptography List referencing the v1.0.0 tag SHA `a260ba9005c48763fadad0de5797bae48989215e`.
