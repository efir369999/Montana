# Montana VPN — Bootstrap Deployment

**Version:** 0.3 (2026-05-30)

**Layer:** Operational — the currently-deployed Montana VPN service, predating the protocol-level [Montana Egress v1.0.0](./Montana%20Egress%20v1.0.0.md). This document describes the running infrastructure; the Egress specification describes the future peer-to-peer egress that will eventually replace it.

---

## Purpose

The Bootstrap layer is the live, user-facing Montana VPN that participants use today. It uses xray/Reality as the transport, a single front node performs UUID-based routing to country-specific exit backends, and a subscription endpoint distributes one VLESS link per country. There is no consensus state, no protocol-level identity, and no peer discovery — those belong to the Egress specification and arrive with the mesh.

---

## Topology

Five nodes, two roles:

| Role | Count | Function |
|------|-------|----------|
| Front | 1 (Moscow) | Public TCP :443. nginx stream layer demultiplexes by TLS SNI; xray Reality terminates the VPN session and routes per client UUID to the chosen exit. |
| Exit | 4 (Frankfurt, Yerevan, Vilnius, Nicosia) | One xray node per country. Each accepts a VLESS connection from the front and forwards traffic to the clearnet. The Frankfurt exit also serves the `frankfurt-cascade` UUID. |

The Moscow node carries no exit role — clearnet traffic never egresses through Moscow.

Cities only. Operational IP addresses live in private snapshots (`_internal-private/network-snapshots/`) and not in public artefacts.

---

## Front: SNI demultiplexing

The front's `:443` is a shared TCP port serving two distinct upstreams. nginx stream performs `ssl_preread` of the ClientHello, matches the SNI, and proxies to the corresponding local upstream over the loopback with **PROXY protocol v1** prepended:

```
SNI = www.googletagmanager.com   →   xray Reality (loopback)
SNI = any other (montana.quest)  →   nginx HTTP (loopback)
```

Both upstreams declare PROXY protocol acceptance, so the real client address is preserved end-to-end despite the loopback hop. The HTTP upstream restores it with `set_real_ip_from 127.0.0.1; real_ip_header proxy_protocol;`; the xray upstream restores it with `streamSettings.tcpSettings.acceptProxyProtocol = true`.

This preservation is the load-bearing change in v0.2; without it every client appeared in xray's access log as `127.0.0.1` and per-user counters were unusable.

---

## Routing: one front, per-UUID outbound

The xray Reality inbound on the front exposes one VLESS client per country. Each client's `email` tag triggers a routing rule whose `outboundTag` selects the country-specific exit:

```
UUID-of-country-X  →  email "X-cascade"  →  outbound "X-out"  →  exit node X
```

The single Reality entry on the front, plus one client UUID per supported country, plus one routing rule per UUID, plus one VLESS outbound per exit: this is the entire cascade. The subscription's only difference between countries is the UUID encoded in the VLESS URL — the host, port, SNI, Reality public key, and shortId are identical across all links.

---

## Subscription

`GET https://montana.quest/vpn/sub` returns the base64-encoded VLESS subscription. Headers:

- `profile-title: base64:<UTF-8 "Монтана <NETWORK_VERSION>">` — clients show this as the profile name and re-fetch when it changes
- `profile-update-interval: 1` — clients automatically refresh once per hour
- `Cache-Control: no-store`

Each VLESS link carries:

- A per-country client UUID
- A label of the form `<flag> <country> Монтана · <N>👤`, where `N` is the count of distinct active source addresses observed for that UUID in the last `WINDOW_SEC` seconds (see SSOT below)
- Identical Reality parameters across countries (same SNI, public key, shortId)

The link list is sorted by `N` descending, so the busiest country appears first.

---

## SSOT — single source of online count

The xray access log on the front is the single source of truth for who is connected to which exit.

