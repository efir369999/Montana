#!/usr/bin/env python3
# Montana explorer data.json collector.
# Polls the three Genesis nodes for their montana-node status and merges in
# any auto-discovered peer (any peer connected with label=unknown that has
# emitted heartbeat OK in the recent journal window). The result is written
# to /var/www/efir/explorer/data.json once per minute.

import json
import re
import subprocess
import time
import os
from datetime import datetime, timezone

OUT = "/var/www/efir/explorer/data.json"
IP_CACHE = "/var/lib/montana-explorer/discovered-ip-cache.json"
SEEN_CACHE = "/var/lib/montana-explorer/discovered-seen-since.json"
DISCOVERY_WINDOW_SECONDS = 600  # consider a peer "live" if heartbeat OK within 10 min

# Peer-id keyed public label map. Genesis peers + known external operators.
# Per the public-artifact rule, no raw IPs appear in /explorer/data.json — IPs
# are uniformly masked to "hidden" and the peer-id carries the public label.
PEER_PUBLIC_LABEL = {
    "QmSDUqLkLcenkkNw6PUKYXjesEmaDksnrEaCzbs3a5nVzj": "moscow",
    "QmPFm5L3WiA47J66zVJvio23QBgBqr4nAqCP626vgEnHNP": "frankfurt",
    "QmNSrA82XExjEXUS5xTPhn9MV55bfhYofxfcm7dTFcQPjL": "helsinki",
    "Qma3XZ8mJZDD4MbtJVNxCyS2sYYn9BQRzxYvfiXiMbNCp9": "yerevan",
    "QmYEFQZmBqWYV7SFreMK6h7N87fVasNv8ho5GU27La8Y9z": "macbook",
}

def mask_ip(ip):
    # Only the orchestrator placeholder "local" is kept; every other IP is hidden.
    if ip in ("local", "", "?"):
        return ip
    return "hidden"

def peer_label(peer_id):
    return PEER_PUBLIC_LABEL.get(peer_id, "external")


# Pinned Genesis XX peer_ids — any other peer_id seen in journals is a discovered (external) node.
GENESIS_PEER_IDS = {
    "QmSDUqLkLcenkkNw6PUKYXjesEmaDksnrEaCzbs3a5nVzj",  # moscow
    "QmPFm5L3WiA47J66zVJvio23QBgBqr4nAqCP626vgEnHNP",  # frankfurt
    "QmNSrA82XExjEXUS5xTPhn9MV55bfhYofxfcm7dTFcQPjL",  # helsinki
}

GENESIS_NODES = [
    ("Moscow",    "local"),
    ("Helsinki",  "<exit-removed>"),
    ("Frankfurt", "<exit-de>"),
]


def run_local(cmd, timeout=10):
    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout).stdout


def run_ssh(host, remote_cmd, timeout=10):
    cmd = [
        "ssh", "-o", "ConnectTimeout=5", "-o", "StrictHostKeyChecking=no",
        f"root@{host}", remote_cmd,
    ]
    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout).stdout


def parse_status(text):
    if not text:
        return None
    def grep(pattern, default=None, group=1):
        m = re.search(pattern, text, re.MULTILINE)
        return m.group(group) if m else default
    return {
        "current_window": int(grep(r"^current_window\s*:\s*(\d+)", "0") or 0),
        "phase": grep(r"^phase\s*:\s*(\S+)", "unknown"),
        "D": int(grep(r"^D \(current\)\s*:\s*(\d+)", "0") or 0),
        "account_id": grep(r"^account_id\s*:\s*([0-9a-f]+)", ""),
        "node_id": grep(r"^node_id\s*:\s*([0-9a-f]+)", ""),
        "balance_nj": int(grep(r"^balance\s*:\s*(\d+)\s*nɈ", "0") or 0),
        "supply_nj": int(grep(r"supply \(closed-form\)\s*:\s*(\d+)", "0") or 0),
        "account_table": int(grep(r"^AccountTable\s*:\s*(\d+)", "0") or 0),
        "node_table": int(grep(r"^NodeTable\s*:\s*(\d+)", "0") or 0),
    }


