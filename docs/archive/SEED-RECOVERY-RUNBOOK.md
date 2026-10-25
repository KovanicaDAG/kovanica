# Seed recovery runbook — stalled public testnet

Status: **RUNBOOK ONLY. Nothing here has been executed.** Written 2026-10-15 against
`main` = `d0a26b11`. Companion to `docs/archive/RENAME-CUTOVER-PLAN.md`; that doc owns the Option A
*rename* cutover — this one covers only **restoring quorum**. Facts come from the plan doc's
read-only audit (§1, §2.5) and the tree. Owner gates are marked **[OWNER]** and collected in §9.

---

## 1. Purpose, scope, non-goals

**Purpose.** The public testnet has no quorum and is not advancing. Restore the three-seed
bootstrap set so the commit rule fires again (`committed leader` / `executed subdag`).

**Scope.** seed3 `srv2013143` (seed3.kovanica.online), seed2 `srv1991525` (seed2.kovanica.online), seed1
`srv1745734` (seed1.kovanica.online, **deferred**); Option A wrapper (`kovanica-testnet-seedN.service`,
no `@`), old data dir, port 8000, `validators/v{1,2,3}.dat` (per-host).

**Non-goals — out of scope.**
- **No mainnet**, and **no genesis ceremony or reset**: wiping `state.redb`/`consensus.redb`,
  editing `genesis.json`, or any reset flag is separate and owner-signed (§9 Q1).
  `KOVANICA_ALLOW_RESET` stays `0` on every seed, always.
- **No Option B migration**: `/etc/kovanica/`, `/var/lib/kovanica-testnet-seedN` and the
  `kovanica-testnet-seed@N.service` template are a different project (plan §2 D1).
- **No seed1 cleanup** (5 leaked harness processes, 4-container Docker cluster — plan §2 D5);
  **no open faucet** (`FAUCET=0`, `OPERATOR=0`).

## 2. Safety rules and stop conditions

**Rules**
1. One host per window, in the §4 order.
2. A data dir is only **copied**, never moved or deleted, until a full soak passes.
3. Old units stay **enabled but stopped**, so rollback is one command (§5.4).
4. `KOVANICA_VALIDATOR_KEY` and its `0600` key file ship **together**, every step; a partial
   env rename **fails closed** (`config.rs:225-232`, `main.rs:2258`).
5. `KOVANICA_MINE`/`KVNC_MINE` is read **nowhere** in the crates. Never treat a mine flag as
   the producing gate; the gate is `run_validator` (default `true`, `config.rs:96`) *plus* a key.
6. Peers by **origin IP only**: `seed1.kovanica.online:8000,seed2.kovanica.online:8000,seed3.kovanica.online:8000` —
   never an orange-cloud hostname on TCP 8000.
7. Live P2P port here is **8000** (plan §5); do not import port-9000 rules.
8. Re-confirm the bound ports with `ss -ltnp` before trusting any probe (§7).

**Stop and roll back the current host if** a step's output differs from §5 and is not in §8; a
key file is not `0600` or the node logs `run_validator=true but no validator key`;
`journalctl` shows `refusing to replay`, an `oom-kill` or a fork; two seeds disagree on
`last_executed_round`; free memory < ~2 GB; or anything needs wiping a data dir (§9 Q1).

## 3. Facts (from plan §1 and §2.5)

| | seed1 `srv1745734` | seed2 `srv1991525` | seed3 `srv2013143` |
|---|---|---|---|
| Role | dev box + seed (**deferred**) | live seed | live seed |
| RAM | 15 987 MB | 7 935 MB | 7 945 MB (~4.3 GB free) |
| Old unit | `kvnc-testnet-seed1` enabled / **dead** (launcher missing) | `kvnc-testnet-seed2` **active** | `kvnc-testnet-seed3` enabled / **dead** |
| New unit | `kovanica-testnet-seed@1` disabled | `@2` crash-loop (restart 2131) | `@3` dead after OOM (restart 2749) |
| Launcher | `run-seed1.sh` **absent on host** | `run-seed2.sh` | `run-seed3.sh` |
| Binary | — | `kvnc-node` Oct 13 (14 825 576 B); `kovanica-node` Oct 5 (5 785 656 B) | `kvnc-node` Oct 13; `kovanica-node` Oct 5 |
| Data dir | `/root/Projects/kvnc/testnet/node-data` | same | same (~5.4 MB on seed2) |

