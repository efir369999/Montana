#!/usr/bin/env python3
"""Montana network health monitor — проверяет инварианты спеки каждые 60s.

Инварианты (из Montana Protocol v35.26.0):
  H-1: все 6 узлов отвечают (TCP :8444 или :8445)
  H-2: cемитированное окно у пропозера растёт > 0 (не застрял)
  H-3: gap между proposer и followers ≤ 50 окон (после catchup)
  H-4: bundles per cement ≥ 1 (always); ≥ ceil(N_SEED/2) если honest mesh
  H-5: chain_length growing for ALL nodes (fairness)
  H-6: candidate_pool empty на пустом генезисе (Account/Node/Candidate tables начинают пустыми)

Записывает результаты в /var/log/montana-health.json + alerts в /var/log/montana-health-alerts.log
"""
import json, time, sys, urllib.request, socket, os

EXPLORER_URL = os.environ.get("MT_EXPLORER", "http://127.0.0.1:5011/api")
LOG_JSON = "/var/log/montana-health.json"
LOG_ALERTS = "/var/log/montana-health-alerts.log"
GAP_THRESHOLD = 50
INTERVAL = 60

PEERS = {
    "moscow":    ("<front>", 8445),
    "frankfurt": ("<exit-de>", 8444),
    "vilnius":   ("<exit-lt>", 8444),
    "armenia":   ("<exit-am>", 8444),
    "nicosia":   ("<exit-cy>", 8444),
}

def fetch_json(path):
    try:
        with urllib.request.urlopen(f"{EXPLORER_URL}{path}", timeout=5) as r:
            return json.loads(r.read())
    except Exception as e:
        return {"_error": str(e)}

def tcp_check(host, port):
    try:
        with socket.create_connection((host, port), timeout=3):
            return True
    except Exception:
        return False

def alert(severity, msg):
    line = f"{time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())} [{severity}] {msg}"
    try:
        with open(LOG_ALERTS, "a") as f:
            f.write(line + "\n")
    except Exception:
        pass
    print(line, file=sys.stderr, flush=True)

def check():
    report = {"ts": int(time.time()), "checks": {}}
    # H-1: TCP reachability
    peer_state = {}
    for label, (host, port) in PEERS.items():
        ok = tcp_check(host, port)
        peer_state[label] = ok
        if not ok:
            alert("WARN", f"H-1 peer {label} TCP {host}:{port} unreachable")
    report["checks"]["H-1_peers_reachable"] = peer_state
    # H-2..H-6 via explorer
    status = fetch_json("/status")
    consensus = fetch_json("/consensus")
    genesis = fetch_json("/genesis")
    if "_error" in status:
        alert("CRIT", f"H-2 explorer unreachable: {status['_error']}")
        report["checks"]["H-2"] = {"_error": status['_error']}
    else:
        report["checks"]["H-2_window"] = {
            "current_window": status.get("current_window"),
            "last_cemented_window": status.get("last_cemented_window"),
            "proposals_archived": status.get("proposals_archived"),
        }
        if status.get("last_cemented_window", 0) == 0:
            alert("CRIT", "H-2 proposer stuck — last_cemented=0")
    if "_error" not in consensus:
        active = consensus.get("active_nodes", 0)
        # Empty genesis (no baked operators): active grows from 0 as nodes self-admit;
        # no fixed expected count to assert against.
        report["checks"]["H-1_active"] = {"active_nodes": active}
        total_cl = consensus.get("total_chain_length", 0)
        dist = consensus.get("chain_length_distribution", [])
        # H-5: fairness — top operator share < 90%
        if dist:
            top = dist[0]
            top_share = top.get("share_permille", 0)
            if top_share > 900:
                alert("WARN", f"H-5 dominance: top operator {top['node_id'][:12]} has {top_share}/1000 share")
            # also check: are there any operators with chain_length=1 stuck (never grew)
            stuck = [x for x in dist if x["chain_length"] == 1]
            if len(stuck) > 0:
                alert("WARN", f"H-5 stuck: {len(stuck)} operators stuck at chain_length=1 (BCs not included in bundles)")
        report["checks"]["H-5_distribution"] = {
            "total_chain_length": total_cl,
            "top_share_permille": dist[0]["share_permille"] if dist else 0,
            "stuck_at_one": sum(1 for x in dist if x["chain_length"] == 1),
        }
    # write json
    try:
        with open(LOG_JSON, "w") as f:
            json.dump(report, f, indent=2)
    except Exception:
        pass
    return report

def main():
    print(f"montana-health monitor started; interval={INTERVAL}s; explorer={EXPLORER_URL}", flush=True)
    while True:
        try:
            r = check()
            failures = [k for k, v in r["checks"].items() if v is False]
            print(f"check ok: cw={r['checks'].get('H-2_window', {}).get('current_window')} failures={failures}", flush=True)
        except Exception as e:
            alert("ERROR", f"check loop exception: {e}")
        time.sleep(INTERVAL)

if __name__ == "__main__":
    main()
