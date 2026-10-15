# Testnet rename cutover + seed recovery — rollout plan

Status: **PLAN ONLY. Nothing has been executed.** No host has been written to, restarted, or
reconfigured. Every statement below is from a read-only audit performed 2026-10-15.

Owner sign-off gates are marked **[OWNER]**; each must be answered before its phase runs.

---

## 1. What the audit actually found

The live fleet runs **two generations of deployment side by side**. The `kvnc-*` generation is
what carries the traffic; the `kovanica-*` generation is staged but not viable.

| | seed1 `srv1745734` (145.223.116.178) | seed2 `srv1991525` (76.13.250.65) | seed3 `srv2013143` (187.7.27.139) |
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
KVNC_BOOTNODES=145.223.116.178:8000,76.13.250.65:8000,187.7.27.139:8000
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
- Bootnode typo `178.7.27.139` → `187.7.27.139` in `run-seed1.sh`, `mainnet/run-seed1.sh`,
  `mainnet/run-seed2.sh` (commit `e316d70a`).
- Android `libkovanica_light_ffi.so` rebuilt; stale `libkvnc_light_ffi.so` removed.
