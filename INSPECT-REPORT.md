# Codebase Inspect Report — 2026-09-29

**Host:** seed1 / vps1 (145.223.116.178) · **Mode:** read-only · **Tree:** `main` @ `77bd1e7`, clean
**Gates:** `fmt` 0 · `clippy -D warnings` 0 · `test --workspace` 0 (68 suites, 874 tests, 0 failed)

No file was modified. No secrets appear in this report.

---

## Summary

The consensus and ledger core is **in better shape than the catalog assumed**. Of 23 catalog IDs,
**zero P0 defects were found in consensus-critical code.** Every hard RFC-006 invariant is
implemented and actively tested: the 90.2M cap, 100-block maturity, 75% fee burn, BLAKE3 sighash
with correct witness-free/full split, and the address version range.

The catalog's four headline P0/P1 leads — **C4** (missing Prometheus supply gauges), **G1**
(prepare/sign/submit drift), **A1** (`kov1` vs `kvnc` placeholder), **B1** (SHA256 vs BLAKE3) —
are all **false positives**. I verified each against the code and explain why below. Two of my own
initial readings were also wrong and I corrected them in-flight; those corrections are recorded
because the false-positive pattern is itself the useful finding.

Real work is concentrated in **H1 (docs topology drift)** and **I1 (dead code)** — both low-risk,
client/docs-only. **There is no consensus P0 to fix**, which means the package's exit criterion
"P0 list ≤ reasonable" resolves to a short list, and the honest answer is that the P0 backlog the
catalog assumed does not exist in this tree.

---

## Repo map

Workspace `protocol/` — 7 crates, ~44k LOC src + ~20k LOC tests.

| Crate | src LOC | test LOC | Role |
|---|---|---|---|
| `kovanica-node` | 18,476 | 7,458 | node, explorer HTTP, P2P, metrics, PoA |
| `kovanica-state` | 10,431 | 9,623 | **ledger, UTXO, tokenomics, sighash** |
| `kovanica-dag` | 4,193 | 2,038 | **GHOSTDAG, authority set** |
| `kovanica-cli` | 2,996 | 0 | CLI / client |
| `kovanica-chat` | 2,083 | 0 | KVP-104 messaging (leaf) |
| `kovanica-ffi` | 1,767 | 905 | UniFFI → Kotlin/Swift |
| `kovanica-wallet` | 732 | 268 | key/slip10 primitives |

`node/` is a thin bin wrapper. `sdk/` is a **separate 6-crate workspace** (`kovanica-rpc`,
`kovanica-tx`, `kovanica-keys`, `kovanica-fee`, `kovanica-types`, `kovanica-sdk`).

---

## Findings

| ID | Area | Path | Layer | Priority | Note |
|---|---|---|---|---|---|
| H1-a | Docs | `vault/99-Unfiled/*`, `vault/30-Operations/*` (12 files) | docs | **P1** | seed3 described inconsistently across vault mirrors; canonical `TESTNET.md` / `TESTNET-RFC006.md` are **correct**, the unfiled mirrors are not |
| ~~I1-a~~ | Dead code | 4× `#[allow(dead_code)]` | mixed | **WITHDRAWN** | **Not a defect.** All 4 suppressions are correct and load-bearing. See correction below. Do not "fix". |
| I1-b | Repo | 5 untracked top-level artifacts | ops | **P2** | `dist/`, `node.log`, `PRE-MIGRATION-LS.txt`, 2 setup `.sh` — untracked, not in git; tidy or ignore |
| D3 | Env | `/opt/kovanica/testnet/config/network.env` | ops | **P2** | 14 env vars the binary never reads (from `NETWORK-LAUNCH-PLAN.md`); misleading, inert |
| — | — | — | — | — | **No consensus/ledger P0 found** |

### Catalog IDs verified clean (not defects)