def fetch_genesis(label, host):
    try:
        if host == "local":
            out = run_local(
                ["/usr/local/bin/montana-node", "status",
                 "--data-dir", "/var/lib/montana"]
            )
        else:
            out = run_ssh(
                host,
                "/usr/local/bin/montana-node status --data-dir /var/lib/montana"
            )
        st = parse_status(out)
        if st:
            return {"label": label, "host": mask_ip(host), "status": "active", **st}
        return {"label": label, "host": mask_ip(host), "status": "no_data"}
    except Exception as e:
        return {"label": label, "host": mask_ip(host), "status": "unreachable",
                "error": str(e)[:100]}


# Patterns:
# [network] CONNECTION ESTABLISHED peer=<XX> label=<L> remote=/ip4/<IP>/tcp/<PORT>...
# [network] heartbeat OK peer=<XX> request_id=N
# [network] connection closed peer=<XX> label=<L> cause=...
CONN_ESTABLISHED_RE = re.compile(
    r"CONNECTION ESTABLISHED peer=(\S+) label=(\S+) remote=/ip4/(\d+\.\d+\.\d+\.\d+)/tcp/(\d+)"
)
HEARTBEAT_RE = re.compile(r"heartbeat OK peer=(\S+)")
CONN_CLOSED_RE = re.compile(r"connection closed peer=(\S+)")


def collect_discovery(witness_label, witness_host):
    """Scan the witness node's recent journal for discovered (unknown-label) peers.

    Returns: dict mapping peer_id → {label, remote_ip, last_heartbeat_seconds_ago}.
    """
    try:
        cmd_str = (
            f"journalctl -u montana-node --since '{DISCOVERY_WINDOW_SECONDS} seconds ago' "
            "--no-pager -o short-iso"
        )
        if witness_host == "local":
            raw = run_local(["bash", "-lc", cmd_str], timeout=15)
        else:
            raw = run_ssh(witness_host, cmd_str, timeout=15)
    except Exception:
        return {}

    # Pass 1: collect ConnectionEstablished events (peer_id → remote_ip + label).
    # Persist seen-IPs across runs so peers whose ConnectionEstablished event aged out of
    # the 10-min journalctl window keep their resolved IP.
    try:
        with open(IP_CACHE, "r") as f:
            seen_ip = json.load(f)
        if not isinstance(seen_ip, dict):
            seen_ip = {}
    except Exception:
        seen_ip = {}
    closed = set()
    for line in raw.splitlines():
        m = CONN_ESTABLISHED_RE.search(line)
        if m:
            peer_id, label, remote_ip, _port = m.group(1), m.group(2), m.group(3), m.group(4)
            if label == "unknown":
                seen_ip[peer_id] = remote_ip
        m = CONN_CLOSED_RE.search(line)
        if m:
            closed.add(m.group(1))

    # Pass 2: track latest heartbeat timestamp for every peer in window.
    last_hb = {}
    for line in raw.splitlines():
        m = HEARTBEAT_RE.search(line)
        if m:
            peer_id = m.group(1)
            ts_match = re.match(r"(\S+)", line)
            if ts_match:
                last_hb[peer_id] = ts_match.group(1)

    # Pass 3: union of (seen-via-ConnectionEstablished) + (any heartbeat OK from a
    # non-Genesis peer in the window) — the latter catches peers whose
    # ConnectionEstablished is older than the journal window but who are still alive.
    discovered_peers = set(seen_ip.keys())
    for peer_id in last_hb:
        if peer_id not in GENESIS_PEER_IDS:
            discovered_peers.add(peer_id)

    now = datetime.now(timezone.utc)
    result = {}
    for peer_id in discovered_peers:
        # Skip peers that disconnected and have no heartbeat after closure.
        if peer_id in closed and peer_id not in last_hb:
            continue
        info = {
            "peer_id": peer_id,
            "remote_ip": seen_ip.get(peer_id, "?"),
            "witness": witness_label,
            "first_seen_via": witness_label,
        }
        if peer_id in last_hb:
            try:
                last_ts = datetime.fromisoformat(last_hb[peer_id])
                age = (now - last_ts).total_seconds()
                info["last_heartbeat_seconds_ago"] = int(age)
            except Exception:
                info["last_heartbeat_seconds_ago"] = None
        result[peer_id] = info
    # Persist seen_ip cache so resolved IPs survive across journal-window rotations.
    try:
        os.makedirs(os.path.dirname(IP_CACHE), exist_ok=True)
        with open(IP_CACHE, "w") as f:
            json.dump(seen_ip, f, indent=2, sort_keys=True)
    except Exception:
        pass
    return result


