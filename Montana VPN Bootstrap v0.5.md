# Montana VPN — Bootstrap Deployment

**Version:** 0.5 (2026-05-31)

**Layer:** Operational — the currently-deployed Montana VPN service, predating the protocol-level [Montana Egress v1.0.0](./Montana%20Egress%20v1.0.0.md). This document describes the running infrastructure; the Egress specification describes the future peer-to-peer egress that will eventually replace it.

---

## Purpose

The Bootstrap layer is the live, user-facing Montana VPN that participants use today. It uses xray/Reality as the transport, a single front node performs UUID-based routing to country-specific exit backends, a subscription endpoint distributes one VLESS link per country, and a warm-standby mirror keeps the subscription reachable if the front goes down. There is no consensus state, no protocol-level identity, and no peer discovery — those belong to the Egress specification and arrive with the mesh.

The Bootstrap commits to **zero retention of per-user data**: no source address, no destination, no timestamp, no per-session record is written to persistent storage anywhere in the path. The trade-offs that follow from this commitment are stated below.

---

## Topology

Six nodes, three roles:

| Role | Count | Function |
|------|-------|----------|
| Front | 1 (Moscow) | Public TCP :443. nginx stream layer demultiplexes by TLS SNI; xray Reality terminates the VPN session and routes per client UUID to the chosen exit. |
| Front mirror | 1 (Frankfurt) | Warm standby. Holds an rsync-synced copy of `nodes.json`, the per-tag online snapshot, and the universal keys. Serves the same `/vpn/sub` content via DNS multi-A failover. |
| Exit | 5 (Frankfurt, Yerevan, Vilnius, Nicosia, Lauterbourg) | One xray node per country. Each accepts a VLESS connection from the front and forwards traffic to the clearnet. The Frankfurt node serves both `frankfurt-cascade` exit and front mirror. |

The Moscow node carries no exit role — clearnet traffic never egresses through Moscow.

Cities only. Operational IP addresses live in private snapshots (`_internal-private/network-snapshots/`) and not in public artefacts.

---

## Front: SNI demultiplexing

The front's `:443` is a shared TCP port serving two distinct upstreams. nginx stream performs `ssl_preread` of the ClientHello, matches the SNI, and proxies to the corresponding local upstream over the loopback with **PROXY protocol v1** prepended:

```
SNI = www.googletagmanager.com   →   xray Reality (loopback)
SNI = any other (montana.quest)  →   nginx HTTP (loopback)
```

Both upstreams declare PROXY protocol acceptance. The HTTP upstream restores the real client address with `set_real_ip_from 127.0.0.1; real_ip_header proxy_protocol;`; the xray upstream restores it with `streamSettings.tcpSettings.acceptProxyProtocol = true`. PROXY-protocol acceptance on both upstreams is mandatory whenever the stream layer prepends PROXY — toggling one without the other is a TLS handshake failure path that has bitten this deployment once already; the pair is changed atomically.

The real client address is restored only into the request-handling memory of each upstream — it is never written to disk (see Privacy).

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
- A label of the form `<flag> <country> Монтана · <N>👤`, where `N` is the per-country online count derived from the live counter (see SSOT below)
- Identical Reality parameters across countries (same SNI, public key, shortId)

The link list is sorted by `N` descending, so the busiest country appears first.

---

## SSOT — single source of online count

The online counter is per-tag (`<country>-cascade`) and reported by the front. **This source is currently degraded by the privacy lockdown:** the access log it derived from has been turned off (see Privacy below), so the counter reports `0` for every tag until the migration to the xray stats API completes in v0.6. The subscription generator, the snapshot file format, and the per-link `· N👤` label are unchanged; only the value source is.

The historical access-log-based pipeline remains documented for reference:

```
xray access.log (front)
   └─ snapshot script (timer, period = SNAPSHOT_INTERVAL_SEC)
   └─ snapshot file: { ts, window_sec, by_email: { "X-cascade": N, ... } }
   └─ subscription generator reads snapshot
   └─ /vpn/sub labels and ordering
```

| Constant | Value | Rationale |
|----------|-------|-----------|
| `WINDOW_SEC` | 90 | A connection is considered active if its acceptance was logged within this window |
| `SNAPSHOT_INTERVAL_SEC` | 15 | Snapshot script runs this often |
| `SUBSCRIPTION_CACHE_SEC` | 20 | The subscription generator caches the snapshot for this many seconds |
| `PROFILE_UPDATE_INTERVAL_HOURS` | 1 | Client auto-refresh cadence |

The migration target (v0.6): pull per-tag counters from the xray statistics API (`StatsService`) which exposes inbound/outbound byte and connection counts without writing per-connection log lines. The result is a counter that observes load without observing the source of load.

---

## Front mirror and failover

The front mirror at Frankfurt runs an identical copy of the subscription generator (`montana-vpn-balance`) bound to `127.0.0.1:5008`, fronted by nginx on `:8442`. A systemd timer rsyncs three files from the front every minute:

```
<front>:/var/lib/montana-orchestrator/nodes.json
<front>:/var/lib/xray-online.json
<front>:/etc/montana-vpn/keys.json
```

DNS `de.montana.quest` is a two-record A set (`multi-A`) holding the front and the mirror, served by Cloudflare with TTL 60–120. Clients reach either address; if one is unreachable they retry the other automatically (browser/HTTP resolver behaviour).