Old launcher (per-host): `KVNC_DATA_DIR=.../testnet/node-data`, `KVNC_LISTEN_ADDR=0.0.0.0:8000`,
`KVNC_VALIDATOR_KEY=.../validators/vN.dat` (**seed1→`v1.dat`, seed2→`v2.dat`, seed3→`v3.dat`** — three
distinct 65-byte blobs, mode `0600`; verified on seed2/seed3), `KVNC_MINE=0`, `FAUCET=0`,
`ALLOW_RESET=0`, `OPERATOR=0`, then `exec /usr/local/bin/kvnc-node`. Data dir: `chain_id`,
`consensus.redb`, `state.redb`, `genesis.json`, `genesis_validators.toml` (0600), `p2p_node.key` (0600).

**Stall evidence (seed2, the only live process):** `/health` → `peer_count: 0`,
`executed_subdags: 0`, `last_executed_round: null`; no ESTABLISHED TCP on :8000; 5 414
`Proposed block`, 0 `committed leader`; `state.redb` frozen while `consensus.redb` grows;
public `/api/head` and `/api/bootstrap` both **502** → use per-host loopback probes.

## 4. Recovery order and rationale

**seed3 → seed2 → seed1.** seed3 has **no node process at all** (OOM-killed Oct 08, then
deactivated), so starting it is **zero incremental risk**. seed2 holds the only live process
and its `consensus.redb`, so it moves second, after seed3 proves the wrapper, data dir and
key. seed1 is the dev box with a live OpenCode session → **deferred** (plan §2 D4).

The fix is **availability, not configuration**: seed2 already produces and votes; the network lacks the other validators being reachable so the commit rule can reach quorum (plan §2 D3).

## 5. Procedures

Run each block **one at a time** and inspect the expected output before the next; each `[GATE]` is a hard stop.

### 5.0 Per-host pre-flight (read-only)

```bash
systemctl list-units --all | grep -E 'kvnc|kovanica'
free -m ; df -h /root/Projects/kvnc/testnet
stat -c '%a %n' /root/Projects/kvnc/testnet/validators/vN.dat     # replace N: seed1->v1, seed2->v2, seed3->v3; expect: 600
sha256sum /usr/local/bin/{kvnc,kovanica}-node
journalctl -u kvnc-testnet-seedN.service -n 50 --no-pager
ss -ltnp | grep -E ':8000|:8545|:3001' ; ss -tnp | grep ':8000'
stat -c '%y %s %n' /root/Projects/kvnc/testnet/node-data/{state.redb,consensus.redb}
```

Expect: key `600`; seed3 nothing bound on :8000/:8545/:3001; seed2 the old binary on
`0.0.0.0:8000` + `127.0.0.1:8545` with **zero** established peers. Differs → §8.

```bash
# snapshot before any write (copy, never move)
cp -a /root/Projects/kvnc/testnet/node-data /root/Projects/kvnc/testnet/node-data.snap-$(date +%F)
```

`[GATE]` Snapshot exists. If declined, record that as an owner decision in the log (§5.5).

### 5.1 seed3 — cold start (first)

1. `systemctl is-active kvnc-testnet-seed3.service` → `inactive`; `is-enabled` → `enabled`.
2. Stage Option A (plan §4): versioned renamed binary, `run-seed3.sh` with `KOVANICA_*`, and `kovanica-testnet-seed3.service` (`MemoryMax=5G`, §6); **not** `...seed@3.service`.
3. Dry-run: `bash -n run-seed3.sh`; `systemd-analyze verify ...seed3.service`.
4. Configure `KOVANICA_VALIDATOR_KEY=/root/Projects/kvnc/testnet/validators/v3.dat` (seed3; use `v2.dat` for seed2, `v1.dat` for seed1), `chmod 0600` it; `systemctl daemon-reload`.
5. `systemctl start kovanica-testnet-seed3.service`
6. `systemctl is-active kovanica-testnet-seed3.service` → `active`.
7. `journalctl -u kovanica-testnet-seed3.service -n 50 --no-pager` → no replay refusal / OOM.
8. Probes (§7): `ss -ltnp | grep :8000`; `curl -s http://127.0.0.1:8545/health | jq`.

