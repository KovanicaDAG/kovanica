# Testnet rename cutover + seed recovery — rollout plan

Status: **PLAN for the cutover itself — the cutover has NOT run.** Every statement in §1–§8 is from a
read-only audit performed 2026-10-15 unless marked otherwise. Two bounded, owner-authorised actions
were executed on 2026-10-16 — see §9, which also supersedes several §1 / §2.5 figures.

Owner sign-off gates are marked **[OWNER]**; each must be answered before its phase runs.

> **Re-audited read-only across all three hosts on 2026-10-16.** Headline: the testnet is **no longer
> stalled** — a self-contained three-validator network now runs on seed1 under Docker and commits every
> ~12 s — but **none of the three seeds is connected to it**, and the bootnode the shipped compose
> advertises (`testnet.kovanica.online:8000`) has **no listener**. Read §9 before acting on §1–§8.

---

## 1. What the audit actually found

The live fleet runs **two generations of deployment side by side**. The `kvnc-*` generation is
what carries the traffic; the `kovanica-*` generation is staged but not viable.

| | seed1 `srv1745734` (seed1.kovanica.online) | seed2 `srv1991525` (seed2.kovanica.online) | seed3 `srv2013143` (seed3.kovanica.online) |
|---|---|---|---|
| Role | **this dev box** | live seed | live seed |
| RAM | 15 987 MB | 7 935 MB | 7 945 MB |
| Old (`kvnc-*`) unit | `kvnc-testnet-seed1` enabled / **dead** | `kvnc-testnet-seed2` **active, serving** | `kvnc-testnet-seed3` enabled / **dead** |
| New (`kovanica-*`) unit | `kovanica-testnet-seed@` disabled | `kovanica-testnet-seed@2` **crash-loop** (restart 2131) | `kovanica-testnet-seed@3` **dead** (OOM-killed Oct 08, restart 2749) |
| `/etc/kovanica/*` | present | present | present |
| `/usr/local/bin/kovanica-node` | — | Oct 5, 5 785 656 B | Oct 5, 5 785 656 B |
| `/usr/local/bin/kvnc-node` | — | Oct 13, 14 825 576 B | Oct 13, 14 825 576 B |

Old-generation launcher (identical shape on all three hosts) — `testnet/run-seedN.sh`:

```
KVNC_DATA_DIR=/root/Projects/kvnc/testnet/node-data
KVNC_LISTEN_ADDR=0.0.0.0:8000
KVNC_BOOTNODES=seed1.kovanica.online:8000,seed2.kovanica.online:8000,seed3.kovanica.online:8000
KVNC_VALIDATOR_KEY=/root/Projects/kvnc/testnet/validators/v2.dat
KVNC_MINE=0 / FAUCET=0 / ALLOW_RESET=0 / OPERATOR=0
exec /usr/local/bin/kvnc-node
```

New-generation unit `kovanica-testnet-seed@.service` (template, sandboxed, `MemoryMax=3G`):

```
EnvironmentFile=/etc/kovanica/testnet-seed%i.env            # 12 keys, all KOVANICA_*
EnvironmentFile=/etc/kovanica/testnet-authorities.conf      # public authority set
EnvironmentFile=/etc/kovanica/testnet-authority-%i.env      # this seed's signing key
ExecStart=/usr/local/bin/kovanica-node explorer 127.0.0.1:3001
ReadWritePaths=/var/lib/kovanica-testnet-seed%i
```

`/etc/kovanica/testnet-seed2.env` (non-secret values, verbatim):

```
KOVANICA_NETWORK=kovanica-testnet   KOVANICA_CONSENSUS=poa
KOVANICA_LISTEN=0.0.0.0:8000        KOVANICA_PEERS=seed.kovanica.online:8000,seed3.kovanica.online:8000
KOVANICA_DATA=/var/lib/kovanica-testnet-seed2
KOVANICA_PRODUCE=1                  KOVANICA_PRODUCE_SECS=3
KOVANICA_ISOLATED_HOST=1            KOVANICA_METRICS_LISTEN=127.0.0.1:9090
KOVANICA_OPERATOR=0 / FAUCET=0 / ALLOW_RESET=0
```