def merge_discoveries(*discovery_maps):
    merged = {}
    for dmap in discovery_maps:
        for peer_id, info in dmap.items():
            if peer_id not in merged:
                merged[peer_id] = {**info, "witnessed_by": [info["witness"]]}
            else:
                if info["witness"] not in merged[peer_id]["witnessed_by"]:
                    merged[peer_id]["witnessed_by"].append(info["witness"])
                # keep smallest heartbeat age
                cur = merged[peer_id].get("last_heartbeat_seconds_ago")
                new = info.get("last_heartbeat_seconds_ago")
                if cur is None or (new is not None and new < cur):
                    merged[peer_id]["last_heartbeat_seconds_ago"] = new
                # prefer a witness that has a remote_ip over "?"
                if merged[peer_id].get("remote_ip", "?") == "?" and info.get("remote_ip", "?") != "?":
                    merged[peer_id]["remote_ip"] = info["remote_ip"]
    # Load/initialise the seen_since cache: peer_id -> first-observed unix timestamp.
    try:
        with open(SEEN_CACHE) as f:
            seen_since = json.load(f)
        if not isinstance(seen_since, dict):
            seen_since = {}
    except Exception:
        seen_since = {}
    now_unix = int(time.time())
    out = []
    for p in merged.values():
        pid = p["peer_id"]
        if pid not in seen_since:
            seen_since[pid] = now_unix
        hb_age = p.get("last_heartbeat_seconds_ago") or 999999
        uptime = now_unix - seen_since[pid]
        out.append({
            "peer_id": pid,
            "label": peer_label(pid),
            "remote_ip": mask_ip(p["remote_ip"]),
            "witnessed_by": p["witnessed_by"],
            "last_heartbeat_seconds_ago": p.get("last_heartbeat_seconds_ago"),
            "first_seen_unix": seen_since[pid],
            "uptime_seconds": uptime,
            "status": ("active" if hb_age < 60 else "stale"),
        })
    # Persist seen-since cache, pruning entries inactive > 7 days.
    keep = {p["peer_id"]: seen_since[p["peer_id"]] for p in merged.values()}
    seven_days = 7 * 24 * 3600
    for pid, ts in list(seen_since.items()):
        if pid not in keep and (now_unix - ts) < seven_days:
            keep[pid] = ts
    try:
        os.makedirs(os.path.dirname(SEEN_CACHE), exist_ok=True)
        with open(SEEN_CACHE, "w") as f:
            json.dump(keep, f, indent=2, sort_keys=True)
    except Exception:
        pass
    return out


# --- Build the document ---
nodes = [fetch_genesis(label, host) for (label, host) in GENESIS_NODES]

discoveries = []
for (label, host) in GENESIS_NODES:
    dmap = collect_discovery(label, host)
    discoveries.append(dmap)
discovered_peers = merge_discoveries(*discoveries)

doc = {
    "updated": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "updated_unix": int(time.time()),
    "nodes": nodes,
    "discovered_peers": discovered_peers,
    "network_summary": {
        "active_nodes": sum(1 for n in nodes if n["status"] == "active"),
        "total_nodes": len(nodes),
        "discovered_peer_count": len(discovered_peers),
        "max_window": max((n.get("current_window", 0) for n in nodes), default=0),
        "total_supply_nj": sum(
            n.get("supply_nj", 0) for n in nodes if n["status"] == "active"
        ),
    },
}

os.makedirs(os.path.dirname(OUT), exist_ok=True)
with open(OUT, "w") as f:
    json.dump(doc, f, indent=2, ensure_ascii=False)
os.chmod(OUT, 0o644)
print(json.dumps(doc, indent=2, ensure_ascii=False)[:800])