`[GATE]` Stays `active` for 5 min and `state.redb` mtime advances (or §8 explains why).

### 5.2 seed2 — replace the running pre-rename node

1. Baseline: `curl -s http://127.0.0.1:8545/health | jq '.peer_count,.consensus,.execution'`.
2. `systemctl stop kvnc-testnet-seed2.service` → `is-active` → `inactive`.
3. `[GATE]` `ss -ltnp | grep :8000` empty.
4. Re-confirm the `0600` key; `systemctl daemon-reload`; `systemctl start kovanica-testnet-seed2.service`.
5. Verify as §5.1 steps 6–8, on seed2.
6. Green → leave the old unit enabled-but-stopped; do **not** disable it yet.

`[GATE]` seed2 and seed3 see each other: `peer_count >= 1` on seed3, `>= 2` on seed2 once both
are up (quorum TBD — §9 Q4).

### 5.3 seed1 — deferred

Do **not** run in this window. `kvnc-testnet-seed1.service` is orphaned (`run-seed1.sh` is
absent on that host, tracked only on `branch network/testnet`) and the box is a live dev
session. Its own window must first restore the launcher on `main` (plan §1 defect 1, §9 Q7).

### 5.4 Rollback (per host, rehearse before §5.2)

```bash
systemctl stop  kovanica-testnet-seedN.service
systemctl start kvnc-testnet-seedN.service
systemctl is-active kvnc-testnet-seedN.service
```

One command **only while** the old unit, old data dir and old `kvnc-node` all exist. For seed3,
rollback is just stopping the new unit (the old one was already dead). **Never delete**
`node-data/`, the `*.snap-*` copy, or `/usr/local/bin/kvnc-node` until a full soak.

### 5.5 Operator log — record per step

Host · unit · UTC time · what changed · why · git SHA + `sha256sum` of the binary started ·
snapshot path · before/after `/health` + `state.redb` mtime · follow-ups — the only audit trail, since the public API is 502.

## 6. OOM / capacity guard

The new-gen template's `MemoryMax=3G` is **below the projected replay peak** (~3 368 MiB and
growing) → seed2's replay refusal and seed3's OOM kill. Per plan §2 D2 the renamed units get **`MemoryMax=5G`** (seed2 has ~6.1 GB free and an unused mainnet instance; 5G fits today).

```bash
systemctl show -p MemoryMax -p MemoryCurrent kovanica-testnet-seedN.service
```

`MemoryCurrent` climbing toward `MemoryMax` is the early warning. This **buys time only** —
pruning is disabled (RFC-009), so replay memory grows with the chain and the ceiling recurs;
the durable fix (pruning / snapshot resync) is a separate workstream.

## 7. Quorum verification and what "green" looks like

Probes are per-host loopback. The node's HTTP surface is only `/health`, `/metrics`, `/rpc` and
`/ws` (`crates/kovanica-rpc/src/lib.rs:307-327`); there is **no `/api/*` route in this tree**, so
`/api/head` belongs to the separate explorer surface (currently 502 — §3). Use `/health` plus a
JSON-RPC call (`kovanica_blockNumber` via `POST /rpc`), and confirm bound ports with `ss -ltnp`.
A missing `/api/head` is **not** a node fault.

```bash
curl -s http://127.0.0.1:8545/health | jq '.healthy,.peer_count,.consensus,.execution'
curl -s -X POST http://127.0.0.1:8545/rpc -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"kovanica_blockNumber","params":[]}'   # tip height
ss -tnp | grep -c ':8000'                                         # established peers
stat -c '%y' /root/Projects/kvnc/testnet/node-data/state.redb     # read twice, 1 min apart
```

**Green per host:** unit `active`; last 50 journal lines clean; `healthy: true`;
`peer_count >= 2`; `committed_leaders` and `executed_subdags` **increase** across two reads a
minute apart; `last_executed_round` non-null; `state.redb` mtime advances.

**Green for the network:** every started seed `active` AND the hosts **agree** on `last_executed_round`/committed count; height monotonic. Disagreement is a fork → §2.

## 8. Failure modes