A leader-election watchdog runs on the front; if its TLS-handshake health-check against any node in `CHAIN` fails `FAIL_THRESHOLD` consecutive cycles (probe interval = 15s, threshold = 3), it rewrites the A record set. The `CHAIN` is the canonical list of front-capable nodes; nodes that are not reachable from a Russian residential vantage are not eligible to host the public hostname and stay out of `CHAIN`. A historical bug where an unreachable node was retained in `CHAIN` caused the watchdog to keep collapsing the multi-A set down to a single record every 30 seconds — fixed in v0.5 by removing the dead entry; the same incident is the reason `CHAIN` membership is now part of the snapshot's pre-flight check.

---

## Privacy

The Bootstrap commits to retaining no per-user data on any node in the path:

| Surface | Before v0.5 | v0.5 (now) |
|---------|-------------|------------|
| `xray access.log` on all 6 nodes | source IP + destination + email tag per connection | `"access": "none"` — not opened, not written |
| `nginx access_log` on the front and the mirror | source IP per HTTP request to `/vpn/*` | `access_log off;` — not opened, not written |
| Persisted ledger entries (`balances.json`) | per-account `{ balance, seconds, last_node, last_hb, created, assigned_node, assigned_ts }` | per-account `{ balance, seconds }` only — PII fields stripped on every write by `_strip_pii_for_persist` |
| `systemd-journald` on every node | persistent at `/var/log/journal/` | `Storage=volatile`, RAM only, 64 MiB cap, lost on reboot |
| Historical access-log archives | gigabytes of `.gz` rotation | deleted |

What is still retained (and why it is not user data):

| Datum | Where | Why it is not PII |
|-------|-------|-------------------|
| Aggregate per-tag online count | `xray-online.json` on the front | A single integer per country — same shape as a load-average gauge |
| Public node topology | `nodes.json` | Public by design; the network is its own directory |
| Account ledger entry | `balances.json` | An address-keyed balance is a ledger entry, not a tracking record — the same shape as a Bitcoin UTXO |
| `error.log` (xray, nginx) | every node | Server-side warnings; no request URLs, no source addresses |

### What each party in the path observes

| Party | Observes | Does not observe |
|-------|----------|------------------|
| **DNS resolver (Cloudflare)** | the client's IP when resolving `montana.quest` for the subscription fetch; the multi-A answer | any VPN traffic — that does not go through Cloudflare |
| **Front (Moscow xray)** | TLS ciphertext (Reality masquerades as `www.googletagmanager.com`); the client's source IP and chosen tag are visible only in transient request-handling memory and routed to the matching outbound — neither is logged | nothing persists past the connection close |
| **Exit (Frankfurt / Yerevan / Vilnius / Nicosia / Lauterbourg)** | the destination it forwards to, for the duration of the connection | the original client address — the exit only sees the cascade tunnel's address; nothing of either is logged |
| **Destination host** | the exit's egress address | the client's identity or address |

This is the same trust boundary an honest VPN exit holds. The Bootstrap states the boundary; it does not pretend to eliminate it (the same caveat as `Egress v1.0.0 → Threat model`).

The architecture preserves anonymity from the destination, hides destinations from the entry, and writes nothing of either to disk. A passive observer at any single node in the path that is rebooted or examined post-hoc finds no per-user record — the volatile journald and the disabled access logs are the only acceptable interpretations of "zero retention."

---

## DNS

`de.montana.quest` is a multi-A record (front + front mirror), TTL 60–120. The subscription embeds this hostname in every VLESS URL. If a front goes down, the watchdog removes the failed address from the set within `FAIL_THRESHOLD × probe_interval = 45s`; resolvers pick it up within TTL. The exit addresses are not in public DNS at all.

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
| Online counting | xray stats API on the front (v0.6) | Out of scope (advisory directory only) |
| Retention | Zero (v0.5) | Zero by design |

The Bootstrap is the running service until enough Montana mesh nodes exist to make Egress v1.0.0 deployable.

---

## Version history

| Version | Date | Change |
|---------|------|--------|
| 0.1 | 2026-05-30 | First named version. Cleanup of legacy exits (Helsinki, NYC) from front, exits, subscription, and registry. Online counter present but reading loopback addresses — counts always zero. |
| 0.2 | 2026-05-30 | PROXY protocol added on the nginx-stream→xray and nginx-stream→nginx-HTTP loopback hops. Snapshot script and subscription generator both moved to the front node. Real client addresses now visible; counter produces real numbers. |
| 0.3 | 2026-05-30 | `profile-update-interval` reduced from 12 hours to 1 hour so clients see updated counts without manual refresh. |
| 0.4 | 2026-05-31 | Lauterbourg added as 5th exit (5-node sub). DNS multi-A `de.montana.quest` activated (Moscow + Frankfurt warm-standby mirror, watchdog leader-election). |
| 0.5 | 2026-05-31 | Privacy lockdown — xray `access.log` disabled on all 6 nodes, nginx `access_log off` on front and mirror, `balances.json` PII fields stripped, journald set to volatile mode. Online counter temporarily reports 0 until xray stats API migration in v0.6. |

Each version bumps the `Монтана X.Y` string in `profile-title`; clients re-render the profile name when it changes.

---

## Out of scope

This document does not specify protocol-level egress, identity, peer discovery, or consensus. Those belong to [Montana Egress v1.0.0](./Montana%20Egress%20v1.0.0.md), [Montana Network v1.3.0](./Montana%20Network%20v1.3.0.md), and [Montana Protocol](./Montana%20Protocol%20v35.26.0.md). The Bootstrap is an operational service that consumes the public internet and runs xray; nothing here enters a state root.

When the next bootstrap version ships, this document is renamed to `Montana VPN Bootstrap v<new>.md`. The previous file is kept in `_internal-private/network-snapshots/` as a frozen baseline for rollback reasoning.