### The two failures are NOT rename failures

```
kovanica-node: explorer failed: refusing to replay /var/lib/kovanica-testnet-seed2/alpha.log for node alpha:
projected replay peak is ~3368 MiB but the memory limit is 3072 MiB.
Block pruning is disabled (RFC-009), so replay memory grows with the chain.
```
- **seed2**: same message every 30 s, restart counter 2131 (~17.7 h) — `MemoryMax=3G` is below the
  projected replay peak. seed2 has 6 168 MB available and its `mainnet-seed2` data dir is empty.
- **seed3**: last new-unit death was an **OOM kill** (`Consumed 3G memory peak`), then it was
  deactivated on Oct 08 and never restarted. seed3 has **no node process at all** — that seed is down.
- Note: the failure message is emitted by the *renamed* binary, so the rename itself works.

### Three further defects found while auditing

1. **seed1 is the dev box and is unmanaged.** `kvnc-testnet-seed1.service` points at
   `/root/Projects/kvnc/testnet/run-seed1.sh`, which **does not exist** on seed1 (that script is only
   tracked on branch `network/testnet`, not on `main`) → the unit is orphaned and cannot start.
   What actually runs on seed1 today is unmanaged by systemd:
   - 5 × `/root/Projects/kvnc/target/debug/kvnc-node` (ppid 1, ~30 h) with `KVNC_DATA_DIR=/tmp/.tmpXXXX`
     and `KVNC_BOOTNODES=127.0.0.1:<random ports>` → leaked test-harness clusters, not a seed;
   - 4 Docker containers `kvnc-testnet-{seed,val1,val2,val3}` (image `kvnc-node:local`, healthy, 12 h,
     RPC 127.0.0.1:8545–8548, P2P 9000) → a **local** testnet started from the **pre-rename** compose
     file (the committed compose names them `kovanica-node:local` / `kovanica-testnet-*`).
2. **`kovanica-seed1.service`** (a third, PoA-flavoured generation) needs
   `/root/kovanica-testnet/authority-keys/`, which does not exist on seed1 → also unstartable.
3. **The new path changes network semantics, not just names.** `KOVANICA_CONSENSUS=poa` with
   `KOVANICA_PRODUCE=1/PRODUCE_SECS=3` plus an authority set from `/etc/kovanica/`, reading a
   *different* data directory (`/var/lib/kovanica-testnet-seedN`) than the running chain
   (`/root/Projects/kvnc/testnet/node-data`). This is a consensus/validator-set change dressed as a rename.

---

## 2. Decisions required before anything runs

**[OWNER] D1 — Which change is actually wanted?** These are two different projects:

- **Option A — rename only (low risk).** Keep the *current* deployment semantics
  (`/root/Projects/kvnc/testnet/node-data`, port 8000, `validators/v2.dat`, non-producing seeds) and
  rename the wrapper: `KVNC_*`→`KOVANICA_*`, `kvnc-node`→`kovanica-node`, `kvnc-*`→`kovanica-*` units.
  Wire/consensus identical; rollback is trivial. Does **not** use `/etc/kovanica/` or the `@` template.
- **Option B — generation migration (high risk).** Move onto the `/etc/kovanica` + `/var/lib` + PoA
  layout that the `kovanica-*` units already encode. Requires data migration, authority-set review,
  and the memory fix in D2. This is a network-wide cutover, not a rolling restart.

Recommendation: do **A** first (it removes the silent-config-break drift and is reversible), then treat
**B** as a separate, scheduled migration with its own dry run. Do not mix them in one window.

> **DECIDED — owner, 2026-10-15: Option A (rename only).** Only the rename half is in scope for this
> rollout. Consequence: this rollout renames the *existing* wrapper units
> (`kvnc-testnet-seedN.service` → `kovanica-testnet-seedN.service`) and their env/launcher, keeping
> `/root/Projects/kvnc/testnet/node-data`, port 8000 and `validators/v2.dat`. The `kovanica-testnet-seed@`
> template plus `/etc/kovanica/` + `/var/lib/kovanica-testnet-seedN` are **not** used by this rollout;
> Option B stays as a separate scheduled migration with its own dry run.

