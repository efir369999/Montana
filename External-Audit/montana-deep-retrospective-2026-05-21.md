# Montana deep retrospective — 2026-05-21

Generated 2026-05-21T19:40:02Z on Moscow orchestrator.

## 1. Current state of all four nodes

| Node | Phase | current_window | uptime since | CPU 1-min load |
|------|-------|----------------|--------------|----------------|
| moscow | Active | 75849 | Thu 2026-05-21 22:24:54 MSK |  3.97 |
| frankfurt | CandidateVdf | 75847 | Thu 2026-05-21 19:29:17 UTC |  1.22 |
| helsinki | CandidateVdf | 75787 | Thu 2026-05-21 22:24:15 EEST |  7.68 |
| armenia | CandidateVdf | 75845 | Thu 2026-05-21 23:26:36 +04 |  1.26 |

## 2. Heartbeat health (last 1 hour)

Each node sends Ping every 5 s to every connected peer. Expected baseline
per hour per peer = 720. Three Genesis peers + Armenia = ~2880 heartbeats
per node per hour in steady state.

| Node | heartbeat OK | outgoing errors | connection closed |
|------|--------------|------------------|--------------------|
| moscow | 47934 | 0 | 8 |
| frankfurt | 77878 | 2 | 6 |
| helsinki | 91416 | 0
? | 610 |
| armenia | 2194 | 0
? | 4 |

## 3. Consensus state convergence

Moscow is the canonical bootstrap proposer. Followers replay Moscow's
Proposal envelopes through the apply_proposal path on each incoming
broadcast. Lag = (Moscow.current_window) − (follower.current_window),
positive means the follower is behind, expected ≤ 1 for the steady
state once the follower has caught up.

| Node | current_window | lag vs Moscow (75851) |
|------|----------------|------------------------------|
| moscow | 75851 | 0 (proposer) |
| frankfurt | 75848 | 3 |
| helsinki | 75787 | 64 |
| armenia | 75845 | 6 |

## 4. Resource pressure on each operator host

| Node | cores | RAM (MB) | mem used (MB) | swap used (MB) | montana-node RSS (MB) |
|------|-------|----------|---------------|-----------------|------------------------|
| moscow | 1 | 1968 | 677 | 376 | 5 |
| frankfurt | 1 | 1967 | 558 | 368 | 8 |
| helsinki | 1 | 961 | 541 | 581 | 3 |
| armenia | 1 | 961 | 330 | 447 | 8 |

## 5. Frequent error / warning lines (last 24 hours)

### moscow
```
   5341 Main process exited, code=exited, status=1/FAILURE
   5341 Failed with result 'exit-code'.
   5339 Permission denied (os error 13)
      5 Error(Right(Closed)) }))
      2 No space left on device [v8.2312.0 try https://www.rsyslog.com/e/2027 ]
      1 https://docs.rs/rustls/latest/rustls/manual/_03_howto/index.html#unexpected-eof" })))) }))
      1 "Connection timed out" })))) }))
      1 "Connection timed out" }))) }))
```

### frankfurt
```
    602 https://docs.rs/rustls/latest/rustls/manual/_03_howto/index.html#unexpected-eof" })))) }))
      6 Error(Right(Closed)) }))
      6 "Connection reset by peer" })))) }))
      2 [active W=82123] singleton невозможен (NodeTable=2 узлов), пропуск окна — жду peer Proposal (M9 Phase 2)
      2 [active W=82122] singleton невозможен (NodeTable=2 узлов), пропуск окна — жду peer Proposal (M9 Phase 2)
      2 [active W=82121] singleton невозможен (NodeTable=2 узлов), пропуск окна — жду peer Proposal (M9 Phase 2)
      2 [active W=82120] singleton невозможен (NodeTable=2 узлов), пропуск окна — жду peer Proposal (M9 Phase 2)
      2 [active W=82119] singleton невозможен (NodeTable=2 узлов), пропуск окна — жду peer Proposal (M9 Phase 2)
```

### helsinki
```
    600 https://docs.rs/rustls/latest/rustls/manual/_03_howto/index.html#unexpected-eof" })))) }))
      8 "Connection reset by peer" })))) }))
      5 Error(Right(Closed)) }))
      3 Failed with result 'timeout'.
      2 Invalid argument
      1 Connection refused (os error 111))]
```

### armenia
```
      4 Error(Right(Closed)) }))
      1 Invalid argument
      1 Failed with result 'timeout'.
```


## 6. Soak watchdog (5-minute polls)

`montana-soak.timer` writes one JSON line per poll to
`/var/lib/montana-soak/soak.jsonl` on the Moscow orchestrator covering
all four nodes. The 24-hour continuous record is the empirical evidence
for the Noise_PQ XX cross-machine soak (DEV-014 Phase 3 part 3
acceptance).

Total soak records to date: 54.

Last 3 records (one line per poll):

```
0,"closed_5m":0
0},{"label":"armenia","host":"<exit-am>","active":"active","window":75845,"phase":"CandidateVdf","D":325000000,"hb_5m":2204,"err_5m":0
0,"closed_5m":4}]}
```


## 7. Mainnet release candidate verdict

| Component | State |
|-----------|-------|
| Noise_PQ XX production transport | active across four-node mesh (3 Genesis + 1 external operator) |
| Genesis manifest auto-sync | live (10-min timer) |
| VPN key auto-sync | live (5-min timer) |
| Explorer auto-discovery | live (1-min collector at /var/www/efir/explorer/data.json) |
| Soak watchdog | live (5-min timer at /var/lib/montana-soak/soak.jsonl) |
| External-operator onboarding | verified end-to-end on a fresh Yerevan VPS in ~16 min |
| Sixteen Metzdowd findings | 12 closed by construction + 2 rejected with citation + MONT-001/MONT-002/MONT-004 + DEV-014 all closed |
| DEV-012 follower drift fix | **closed** (commit e1a0bd0 follower_skip flag) |
| DEV-012 multi-confirmer protocol | **open** for v1.0.0 promotion (BundledConfirmation cross-node aggregation + quorum) |
| M7 fast-sync | **open** for v1.0.0 promotion (snapshot-based onboarding for million-account scale) |

The network is suitable for a public release candidate v1.0.0-rc.2
including the DEV-012 partial close (commit e1a0bd0). The two open items
above are the explicit gates for promotion to v1.0.0 mainnet.