| ID | Claim | Verdict |
|---|---|---|
| A1 | `kov1` vs `kvnc` | **Clean.** 0 occurrences of `kov1` in any crate. Canonical `kvnc…dag` in `keys.rs:299,504`; no bech32 dep anywhere. |
| A2 | version bytes | **Clean.** `VERSION_P2PK=0x00` … `VERSION_VAULT=0x05`, `VERSION_MAX=0x05`, with `unsupported_version_rejected` (0x06) test. |
| B1 | SHA256 vs BLAKE3 | **Clean in consensus.** `sha2` appears only in RWA asset-id derivation (`tx.rs:308`), SLIP-10 (`slip10.rs:33`, SHA-512), and an explorer test helper. `sighash()` and `id()` are BLAKE3. |
| B2 | TxId parity | **Clean.** `id()` = BLAKE3 over full encode; `sighash()` = BLAKE3 over `encode_into(_, false)`. Tests `txid_differs_with_stealth_output`, `sighash_ignores_multisig_witness` cover the split. |
| C1 | max supply | **Clean.** `MAX_SUPPLY = 90_200_000 * ATOM` (`ledger.rs:170`), enforced in `apply_block`. |
| C2 | maturity | **Clean.** `COINBASE_MATURITY = 100` (`ledger.rs:182`); boundary-tested at height 99/100. |
| C3 | `/api/head` | **Clean.** Serves `native_minted`, `total`, `circulating`, `burned`, `max_supply`, `subsidy` (`explorer.rs:1822`). |
| **C4** | **missing supply gauges** | **FALSE POSITIVE.** All 5 gauges exist (`metrics.rs:164-168`, set at 341-345). `SUPPLY_MINTED` is fed from `total`, which *is* `native_minted` (`ledger.rs:2342`) — the struct has no separate field, and `metrics.rs:337-339` documents this deliberately. |
| D1 | seed IPs in code | **Clean.** Only DNS names, no hardcoded IPs. `DEFAULT_PEERS` = seed + seed2 (`explorer.rs:1548`). |
| D2 | orange-cloud peer | **Clean.** The only `explorer.kovanica.online` mention is a doc comment *warning against* it (`p2p.rs:14`). |
| D4 | faucet / reset | **Clean.** Faucet gated to testnet profile at runtime (`explorer.rs:3480`); `allow_reset` defaults `false` (842); only devnet sets `true` (550). |
| E1 | native null | **Clean.** Native = `None` in Rust, all-zeros `AssetId::native()`, encoded as 1-byte flag + optional 32 bytes (`tx.rs:655`). |
| **G1** | **prepare/submit drift** | **FALSE POSITIVE.** Both routes exist. They're dispatched by a `match` on the path *segment* (`explorer.rs:3290 "prepare"`, `3315 "submit"`), so a literal `"/api/…"` grep misses them. The CLI calls both; docs are correct. |
| J1 | ignored RFC-006 tests | **Clean.** The 2 supply-relevant ignores (`treasury.rs:224`, `live_sync_spike.rs`) are blocked on a *genesis-hash fixture* pending the PoA testnet reset — not missing coverage. Maturity/halving/cap tests are active: `emission_decays_geometrically_by_three_quarters_per_era`, `supply_cap_rejects_coinbase_past_max_supply`, `coinbase_output_is_immature_until_100_blocks_old`, `exact_cap_block_accepted_over_cap_rejected`. |

### Two of my own false leads (recorded for honesty)

1. **I initially reported C4 as a bug** — "`SUPPLY_MINTED` set from `total`, two gauges will
   report identical values." Wrong: `SupplyMetrics` has no separate `minted` field by design
   (`ledger.rs:196-201`), and the doc comment says so explicitly. I checked the struct before
   concluding.
2. **I initially reported G1 as a real drift** — "`/api/prepare` and `/api/submit` don't exist."
   Wrong: my route grep was too narrow. Found by cross-checking `kovanica-cli`, which *does* call
   both.