> **D2–D5 are now decided too** (owner, 2026-10-15) — see the blockquotes below. D2 is partly an
> Option-B-only concern (the rename-only path does not apply `MemoryMax=3G`), but seed3 must still come
> back without an OOM.

**[OWNER] D2 — Memory ceiling.** The new units cannot start at `MemoryMax=3G` (projected peak
3 368 MiB and growing; pruning disabled per RFC-009). Options:
  - raise the ceiling (seed2 has 6 168 MB free and its mainnet instance is unused → 5G is feasible
    today) — but this recurs as the log grows;
  - wipe + resync (`mv alpha.log alpha.log.broken`, then resync from a peer on :8000) — the binary's
    own suggested recovery; also temporary;
  - snapshot-restore — **not yet verified to exist** for this network; needs confirmation;
  - revisit pruning / RFC-009 — a design change, out of scope for a rollout.
  Recommendation: unblock with a raised ceiling **and** open a separate workstream for the durable fix.

> **DECIDED — owner, 2026-10-15: raise `MemoryMax` to `5G` on the renamed units.** seed2 has 6 168 MB
> free and its unused mainnet instance, so 5G fits today. This only buys time — the ceiling recurs as
> the log grows, so the durable fix (pruning / RFC-009, or a snapshot-based resync) stays a separate
> workstream and is not in this rollout.

**[OWNER] D3 — Authority set / producer set.** Confirm that `/etc/kovanica/testnet-authorities.conf`
and the `PRODUCE=1` flags describe the intended validator set. If the new authority set differs from
the one on the running chain, the cutover forks or the new nodes reject the chain.

