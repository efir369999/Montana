#!/usr/bin/env python3
"""Montana explorer JSON API v3 — расширенная.

Routes:
  /api/status            — current/last_cemented_window, accounts/nodes/candidates counts
  /api/nodes             — все NodeTable записи
  /api/proposals?limit=N — последние N proposal headers
  /api/proposal/<W>      — единичный proposal header
  /api/window/<W>        — window detail: header + bundles list
  /api/peers             — live peer соединения из journalctl
  /api/consensus         — состояние консенсуса: proposer, quorum, chain_length с labels
  /api/genesis           — Genesis manifest content
  /api/winners?limit=N   — последние N cement winners с labels (proposer/winner)
"""
import os, sys, json, struct, glob, time, subprocess, re
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import urlparse, parse_qs

DATA_DIR = os.environ.get("MT_DATA_DIR", "/var/lib/docker/volumes/montana-data/_data")
MANIFEST_PATH = os.environ.get("MT_MANIFEST", "/etc/montana/genesis-manifest.json")
PROPOSALS = os.path.join(DATA_DIR, "proposals")
ACCOUNTS_BIN = os.path.join(DATA_DIR, "accounts.bin")
NODES_BIN = os.path.join(DATA_DIR, "nodes.bin")
CANDIDATES_BIN = os.path.join(DATA_DIR, "candidates.bin")
META_LAST = os.path.join(DATA_DIR, "meta_last_cemented.bin")
CURRENT_WIN = os.path.join(DATA_DIR, "current_window.bin")

ACCOUNT_SZ = 2059
NODE_SZ = 2098
CANDIDATE_SZ = 2082
PROPOSAL_HDR_SZ = 3722

def read_u64_le(path):
    try:
        with open(path, "rb") as f:
            return struct.unpack("<Q", f.read(8))[0]
    except Exception:
        return None

def parse_proposal_header(buf):
    if len(buf) < PROPOSAL_HDR_SZ:
        return None
    bundle_count = 0
    if len(buf) >= PROPOSAL_HDR_SZ + 2:
        bundle_count = struct.unpack("<H", buf[PROPOSAL_HDR_SZ:PROPOSAL_HDR_SZ+2])[0]
    return {
        "prev_proposal_hash": buf[0:32].hex(),
        "window_index": struct.unpack("<Q", buf[32:40])[0],
        "protocol_version": struct.unpack("<I", buf[40:44])[0],
        "control_root": buf[44:76].hex(),
        "node_root": buf[76:108].hex(),
        "candidate_root": buf[108:140].hex(),
        "account_root": buf[140:172].hex(),
        "state_root": buf[172:204].hex(),
        "timechain_value": buf[204:236].hex(),
        "included_bundles_root": buf[236:268].hex(),
        "included_reveals_root": buf[268:300].hex(),
        "winner_endpoint": buf[300:332].hex(),
        "winner_id": buf[332:364].hex(),
        "proposer_node_id": buf[364:396].hex(),
        "target": int.from_bytes(buf[396:412], "little"),
        "fallback_depth": buf[412],
        "envelope_size": len(buf),
        "bundle_count": bundle_count,
        "is_cemented_envelope": len(buf) > PROPOSAL_HDR_SZ,
    }

def parse_node_record(buf):
    if len(buf) < NODE_SZ:
        return None
    return {
        "node_id": buf[0:32].hex(),
        "node_pubkey_first16": buf[32:48].hex(),
        "suite_id": struct.unpack("<H", buf[1984:1986])[0],
        "operator_account_id": buf[1986:2018].hex(),
        "start_window": struct.unpack("<Q", buf[2018:2026])[0],
        "chain_length": struct.unpack("<Q", buf[2026:2034])[0],
        "chain_length_snapshot": struct.unpack("<Q", buf[2034:2042])[0],
        "last_confirmation_window": struct.unpack("<Q", buf[2090:2098])[0],
    }

def parse_bundle_in_envelope(buf, off):
    if off + 32 + 32 + 8 + 2 > len(buf):
        return None, 0
    node_id = buf[off:off+32].hex()
    endpoint = buf[off+32:off+64].hex()
    window = struct.unpack("<Q", buf[off+64:off+72])[0]
    op_count = struct.unpack("<H", buf[off+72:off+74])[0]
    after_ops = off + 74 + 32 * op_count
    if after_ops + 2 > len(buf):
        return None, 0
    reveal_count = struct.unpack("<H", buf[after_ops:after_ops+2])[0]
    after_reveals = after_ops + 2 + 32 * reveal_count
    if after_reveals + 4032 > len(buf):
        return None, 0
    used = after_reveals + 4032 - off
    return {
        "node_id": node_id,
        "endpoint": endpoint,
        "window_index": window,
        "op_hashes_count": op_count,
        "reveal_hashes_count": reveal_count,
    }, used