**Pattern worth carrying forward:** three of my four "findings" that looked like defects at first
glance were false, and both real corrections came from *checking the consuming side* (the struct
definition; the CLI's actual calls) rather than the producing side (the call site). For a
consensus-adjacent codebase, a grep-level hit is a hypothesis, not a finding.

---

## Relevant vs legacy

**Relevant:** `protocol/crates/*` (all 7), `sdk/crates/*` (6), `web/site`, `node/` (bin wrapper),
`wallet/`, `config/devnet`.

**Legacy / not-yet-wired:** `kovanica-chat` (0 in-workspace dependents — a lib with no shipped
consumer yet; its API is exercised by its own 28 test call sites). PoW surface
(`set_proof_of_work`, `difficulty`, `KOVANICA_POW`) is `[TARGET]`-removal per RFC-POA-Migration §0
but still compiled and reachable — known, tracked, not a finding.

### ⚠️ CORRECTION — I1-a was wrong; do not "fix" the `#[allow(dead_code)]`s

I originally wrote that 2 of the 4 suppressions masked genuinely-dead code and recommended
deleting them. **That recommendation was wrong, and acting on it would have broken the crate.**

`ed25519_seed_to_x25519` (`chat/crypto.rs:124`) looked dead from a definition-site grep, but it
has **28 call sites** — all of them inside `#[cfg(test)]` modules (`message.rs:181`,
`voice.rs:155`, `tipped.rs:185`, `file.rs:279`, and their test siblings). In a normal build the
function *is* unreachable, so `#[allow(dead_code)]` is exactly what keeps `clippy -D warnings`
green. Deleting it would break 28 test call sites.

All 4 suppressions check out:

| Site | Why it is correct |
|---|---|
| `chat/crypto.rs:123` | reached only from `#[cfg(test)]` (28 sites) |
| `chat/message.rs:12` | `MAX_PAYLOAD_LEN` is a documented public bound; only self-referenced today |
| `chat/tipped.rs:90` | forward declaration, explicitly marked "Used by node integration (Milestone 5)" |
| `state/spv.rs:429` | `checkpoint` field retained **for introspection**; verification uses `headers` — deliberately kept, and the comment says so |

**Root cause of my error:** I grepped for call sites and treated *test-only* reachability as
*zero* reachability. Same class of mistake as C4 and G1 — checking the wrong side of the
boundary. `cargo test` builds a different binary than `cargo clippy --workspace`; the lint
suppression and the test suite are answering different questions.

---

## Recommended fix order

| # | Item | Layer | Risk |
|---|---|---|---|
| 1 | H1-a — reconcile seed3 wording in the 12 vault mirrors | docs | none |
| 2 | D3 — strip the 11 unread vars from `config/mainnet/network.env` (tracked!) or comment them | ops/config | none (inert, but tracked — operators read it) |
| 3 | ~~I1-a~~ — **WITHDRAWN, do not "fix"** (see correction above) | — | **would break 28 tests** |
| 4 | I1-b — DONE: 5 top-level artifacts now `.gitignore`d (`.gitignore` modified on `docs/inspect-fixes`; nothing deleted) | ops | none |

All four are **client-only / docs-only**. None touch consensus or the ledger.

---

### ⚠️ CORRECTION — H1-a was overstated; the real residual is one orphaned generated mirror

H1-a as first written claimed "seed3 described inconsistently across 12 vault mirrors." I checked
each claim this pass. **The mirrors are already correct** — `vault/30-Operations/protocol--TESTNET.md:60`
("formerly also `seed3` — retired 2026-09-17"), `TESTNET-RESET-CREDENTIALS.md:17/102/136`, and
`TESTNET-RESET-PROCEDURE.md:37/195` all say retired/decommissioned correctly.

Two things I first read as drift are not:

- `protocol/OPERATIONS.md:191,234` list `seed3.kovanica.online:9000` as an advertised peer, but both
  blocks are explicitly marked `[HISTORICAL — PoW era]` period snapshots. Accurate for their date.
- `vault/docs/_planning/.../GENESIS_PLAYBOOK.md:115` is a **mainnet template** with `seed3...`
  elided — a future-provisioning placeholder, not a live testnet peer.

The canonical source `protocol/NETWORK.md` is **correct**: lines 23-24 list only `seed` and `seed2`
as seeds, and there are **zero** seed3 mentions in the file.

The one genuine residual: `vault/docs/_root/NETWORK.md:53` carried the same false
"**DNS deleted 2026-09-21** … **NXDOMAIN**" claim I already corrected in `protocol/OPERATIONS.md`.
Corrected in place. But note *why* it drifted: `scripts/build-vault.py` lists `"NETWORK.md"` (root)
in its `20-Network` source group, and **no root `NETWORK.md` exists** — the file is at
`protocol/NETWORK.md`. So this mirror is generated from a source path that is dead, and the
generator's own comment (line 129: "Canonical network truth is the root NETWORK.md") is stale.

`/vault/` is both **tracked** (82 files in the index) and **listed in `.gitignore:34`** as a
generated projection. Tracked-but-ignored: already-committed files stay tracked, but a regenerated
tree would not be re-added. Editing that mirror by hand is therefore a stopgap.

**Recommendation:** the durable fix is to point `build-vault.py`'s `20-Network` group at
`protocol/NETWORK.md` and regenerate, not to keep hand-patching mirrors. Flagged, not done — it
touches the vault sync tooling and is the operator's call.

The first draft of D3 said the dead env vars live in `/opt/kovanica/testnet/config/network.env`.
**That file does not exist on this host** — `/opt/kovanica` does not exist at all, and no live
systemd unit references that path. The finding was stale.

The real file is **`config/mainnet/network.env`**, which is **tracked in git** (`git ls-files`
confirms). That makes it more serious than first written, not less: it is the config an operator
reads when standing up mainnet, not a stray local file.

Verified by whole-word env read across `crates/*/src`: **0 exact reads** for all of
`KOVANICA_K`, `KOVANICA_MAX_SUPPLY`, `KOVANICA_COINBASE_MATURITY`, `KOVANICA_SUBSIDY`,
`KOVANICA_MIN_FEE`, `KOVANICA_ATOM`, `KOVANICA_FINALITY_DEPTH`, `KOVANICA_PRUNING_DEPTH`,
`KOVANICA_NETWORK_ID`, `KOVANICA_EXPLORER_PORT`, `KOVANICA_POA_NOMINAL_WORK`. 11 of these 12 are
defined in `config/mainnet/network.env`; `config/devnet/network.env` defines none of them (it is
clean). The values are inert — the binary ignores them — which is exactly why they are dangerous:
an operator can set `KOVANICA_MAX_SUPPLY` there and it will have **no effect**, silently.

**Root cause of my error:** a substring grep reported 11 "reads" of `KOVANICA_K`, which contradicted
my own finding. On inspection all 11 were `KOVANICA_KEY` (the CLI keyfile path in
`kovanica-cli/src/main.rs`) — `KOVANICA_K` matched as a prefix. Same class of error as C4/G1/I1-a:
a loose match instead of an exact one. I re-ran with quoted whole-word patterns to get the real
answer.

---

## Explicit non-goals this pass

- No edits of any kind (read-only per rule 1).
- No chain reset, no `MAX_SUPPLY` change (rule 3).
- No private keys, seeds, or password material in this report (rule 4).
- No consensus fix proposed, because none is needed (rule 5).
- Did **not** test any credential against seed2/seed3, and did **not** change SSH/auth config.

---

## Open questions

1. **Is `kovanica-chat` shipping?** 0 in-workspace dependents. If it's Milestone-5 staged, it
   should say so in its `Cargo.toml`; if it's live, something should depend on it.
2. **The `#[allow(dead_code)]` at `state/spv.rs:429`** — inside SPV, so closer to consensus than the
   others. Worth a deliberate keep-or-delete decision rather than leaving suppressed. (Kept today;
   correct, not urgent.)
3. **Vault mirror drift is systemic.** `99-Unfiled/` and `30-Operations/` both hold copies of the
   same operational docs with divergent content. Worth deciding whether mirrors are generated or
   hand-maintained — that determines whether H1 recurs after every fix.
4. **Did I1-b's `.gitignore` addition conflict with the operator's intent for the 3 setup scripts?**
   `kovanica-clean-setup.sh` deletes `/root/.cargo` and `/root/.rustup` — I only ignored it, I did
   not run it. If these are meant to be shared tooling, they belong in `scripts/` on their own
   branch, not ignored in the root.