> **DECIDED — owner, 2026-10-15: the seeds should be *producing* validators** (not the "seeds stay
> non-producing" recommendation). Implementation notes for Option A — "producing" is already the code
> default: `run_validator` defaults to `true` (`crates/kovanica-node/src/config.rs:96`) and the real gate
> is a validator key, not a mine flag. `KVNC_MINE` / `KOVANICA_MINE` is read **nowhere** in the crates,
> so the old launchers' `KVNC_MINE=0` was a no-op — which is precisely why seed2 proposes and votes
> despite it. The renamed units must therefore set `KOVANICA_VALIDATOR_KEY` (read at `config.rs:156`,
> validated at `config.rs:225-232`) to a `0600` seed file, or the renamed binary **refuses to start**
> (`crates/kovanica-node/src/main.rs:2259`). This is stricter than the old path: a *partial* env rename
> is not silent — it fails closed. Practical consequence for D4: seed2 is already producing; what the
> network lacks is *availability*, not configuration.

> **Evidence (2026-10-15, read-only on seed2/seed3): the two generations name different validator sets.**
> The old path signs with per-host keys `testnet/validators/v{1,2,3}.dat` (mode `0600`, three distinct
> 65-byte blobs) against the genesis set in `testnet/node-data/genesis_validators.toml` (`aa7ec9c9…`,
> `7173710d…`, `85043fd3…`, 50 000 KUNA each — identical on both hosts).
> `/etc/kovanica/testnet-authorities.conf` instead declares `d1a14d2c…`, `a5e261ae…`, `a1affed9…` with
> `KOVANICA_AUTHORITY_THRESHOLD=2` and `KOVANICA_SLOT_DURATION=3000`. The two key sets do not overlap,
> so D3 is not academic: mixing the paths puts a node on a set the running chain does not recognise.
> Option A must stay on the genesis set.

**[OWNER] D4 — Window and blast radius.** The three seeds are the testnet's bootstrap set. Recommend
seed3 first (already down → zero incremental risk), then seed2, then seed1 (dev box, deferred).

> **DECIDED — owner, 2026-10-15: accepted (seed3 → seed2 → seed1).** Seed1 goes last and stays deferred
> while the concurrent OpenCode session holds the shared cwd.

**[OWNER] D5 — What to do with the seed1 unmanaged processes and the pre-rename Docker cluster**
(kill/recreate under the renamed compose, or leave alone).

> **DECIDED — owner, 2026-10-15: leave them alone for now.** The 5 leaked `target/debug/kvnc-node`
> harness processes and the 4-container pre-rename Docker cluster are out of scope for this rollout and
> must **not** be killed as a side effect of the cutover. Revisit as its own item.

---

## 2.5 ⚠️ Prerequisite found 2026-10-15: the public testnet is stalled — and not because of the rename

Read-only evidence collected the same day. **The network is not advancing today,** so Phase 3's
"tip advancing / peers ≥ 2" acceptance criteria cannot be satisfied by a rename alone.

**seed2 is running, alone, and has never committed anything.** The process is the *pre-rename*
`/usr/local/bin/kvnc-node` (PID env still carries `KVNC_VALIDATOR_KEY` / `KVNC_*`), started ~Oct 14
13:10:

| Probe | Result |
|---|---|
| `curl 127.0.0.1:8545/health` | `peer_count: 0`, `executed_subdags: 0`, `last_executed_round: null`, `consensus: running` |
| `ss -ltnp` | only `127.0.0.1:8545` + `0.0.0.0:8000` — **no established TCP on :8000, zero peers** |
| journal | **5 414 × `Proposed block`**, **0 ×** `committed leader` / `executed subdag` / `commit rule` |
| `state.redb` | 2 113 536 B, mtime **Oct 14 13:27** — untouched since startup |
| `consensus.redb` | 6 844 416 B, mtime **Oct 15 07:13** — still growing |

So the node proposes endlessly into its own DAG and never reaches the commit rule: it has **no quorum**.
The other two genesis validators are unreachable — seed3 has no node process at all (OOM-killed Oct 08,
then deactivated) and seed1 is not serving (nothing listening on 3000/3001/8080; `pm2 ls` empty; only
system nginx on 80/443). The public surface agrees: `https://explorer.kovanica.online/api/head` and
`/api/bootstrap` both return **HTTP 502**.

**Therefore:**

1. Option A (rename only) preserves the *current semantics* but does **not** fix the stall. The cutover
   window should be combined with — or immediately preceded by — bringing the other validators up and
   interconnected, otherwise Phase 3 cannot go green.
2. Restoring the other two seeds is the higher-priority workstream; the rename is the smaller,
   reversible half.
3. The 502 on the explorer means "verify against the network" steps must temporarily use per-host
   loopback probes instead of the public API. The node's own HTTP surface is only `/rpc`, `/ws`,
   `/health` and `/metrics` (`crates/kovanica-rpc/src/lib.rs:307-327`) — there is **no** `/api/*` route
   in this tree, so `/api/head` belongs to the separate explorer service. Probe `/health`, plus the
   JSON-RPC method `kovanica_blockNumber` via `POST /rpc`.

---

## 3. Phase 0 — pre-flight (read-only, no host changes)

1. Freeze: no other session or operator touching the three hosts (the dev-box cwd is shared with
   another live OpenCode session today).
2. Capture a baseline per host:
   `systemctl list-units --all | grep -E 'kvnc|kovanica'`, `free -m`, `df -h /var/lib`,
   `sha256sum /usr/local/bin/{kvnc,kovanica}-node`, `journalctl -u <unit> -n 100`.
3. Snapshot every data dir before touching it: `/root/Projects/kvnc/testnet/node-data` (old path) and
   `/var/lib/kovanica-testnet-seed{1,2,3}` (new path). Copy, do not move.
4. Record the exact git SHA the deployed binaries were built from; build the cutover binary from a
   **pinned, pushed** SHA (currently `main` = `d0a26b11`) and record its `sha256sum`. Build it from a
   **clean worktree** at that SHA: the shared working tree currently carries 12 modified foreign-WIP
   files, so an in-tree build would not correspond to any committed revision.
5. Confirm the old units are `enabled` (they are) so rollback needs no re-configuration.

## 4. Phase 1 — build and stage artefacts (no host changes)

1. Build the release node binary from the pinned SHA; verify `kovanica-node --version` / `--help`.
2. Stage, per host, **without** activating:
   - the binary to a versioned path (e.g. `/usr/local/bin/kovanica-node-<sha>`);
   - the chosen unit set (`kovanica-*`, typed per D1);
   - the env files, with `chmod 0600` on anything holding a key.
3. Diff old vs new env key-by-key to prove the rename is the *only* semantic difference for Option A
   (`KVNC_X` → `KOVANICA_X`), or enumerate the intended differences for Option B.
4. Dry-run: `systemd-analyze verify` on every unit file; `bash -n` on every script.

## 5. Phase 2 — cut over one host at a time (seed3 → seed2 → seed1)

Per host, in a maintenance window:

```bash
# 1. stop the old path WITHOUT disabling it (keeps rollback one command away)
systemctl stop kvnc-testnet-seedN.service

# 2. (Option B only) migrate data, if D1=B and D2 chose migration over resync
#    cp -a /root/Projects/kvnc/testnet/node-data/. /var/lib/kovanica-testnet-seedN/

# 3. start the new path
systemctl start kovanica-testnet-seed@N.service

# 4. verify (see Phase 3) BEFORE disabling the old unit
# 5. only when green: systemctl disable kvnc-testnet-seedN.service
```

Verification after each host, before moving to the next:

```bash
systemctl is-active kovanica-testnet-seed@N.service      # expect: active
journalctl -u kovanica-testnet-seed@N.service -n 50      # expect: no replay-refusal / OOM
curl -s http://127.0.0.1:8545/health | jq '.healthy, .peer_count, .execution'
curl -s -X POST http://127.0.0.1:8545/rpc -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"kovanica_blockNumber","params":[]}'   # tip height
ss -ltnp | grep -E ':8000|:8545'                         # expect the P2P + loopback RPC binds
```

The node serves no `/api/*` route: `/rpc` (POST), `/ws`, `/health` and `/metrics` only
(`crates/kovanica-rpc/src/lib.rs:307-327`). The public explorer (`https://explorer.kovanica.online`
`/api/head`, `/api/bootstrap`) is a **separate service** and is currently returning 502, so cross-check
it only as a secondary signal. **Port note:** this network runs P2P on **8000** (not 9000) and RPC on
loopback **8545**; re-confirm with `ss -ltnp` before hard-coding either.

## 6. Phase 3 — post-cutover verification

- all three seeds `active`, peer counts ≥ 2, tip advancing, `kovanica_blockNumber` monotonic (via
  `POST /rpc` — the node exposes no `/api/*` route);
- `grep -rn 'KVNC_' /etc/systemd/system` → empty on every host (the drift in TASKLIST §B 471);
- no unit references `/root/Projects/kvnc` any more, or the dependency is documented;
- the old units remain `disabled` but present for one release cycle before deletion;
- update `TASKLIST.md` §B 471 as resolved, with the per-host evidence and the memory workstream.

## 7. Rollback (rehearse before Phase 2)

```bash
systemctl stop  kovanica-testnet-seed@N.service
systemctl start kvnc-testnet-seedN.service     # old unit + old data dir untouched
systemctl is-active kvnc-testnet-seedN.service
```

Rollback is only trivially safe while the old data directory and old binary are still present — do not
delete or overwrite either until the new path has been green for a full soak.

## 8. Open risks / unknowns

| Risk | Impact | Handling |
|---|---|---|
| Memory ceiling is a moving target (pruning disabled, RFC-009) | seeds die again as the chain grows | separate durable workstream; D2 |
| Authority set may differ from the running chain | fork / rejection at cutover | D3, verify before Phase 2 |
| seed1 doubles as the dev box and has an active concurrent session | cutover on seed1 could disrupt in-flight work | defer seed1; D4 |
| Unmanaged processes + pre-rename Docker cluster on seed1 | incompletely-documented traffic source | D5 |
| Two docs disagree on the seed domain (`.io` vs `.online`) | operator confusion | fix in a docs-only commit |
| `seed-lint` reports a false "no data path" on these scripts | masks real findings | tool nit: it greps for `KOVANICA_DATA=` while the node reads `KOVANICA_DATA_DIR` |
| **Testnet is stalled right now: seed2 alone, 0 peers, 0 executed subdags, explorer 502** | the rollout cannot be validated; the network is not advancing | §2.5 — bring the other validators up first |
| Renamed binary fails closed without `KOVANICA_VALIDATOR_KEY` | a partial env rename prevents the node from starting | intended behaviour; always ship env + `0600` key file together (§2 D3) |

---

### Related, already-fixed in this session
- Bootnode typo `seed3.kovanica.online` → `seed3.kovanica.online` in `run-seed1.sh`, `mainnet/run-seed1.sh`,
  `mainnet/run-seed2.sh` (commit `e316d70a`).
- Android `libkovanica_light_ffi.so` rebuilt; stale `libkvnc_light_ffi.so` removed.

---

## 9. Update 2026-10-16 — re-audit, corrections, executed actions

All of §9.1–§9.2 and §9.4 comes from a read-only sweep of all three hosts plus seed1's own Docker
surface. No secret was read or printed. §9.3 lists the only two writes, both owner-authorised.

### 9.1 Headline: the chain is advancing, and it does not need the three seeds

The public RPC `https://rpc-testnet.kovanica.online/rpc` is nginx `location /rpc` →
`http://127.0.0.1:8545/rpc` → docker-proxy → container `kovanica-testnet-seed`, i.e. it is served by
**seed1 itself**. Measured:

- 4/4 containers `Up (healthy)` — `kovanica-testnet-{seed,val1,val2,val3}`, image `kovanica-node:local`,
  RPC published on loopback 8545 / 8546 / 8547 / 8548 only;
- 3/3 validators `active: true`, 50 000 KUNA each; committed-leader rotation healthy (see 9.2);
- `kovanica_blockNumber` = `0x40d` (1037) and advancing ≈ 12 s per committed leader;
- no `ERROR` / `ConflictingDecision` / `engine stopped` / `stalled` in the recent journal of any node.

The "single leader forever" defect behind the earlier stall was a **`u128` saturation in the
VRF leader priority** (`hash × total_stake / stake` pinned every authority to `u128::MAX`, so the
lowest committee index won every round and the others were eventually marked offline by
`advance_validator_liveness`). Fixed with an exact 192-bit cross-multiplied comparison and merged to
`main` (merge `d56ad8a4`, pushed as `b4b7eb65` and `e7e8d48d`), together with a round-counter resume
on restart. **This is why §2.5's "the network is not advancing" is superseded.**

Consequence: **Phase 3 cannot be "all three seeds green".** The live chain is carried by seed1's Docker
cluster, not by the three seed hosts. Restate Phase 3 against `rpc-testnet.kovanica.online` (§9.6).

### 9.2 Corrections to §1 and §2.5 (measured 2026-10-16)

| Claim in this doc | 2026-10-16 measurement |
|---|---|
| seed2 new unit "crash-loop (restart **2131**)" | counter had reached **5 612**; the unit is now **stopped and disabled** (§9.3) |
| §1 defect 1 — "4 Docker containers `kvnc-testnet-*`, image `kvnc-node:local`" | **stale**: the committed compose names them `kovanica-testnet-*` with image `kovanica-node:local` |
| seed3 "OOM-killed Oct 08, restart 2749" | that is the **new** `@3` unit, whose journal is now empty. The **old** `kvnc-testnet-seed3` unit was **deliberately stopped on 2026-10-14 04:44** (`received SIGTERM` → `KVNC node stopped cleanly` → `Deactivated successfully`), not killed. |
| §2.5 "seed2 … **0 peers**, never committed" | **still true**: `/health` → `peer_count: 0`, `executed_subdags: 0`, `last_executed_round: null`; since 2026-10-14 13:00 the journal holds **14 336 × `Proposed block`** and **0 × `committed leader`** |
| D5 — "leave the 5 leaked `target/debug/kvnc-node` harness processes alone" | **superseded**: the owner authorised killing them on 2026-10-16; done (§9.3). The pre-rename Docker cluster is likewise no longer "pre-rename". |
| §5 port note — "P2P on **8000** (not 9000)" | **confirmed**: `KOVANICA_LISTEN_ADDR=0.0.0.0:8000` in the running seed container. The `9000/tcp` in `docker ps` is only the image's EXPOSE, not the bind. |

seed2's old node is healthy-looking but useless: it proposes every 12 s into its own DAG and never
reaches the commit rule, because the other two genesis validators are unreachable.

### 9.3 Executed 2026-10-16 (owner-authorised, bounded)

1. **seed2 (`seed2.kovanica.online`): `kovanica-testnet-seed@2.service` stopped and disabled.**
   - before: `activating` / `auto-restart`, `NRestarts=5612`, failing every 30 s
   - after: `inactive` / `dead`, `disabled`; the `multi-user.target.wants` symlink was removed
   - the **old** `kvnc-testnet-seed2.service` was deliberately **not touched** — still `active`,
     `MainPID 42897`
   - rollback: `systemctl enable --now kovanica-testnet-seed@2.service` (it will fail again with the
     replay refusal until D2's 5 G ceiling is actually applied)
2. **seed1 (this host): 5 leaked harness processes killed.** PIDs `1917148`, `1920433`, `1934576`,
   `1934854`, `1934996` — all `/root/Projects/kvnc/target/debug/kvnc-node`, `ppid 1`, ~2.5 days old,
   SIGTERM'd. 10 loopback ports released (20130/20134, 23042/23046, 23066/23070, 24146/24150,
   28202/28206). The Docker cluster was **not** touched; the devnet was verified healthy afterwards
   (`0x40d` and 4/4 containers healthy).

### 9.4 New findings not in §1–§8

1. **The shipped compose advertises a dead bootnode.** `docker-compose.yml` sets
   `KOVANICA_BOOTNODES: "testnet.kovanica.online:8000"` for the seed container and
   `"testnet.kovanica.online:8000,seed:8000"` for the validators. But:
   - `testnet.kovanica.online` → `seed1.kovanica.online` (seed1, grey-cloud);
   - the compose publishes **only** `127.0.0.1:8545–8548`; the container's own `0.0.0.0:8000` bind is
     never published to the host, and `ss -ltnp` on seed1 shows **nothing** on 8000;
   - there is **no nginx `stream` block** and no `conf.d` — nginx is HTTP-only on 80/443.

   So every node launched from the committed compose is told to bootstrap from a port that has no
   listener. In practice the Docker validators only ever connect via the compose-internal alias
   `seed:8000`. Any third-party node following the documented bootnode cannot join.
2. **`seed1.kovanica.online:8000` has no listener either**, for the same reason. Of the three
   advertised DNS bootnodes, **only seed2** has a live P2P bind — and seed2 speaks the old
   libp2p/gossipsub transport, which the renamed node cannot use.
3. **The units and their env are not version-controlled.** There is **no** `*.service`, `*.env` or
   `*.conf` anywhere under `deploy/`, `ops/` or `docs/`. Nothing in the repo produces
   `/etc/kovanica/testnet-seed%i.env`, `testnet-authorities.conf` or `testnet-authority-%i.env`. The
   template's own comment points at `deploy/testnet/configs/seed%i.env` — **that path does not exist in
   the repo**. Until these are committed (key values supplied out-of-band), the migration cannot be
   reviewed or reproduced and Phase 1 step 2 has no source of truth.
4. **Template port bug.** `ExecStart=/usr/local/bin/kovanica-node explorer 127.0.0.1:3001` hard-codes
   the HTTP port **3001** and never uses `%i`, even though the template exists precisely to vary per
   instance (`ReadWritePaths=/var/lib/kovanica-testnet-seed%i` does use it). Two instances on one host
   would collide on 3001. Intended shape is probably `127.0.0.1:30%i`, or the port should come from
   `testnet-seed%i.env`.
5. **`/etc/systemd/system` is already `KVNC_`-clean**, so §6's `grep -rn 'KVNC_' /etc/systemd/system`
   acceptance check is a false-negative generator: the `KVNC_*` variables live in the **launcher
   scripts** (`testnet/run-seedN.sh`), not the unit files. Extend the grep to those scripts.
   Units that do reference the repo path today: `/etc/systemd/system/kvnc-node.service` and
   `/etc/systemd/system/kvnc-testnet-seed2.service`.
6. **There is at least a third legacy unit name in play**: `kvnc-node.service` (disabled, present on
   both seed2 and seed3, also pointing at `/root/Projects/kvnc`). §1's two-generation inventory is
   incomplete.
7. **RFC-009 / pruning bites even at a 3.2 MB log.** The refusal projected ~3 368 MiB from
   `/var/lib/kovanica-testnet-seed2/alpha.log`, which is 3.2 MB and was last written **2026-10-06**.
   The new-path data dir on seed2 is stale and dirty — a wipe-and-resync candidate, not a migration
   candidate. The ceiling will recur as the chain grows, so D2's 5 G only buys one growth cycle.
8. **Memory headroom (for D2)**: seed2 = 7 935 MB total / 6 176 MB available / 8 191 MB swap;
   `/var/lib` on `/` with 69 G free. A 5 G ceiling fits.
9. **`docs/TESTNET-SEED-NODES.md` is now wrong on almost every line** — it claims "P2P TCP 8000,
   libp2p swarm", `chain_id 1337`, "deployed 2026-10-13 (build c44f672)", all three seeds healthy with
   `peer_count 2`, `systemctl start kovanica-node.service` (no such unit), data dir
   `/root/Projects/kvnc/node-data` (the live path is `testnet/node-data`), binaries `kovanica-node`
   (the running ones are `kvnc-node`), and validator keys `val{1,2,3}.pem` (seed2 actually reads
   `testnet/validators/v2.dat`). Treat it as superseded.

### 9.5 Revised remaining work (today's state; supersedes §5's ordering)

- **Step 0 — name the objective.** The seeds' original job was to let new nodes bootstrap. RPC clients
  are already served by seed1's Docker cluster, but **no reachable plaintext-TCP bootnode exists** for
  a third-party node that wants to join. That gap — not the rename — is the thing worth closing.
- **Step 1 — commit the units + env templates** (§9.4 defect 3). Nothing downstream is reviewable until
  a source of truth exists in git.
- **Step 2 — fix the template port** to a `%i`-derived value (§9.4 defect 4).
- **Step 3 — decide what the seeds are for.** Note that Option A (rename-only, already decided) is about
  *drift hygiene*: seed2's old node runs a different **transport** (libp2p on 8000) from the renamed
  node (plaintext TCP on 8000), and does not share the renamed node's `/etc/kovanica` authority set, so
  a pure rename will **not** connect the seeds to the live Docker chain. Only Option B does that, and it
  needs D2 + D3 resolved per seed.
- **Step 4 — seed3 first** (already down → zero incremental risk): apply the 5 G ceiling, wipe
  `/var/lib/kovanica-testnet-seed3` if stale, start, then verify against the live chain. **D3 is still
  unresolved and blocking**: `/etc/kovanica/testnet-authorities.conf` declares
  `d1a14d2c… / a5e261ae… / a1affed9…` with `KOVANICA_AUTHORITY_THRESHOLD=2`, while the running chain's
  genesis set is `aa7ec9c9… / 7173710d… / 85043fd3…`. The two sets do not overlap.
- **Step 5 — seed2**: 5 G ceiling + wipe the stale `/var/lib/kovanica-testnet-seed2` (3.2 MB, Oct 6) +
  start the new unit, leaving `kvnc-testnet-seed2` running for rollback. Remember the bootnode it is
  configued with (`seed.kovanica.online:8000`, `seed3.kovanica.online:8000`) is not a listener
  (§9.4 defects 1–2).
- **Step 6 — seed1**: publish a TCP P2P port from the seed container if seed1 is to be a real bootnode;
  today it advertises nothing. Fix the compose's `KOVANICA_BOOTNODES` in the same change. Defer while
  the concurrent session holds the shared cwd.
- **Step 7 — rewrite `docs/TESTNET-SEED-NODES.md`** (§9.4 defect 9).

### 9.6 Acceptance criteria (revised)

- `rpc-testnet.kovanica.online` `POST /rpc` → `kovanica_blockNumber` monotonically advancing
  (**already true**);
- at least one of seed1/2/3 accepts an **inbound plaintext-TCP P2P connection** from a node that is not
  on the same Docker network, and its genesis/authority set matches the live chain;
- the units + env templates exist in git, and `systemd-analyze verify` passes on each (`§4.4`);
- `grep -rn 'KVNC_' /etc/systemd/system /root/Projects/kvnc/testnet` → empty (extended beyond §6's
  unit-only check);
- no unit or launcher script depends on `/root/Projects/kvnc` for its **code** path, or that dependency
  is documented and versioned.

