# Testnet rename cutover + seed recovery — rollout plan

Status: **PLAN ONLY. Nothing has been executed.** No host has been written to, restarted, or
reconfigured. Every statement below is from a read-only audit performed 2026-10-15.

Owner sign-off gates are marked **[OWNER]**; each must be answered before its phase runs.

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
> Option B stays as a separate scheduled migration with its own dry run. D2–D5 below still need answers
> before Phase 2, and D2 is partly an Option-B-only concern (the rename-only path does not apply
> `MemoryMax=3G`), though seed3 must still be restarted without an OOM.

**[OWNER] D2 — Memory ceiling.** The new units cannot start at `MemoryMax=3G` (projected peak
3 368 MiB and growing; pruning disabled per RFC-009). Options:
  - raise the ceiling (seed2 has 6 168 MB free and its mainnet instance is unused → 5G is feasible
    today) — but this recurs as the log grows;
  - wipe + resync (`mv alpha.log alpha.log.broken`, then resync from a peer on :8000) — the binary's
    own suggested recovery; also temporary;
  - snapshot-restore — **not yet verified to exist** for this network; needs confirmation;
  - revisit pruning / RFC-009 — a design change, out of scope for a rollout.
  Recommendation: unblock with a raised ceiling **and** open a separate workstream for the durable fix.

**[OWNER] D3 — Authority set / producer set.** Confirm that `/etc/kovanica/testnet-authorities.conf`
and the `PRODUCE=1` flags describe the intended validator set. If the new authority set differs from
the one on the running chain, the cutover forks or the new nodes reject the chain.

**[OWNER] D4 — Window and blast radius.** The three seeds are the testnet's bootstrap set. Recommend
seed3 first (already down → zero incremental risk), then seed2, then seed1 (dev box, deferred).

**[OWNER] D5 — What to do with the seed1 unmanaged processes and the pre-rename Docker cluster**
(kill/recreate under the renamed compose, or leave alone).

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
   **pinned, pushed** SHA (currently `main` = `0d1b114c`) and record its `sha256sum`.
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
curl -s http://127.0.0.1:3001/api/head | jq '.height, .chain_id'
ss -ltnp | grep -E ':8000|:3001'                         # expect the P2P + loopback RPC binds
```

Then cross-check the public surface (`https://explorer.kovanica.online/api/head` and
`/api/bootstrap`) to confirm the network is still advancing and the advertised peers/port are sane.
**Port note:** this network runs P2P on **8000** (not 9000). Keep the live port from `/api/bootstrap`
as the source of truth before hard-coding it anywhere.

## 6. Phase 3 — post-cutover verification

- all three seeds `active`, peer counts ≥ 2, tip advancing, `/api/head` height monotonic;
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

---

### Related, already-fixed in this session
- Bootnode typo `seed3.kovanica.online` → `seed3.kovanica.online` in `run-seed1.sh`, `mainnet/run-seed1.sh`,
  `mainnet/run-seed2.sh` (commit `e316d70a`).
- Android `libkovanica_light_ffi.so` rebuilt; stale `libkvnc_light_ffi.so` removed.