| Symptom | Likely cause | Action |
|---|---|---|
| `refusing to replay … projected replay peak … exceeds limit` | `MemoryMax` below replay peak | confirm `MemoryMax=5G` (§6); resync is the durable fix, not a bigger number |
| `oom-kill` / dead after start | same, or host pressure | free RAM, watch `MemoryCurrent`, roll back per §5.4 |
| `peer_count: 0`, no ESTABLISHED :8000 | bootnodes wrong/absent, or peers down | verify `KOVANICA_BOOTNODES` = origin IPs:8000 (§2 rule 6); confirm other seeds up |
| `run_validator=true but no validator key` / refuses start | partial env rename, or key not `0600` | ship env + `0600` key together (§2 rule 4) |
| Binary rejects blocks / wrong chain | authority set differs from the running chain | STOP; do not reset — §9 Q4/Q5 |
| Two seeds disagree on tip/round | fork | STOP both new units, inspect journals, escalate — §9 Q1 |
| seed3 replay refuses / rejects seed2's chain | stale or divergent seed3 data dir | STOP; resync vs reset is an owner action — §9 Q2 |
| `/api/head` 404 / refused (explorer surface, not the node) | the explorer is a separate service and is down today (502) | expected; probe `/health` + `kovanica_blockNumber` via `/rpc` + `state.redb` mtime |
| `address already in use` on :8000 | stale process holds the port | find with `ss -ltnp`, stop the correct unit; never kill seed1's D5 processes |

## 9. Owner decisions required (do not pass a blocked gate)

| # | Question | Why it blocks |
|---|---|---|
| Q1 | Does the stalled chain **resume** or need a **reset**? | A reset is genesis-class; this runbook assumes resume only |
| Q2 | Is seed3's `node-data` **current, empty, or fork-divergent**? | A stale dir means seed3 cannot join by plain start |
| Q3 | Which **SHA** is the renamed binary built from? (plan §3 cites `0d1b114c`; `main` is now `d0a26b11`, with uncommitted `kovanica-{node,rpc,types,consensus}` changes) | Wire-mismatch risk; drives the logged `sha256sum` |
| Q4 | ✅ **ANSWERED (evidence §3)** — the genesis set is 3 validators (50 000 KUNA each in `genesis_validators.toml`, identical on seed2/seed3). `/etc/kovanica/testnet-authorities.conf` is a **different** 3-key PoA set (`d1a14d2c…`, `a5e261ae…`, `a1affed9…`, `KOVANICA_AUTHORITY_THRESHOLD=2`, `KOVANICA_SLOT_DURATION=3000`) | the two generations name different validator sets — confirm the **live chain's own** threshold before declaring quorum; this is why the paths must not be mixed |
| Q5 | ✅ **ANSWERED — favourable.** Per-host, distinct identities: seed1→`v1.dat`, seed2→`v2.dat`, seed3→`v3.dat` (three distinct 65-byte blobs: `5e46da92…`, `23f84da4…`, `823f2e11…`; verified on-host on seed2/seed3, mode `0600`) | closed — quorum is achievable; **never** reuse one key across seeds |
| Q6 | ✅ **ANSWERED.** The node exposes no `/api/*` route at all (`router()` = `/rpc`, `/ws`, `/health`, `/metrics`); `/api/head` is the explorer surface (502 today). RPC defaults to `127.0.0.1:8545` (`config.rs:79-103`) | closed — probe `/health` + `kovanica_blockNumber` via `POST /rpc`; confirm with `ss -ltnp` |
| Q7 | **seed1 window** and confirmation D5's processes are left alone? | seed1 is the dev box with a concurrent session |
| Q8 | Snapshot **storage capacity** for the `*.snap-*` copies? | No snapshot ⇒ no verified rollback |

---

## 10. Related documents

- **`docs/archive/RENAME-CUTOVER-PLAN.md`** — the Option A rename cutover itself (D1–D5, phases,
  rollback, risks). Use its §5 for rename mechanics, this doc for the recovery/quorum half.
- `TASKLIST.md` §B 471 — the `KVNC_*` rename-drift item this workstream closes.
- `docs/TESTNET-SEED-NODES.md`, `docs/TESTNET-RESET.md` — historical, partly superseded.