```
xray access.log (front)
   └─ snapshot script (timer, period = SNAPSHOT_INTERVAL_SEC)
   └─ snapshot file: { ts, window_sec, by_email: { "X-cascade": N, ... } }
   └─ subscription generator reads snapshot
   └─ /vpn/sub labels and ordering
```

The snapshot script counts **distinct source addresses per email tag** within a sliding window. Two filters apply:

1. **Internal addresses** — every Montana own-IP (front, all exits) is dropped, otherwise loopback or backend-to-backend probes pollute the count
2. **Reserved emails** — the universal-key email and any verification email are dropped; they are not part of the cascade product

Parameters:

| Constant | Value | Rationale |
|----------|-------|-----------|
| `WINDOW_SEC` | 90 | A connection is considered active if its acceptance was logged within this window |
| `SNAPSHOT_INTERVAL_SEC` | 15 | Snapshot script runs this often |
| `SUBSCRIPTION_CACHE_SEC` | 20 | The subscription generator caches the snapshot for this many seconds |
| `PROFILE_UPDATE_INTERVAL_HOURS` | 1 | Client auto-refresh cadence |

End-to-end latency from a new client connection to a visible count change in a client's UI: ~`SNAPSHOT_INTERVAL_SEC + SUBSCRIPTION_CACHE_SEC + manual refresh`, or up to one hour if the user does not pull to refresh.

The subscription generator runs on the front node and reads the snapshot file from the local filesystem. There is no cross-node transport for online counts — the entire count chain is local to the front.

---

## DNS

The single public hostname `de.montana.quest` resolves to the front. The subscription embeds this hostname in every VLESS URL. There is one DNS record to rotate if the front is blocked; the exit addresses are not in public DNS at all.

---

## Comparison with Egress v1.0.0

| Concern | Bootstrap (this document) | Egress v1.0.0 (target) |
|---------|---------------------------|------------------------|
| Transport | xray Reality (VLESS) | Noise_PQ XX, two nested sessions |
| Entry | One centralized front | Any reachable mesh peer, IBT-authenticated |
| Exit selection | Client picks UUID in subscription | Client picks from egress directory by `country_code` |
| Authentication | UUID possession | IBT level-3 (ML-DSA-65 account proof) |
| Entry sees destinations | Yes (front terminates the inner session) | No (entry carries ciphertext only) |
| Exit attribution | Backend operator | Opt-in mesh node operator |
| Online counting | xray access log on the front | Out of scope (advisory directory only) |

The Bootstrap is the running service until enough Montana mesh nodes exist to make Egress v1.0.0 deployable.

---

## Version history

| Version | Date | Change |
|---------|------|--------|
| 0.1 | 2026-05-30 | First named version. Cleanup of legacy exits (Helsinki, NYC) from front, exits, subscription, and registry. Online counter present but reading loopback addresses — counts always zero. |
| 0.2 | 2026-05-30 | PROXY protocol added on the nginx-stream→xray and nginx-stream→nginx-HTTP loopback hops. Snapshot script and subscription generator both moved to the front node. Real client addresses now visible; counter produces real numbers. |
| 0.3 | 2026-05-30 | `profile-update-interval` reduced from 12 hours to 1 hour so clients see updated counts without manual refresh. |

Each version bumps the `Монтана X.Y` string in `profile-title`; clients re-render the profile name when it changes.

---

## Out of scope

This document does not specify protocol-level egress, identity, peer discovery, or consensus. Those belong to [Montana Egress v1.0.0](./Montana%20Egress%20v1.0.0.md), [Montana Network v1.3.0](./Montana%20Network%20v1.3.0.md), and [Montana Protocol](./Montana%20Protocol%20v35.26.0.md). The Bootstrap is an operational service that consumes the public internet and runs xray; nothing here enters a state root.

When the next bootstrap version ships, this document is renamed to `Montana VPN Bootstrap v<new>.md`. The previous file is kept in `_internal-private/network-snapshots/` as a frozen baseline for rollback reasoning.