def _labels():
    if not os.path.exists(MANIFEST_PATH):
        return {}
    try:
        with open(MANIFEST_PATH) as f:
            m = json.load(f)
        return {p["node_id_hex"]: p["label"] for p in m.get("peers", []) if p.get("node_id_hex")}
    except Exception:
        return {}

def api_status():
    cur_win = read_u64_le(CURRENT_WIN)
    last_cem = read_u64_le(META_LAST)
    if not cur_win or cur_win == 0:
        cur_win = last_cem
    n_acc = (os.path.getsize(ACCOUNTS_BIN) // ACCOUNT_SZ) if os.path.exists(ACCOUNTS_BIN) else 0
    n_node = (os.path.getsize(NODES_BIN) // NODE_SZ) if os.path.exists(NODES_BIN) else 0
    n_cand = (os.path.getsize(CANDIDATES_BIN) // CANDIDATE_SZ) if os.path.exists(CANDIDATES_BIN) else 0
    props = sorted(glob.glob(os.path.join(PROPOSALS, "*.bin")))
    return {
        "current_window": cur_win,
        "last_cemented_window": last_cem,
        "accounts": n_acc,
        "nodes": n_node,
        "candidates": n_cand,
        "proposals_archived": len(props),
        "ts": int(time.time()),
    }

def api_nodes():
    if not os.path.exists(NODES_BIN):
        return {"nodes": []}
    with open(NODES_BIN, "rb") as f:
        data = f.read()
    out = []
    labels = _labels()
    for i in range(0, len(data), NODE_SZ):
        rec = parse_node_record(data[i:i + NODE_SZ])
        if rec:
            rec["label"] = labels.get(rec["node_id"], "unknown")
            out.append(rec)
    return {"nodes": out, "count": len(out)}

def api_proposals(limit=30):
    files = sorted(glob.glob(os.path.join(PROPOSALS, "*.bin")), reverse=True)[:limit]
    out = []
    labels = _labels()
    for fp in files:
        try:
            with open(fp, "rb") as f:
                buf = f.read()
            hdr = parse_proposal_header(buf)
            if hdr:
                out.append({
                    "window_index": hdr["window_index"],
                    "proposer_node_id": hdr["proposer_node_id"],
                    "proposer_label": labels.get(hdr["proposer_node_id"], "unknown"),
                    "winner_id": hdr["winner_id"],
                    "winner_label": labels.get(hdr["winner_id"], "unknown"),
                    "state_root": hdr["state_root"],
                    "envelope_size": hdr["envelope_size"],
                    "bundle_count": hdr["bundle_count"],
                })
        except Exception:
            continue
    return {"proposals": out, "count": len(out)}

def api_proposal(window):
    fp = os.path.join(PROPOSALS, f"{window:020d}.bin")
    if not os.path.exists(fp):
        return {"error": "not found"}, 404
    with open(fp, "rb") as f:
        buf = f.read()
    hdr = parse_proposal_header(buf)
    if not hdr:
        return {"error": "parse error"}, 500
    return hdr

def api_window(window):
    fp = os.path.join(PROPOSALS, f"{window:020d}.bin")
    if not os.path.exists(fp):
        return {"error": "not found"}, 404
    with open(fp, "rb") as f:
        buf = f.read()
    hdr = parse_proposal_header(buf)
    if not hdr:
        return {"error": "parse error"}, 500
    bundles = []
    if hdr["bundle_count"] > 0:
        off = PROPOSAL_HDR_SZ + 2
        for _ in range(hdr["bundle_count"]):
            bc, used = parse_bundle_in_envelope(buf, off)
            if bc is None:
                break
            bundles.append(bc)
            off += used
    return {"header": hdr, "bundles": bundles, "confirmer_count": len(bundles)}

def api_peers():
    try:
        out = subprocess.check_output(
            ["journalctl", "-u", "montana-node", "--since", "60s ago", "--no-pager"],
            stderr=subprocess.DEVNULL, timeout=10
        ).decode("utf-8", errors="ignore")
        peers = set()
        for m in re.finditer(r"peer=(Qm[a-zA-Z0-9]+|12D3[a-zA-Z0-9]+)", out):
            peers.add(m.group(1))
        return {"peers": sorted(peers), "count": len(peers), "window_seconds": 60}
    except Exception as e:
        return {"peers": [], "count": 0, "error": str(e)}

def api_consensus():
    status = api_status()
    n_data = api_nodes()
    nodes = n_data["nodes"]
    if not nodes:
        return {"error": "no nodes"}, 503
    labels = _labels()
    total_chain_length = sum(n["chain_length"] for n in nodes)
    quorum = (67 * total_chain_length + 99) // 100
    proposer = max(nodes, key=lambda n: n["chain_length"]) if nodes else None
    return {
        "current_window": status["current_window"],
        "last_cemented_window": status["last_cemented_window"],
        "active_nodes": len(nodes),
        "total_chain_length": total_chain_length,
        "quorum_required": quorum,
        "proposer_node_id": proposer["node_id"] if proposer else None,
        "proposer_label": labels.get(proposer["node_id"], "unknown") if proposer else None,
        "chain_length_distribution": [
            {
                "node_id": n["node_id"][:16],
                "label": labels.get(n["node_id"], "unknown"),
                "chain_length": n["chain_length"],
                "share_permille": (1000 * n["chain_length"]) // max(1, total_chain_length),
            }
            for n in sorted(nodes, key=lambda n: -n["chain_length"])
        ],
    }

def api_genesis():
    if not os.path.exists(MANIFEST_PATH):
        return {"error": "manifest not found"}, 404
    try:
        with open(MANIFEST_PATH) as f:
            m = json.load(f)
        n_seed = sum(1 for p in m.get("peers", []) if p.get("force_active"))
        return {
            "network_name": m.get("network_name"),
            "peer_count": len(m.get("peers", [])),
            "n_seed": n_seed,
            "peers": [
                {
                    "label": p["label"], "multiaddr": p["multiaddr"], "peer_id": p["peer_id"],
                    "force_active": p.get("force_active", False),
                    "node_id_hex": p.get("node_id_hex"),
                    "account_id_hex": p.get("account_id_hex"),
                }
                for p in m.get("peers", [])
            ],
        }
    except Exception as e:
        return {"error": str(e)}, 500

def api_winners(limit=20):
    files = sorted(glob.glob(os.path.join(PROPOSALS, "*.bin")), reverse=True)[:limit]
    labels = _labels()
    out = []
    for fp in files:
        try:
            with open(fp, "rb") as f:
                buf = f.read()
            hdr = parse_proposal_header(buf)
            if hdr:
                out.append({
                    "window_index": hdr["window_index"],
                    "winner_id": hdr["winner_id"],
                    "winner_label": labels.get(hdr["winner_id"], "unknown"),
                    "proposer_node_id": hdr["proposer_node_id"],
                    "proposer_label": labels.get(hdr["proposer_node_id"], "unknown"),
                    "bundle_count": hdr["bundle_count"],
                })
        except Exception:
            continue
    return {"winners": out, "count": len(out)}

class H(BaseHTTPRequestHandler):
    def log_message(self, *a, **k): pass
    def _send(self, body, code=200):
        body_b = json.dumps(body, ensure_ascii=False, indent=2).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Content-Length", str(len(body_b)))
        self.end_headers()
        self.wfile.write(body_b)
    def do_GET(self):
        u = urlparse(self.path)
        q = parse_qs(u.query)
        try:
            if u.path == "/api/status":
                return self._send(api_status())
            if u.path == "/api/nodes":
                return self._send(api_nodes())
            if u.path == "/api/proposals":
                limit = min(int(q.get("limit", [30])[0]), 200)
                return self._send(api_proposals(limit))
            if u.path.startswith("/api/proposal/"):
                w = int(u.path.split("/")[-1])
                r = api_proposal(w)
                if isinstance(r, tuple): return self._send(r[0], r[1])
                return self._send(r)
            if u.path.startswith("/api/window/"):
                w = int(u.path.split("/")[-1])
                r = api_window(w)
                if isinstance(r, tuple): return self._send(r[0], r[1])
                return self._send(r)
            if u.path == "/api/peers":
                return self._send(api_peers())
            if u.path == "/api/consensus":
                r = api_consensus()
                if isinstance(r, tuple): return self._send(r[0], r[1])
                return self._send(r)
            if u.path == "/api/genesis":
                r = api_genesis()
                if isinstance(r, tuple): return self._send(r[0], r[1])
                return self._send(r)
            if u.path == "/api/winners":
                limit = min(int(q.get("limit", [20])[0]), 200)
                return self._send(api_winners(limit))
            self._send({"error": "unknown route", "path": u.path, "available": [
                "/api/status", "/api/nodes", "/api/proposals", "/api/proposal/<W>",
                "/api/window/<W>", "/api/peers", "/api/consensus", "/api/genesis", "/api/winners",
            ]}, 404)
        except Exception as e:
            self._send({"error": str(e)}, 500)

if __name__ == "__main__":
    port = int(os.environ.get("MT_EXPLORER_PORT", "5011"))
    print(f"montana-explorer v3 listening on :{port} reading {DATA_DIR}", flush=True)
    HTTPServer(("0.0.0.0", port), H).serve_forever()
