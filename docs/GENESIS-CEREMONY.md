# Genesis Ceremony (draft — [DRAFT - needs human review])

Status: Phase 14 / Phase 24 — NOT DONE. Created 2026-10-14.
Consensus impact: ledger-safe / client-only (genesis file is initial state; changes only at genesis, before chain start).

Purpose: coordinate multi-validator genesis for `kovanica-node genesis` so all 5–20 validators agree on the initial `genesis_validators.toml`, staking allocations, treasury address, and chain-id.

Procedure:

1. **Validator selection** (manual, before ceremony): agree on 5–20 active validators; collect `public_key` + optional `vrf_public_key` + `stake` (min 50_000 KUNA per `MIN_VALIDATOR_STAKE`) from each validator operator.
2. **Build `genesis_validators.toml`**: array-of-tables `[[validator]]` with `address`, `stake`, `public_key` (per parser `main.rs:2043-2050`). Use `docs/SEED-DNS-AUDIT.md` format only for reference — genesis file is separate.
3. **Hash comparison**: each validator runs `sha256sum genesis_validators.toml`; compare digests across all parties (off-chain, out-of-band). Mismatch → rebuild, do NOT proceed.
4. **Single-node genesis (current tool)**: `kovanica-node genesis --validators genesis_validators.toml --treasury-address <address> --founder-address <address> --validator-keys-out ./keys --force`. Writes `genesis.json` + staking state + `data_dir` DB.
5. **Multi-node verification** (post-genesis): each validator starts `kovanica-node` with same `genesis.json`; confirm `/api/head` identical across all nodes before opening to peers.
6. **Data preservation**: after first genesis write, set `KOVANICA_DATA` and never delete `data_dir` / `consensus.redb` / `state.redb`; only rebuild together (see `docs/TESTNET-RESET.md` line 5–6).

Safety rules (from `AGENTS.md`):
- No `KOVANICA_ALLOW_RESET=1` on public node.
- `FAUCET=0`; seed nodes must NOT expose faucet.
- Keep seed DNS grey-cloud (`seed.kovanica.online` removed already).
- `P2P` plaintext TCP 8000; use `KOVANICA_BOOTNODES` at deploy, never hard-code.

Gaps (verified by `nod-1`):
- No ceremony wrapper script (`ops/genesis/ceremony.sh` missing).
- No `docs/TESTNET-MAINNET.md` (6-line stub missing per TASKLIST #416).
- `docs/GENESIS-*.md` missing.
- No multi-party `validators.json` build helper.
- Single `run_genesis()` only — sufficient if ceremony is manual.

Next actions (not done):
- Create `ops/genesis/ceremony.sh` (calls `kovanica-node genesis --force` after hash-check pass).
- Register seeds (`seed1/2/3.kovanica.online`) and set `KOVANICA_BOOTNODES`.
- After genesis + freeze checklist (`docs/MAINNET-FREEZE-CHECKLIST.md`), only then tag `mainnet-freeze`.
