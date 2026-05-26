# Montana — VPN Alliance Architecture

**Version:** 1.0.0 (2026-05-26)

**Layer:** Application — a federation pattern over the Egress layer. Defines no consensus state.

---

## Concept

A VPN Alliance is the voluntary federation of Montana nodes that opt into the exit role and insure one another's reachability. Each node is a city; a city opens its own egress; cities insure each other so that a client who cannot reach one city directly still reaches it through a city it can reach. The alliance is the operational expression of the Montana principle that a personal network works when everyone can join: the union of reachable entry points and country exits is the usable surface, and no single blocked address removes a country from that surface.

The alliance is a service of its member operators, not a protocol guarantee. The consensus layer neither requires nor records alliance membership. A node participates fully in consensus and messaging whether or not it joins the alliance.

---

## Membership

A node joins by enabling the exit role (Egress spec, Exit node) and registering an `EgressDirectoryEntry`:

- it advertises a jurisdiction (`country_code`) and a capacity class;
- it adopts either the **alliance universal key** (one Reality keypair shared across members so a single client subscription authenticates to every member) or its **own key** (when its port is shared with another public service, the node masquerades as its own real site);
- it accepts the operator obligations of forwarding third-party traffic (egress policy, jurisdictional exposure).

Membership is opt-in and revocable: a node leaves by deregistering and disabling the exit role. The alliance defines no protocol-level reward to members; an incentive mechanism, if introduced, is specified separately in the monetary layer.

---

## Universal-key federation

Alliance members that adopt the universal key present an identical client-facing identity: the same UUID, public key, short id, and cover SNI. One client subscription therefore authenticates to any member without per-exit credentials. Members that share a port with another public service adopt an own key and masquerade as their own real site; their subscription entry carries that member's distinct public parameters. Both classes coexist in one subscription.

The universal private key is operator-held secret material distributed out of band to alliance members; it never appears in any public artifact. A member node holds it locally to terminate client Reality sessions.

---

## Mutual insurance (the alliance property)

Censorship is per access network: an exit address reachable from one operator is filtered on another. A static one-address-per-country map fails under this. The alliance closes the gap by separating *where the client connects* from *where the client exits*:

```
client → reachable front (any alliance member the client can reach)
              → relay to chosen exit (any alliance member in the target country)
                   → clearnet
```

A client picks an exit country; the client connects to a front it can reach; the front relays to the exit. An exit whose own address is blocked from the client's operator remains usable, because the client reaches it through a front that is not blocked. Cities insure each other: the reachability of any one exit is the union of the reachabilities of all fronts that can relay to it. Blocking a country requires blocking every front that can reach its exit — across multiple operators, hosting providers, and address ranges.

### Front load model

The front carries only the relayed byte stream; it does not terminate the client's cryptographic session. The inner Noise_PQ XX session is end-to-end between client and exit (Network → Circuit Relay v2 carries ciphertext only), so the front performs no per-byte decryption and re-encryption. Cryptographic and egress load fall on the chosen exit — the server the client selected — while the front remains a light relay. This is the normative load model; a deployment that terminates and re-originates the session at the front concentrates load on the front and is a degraded fallback, not the target architecture.

---

## Discovery and selection

Members are discovered through the egress directory (Egress spec) and ranked for the client's vantage through reachability sensing (Network spec). The client selects an exit manually (a chosen country) or automatically (the reachability-ranked, lowest-latency reachable exit for its vantage). Selection is client-side and confirmed by a direct IBT handshake to the chosen exit; no front dictates the client's exit.

---

## Resilience

The alliance is available while at least one (reachable front × live exit) pair exists for a requested country. With members across multiple operators, hosting providers, address ranges, and transport profiles (Network → Transport profile ladder), the pair matrix is redundant: an adversary must simultaneously block every front's reachable transport and every exit's path to remove a country. Reachability sensing converges the client onto a working pair without operator intervention; loss of a front mid-session re-steers to another while preserving the exit and its country.

---

## Trust boundary

| Party | Learns | Does not learn |
|-------|--------|----------------|
| Front / relay | the addresses of the hops it connects | egress destinations, payload (inner session is end-to-end) |
| Exit | destinations and payload it forwards; the client's account identity | the client's source address |
| Destination | the exit's egress address | the client's identity and address |

This is the trust boundary of any honest exit and is stated, not eliminated. A client requiring no trusted forwarder runs its own member node as front, relay, and exit simultaneously; no third party then forwards its traffic. Operator-declared `country_code` is advisory and corroborated by directory quorum, not a cryptographic proof of jurisdiction.

---

## Relationship to other specifications

- Roles, directory, control messages, two-session establishment, exit policy: **Montana Egress** specification.
- Reachable-front discovery, transport profile ladder, reachability sensing, Circuit Relay v2 transit, Noise_PQ XX, IBT: **Montana Network** specification.
- Account identity, post-quantum primitives: **Montana Protocol** specification.

The alliance redefines none of these; it is the federation pattern that composes them into a censorship-resilient, country-selectable egress whose load rests on the chosen exit.
