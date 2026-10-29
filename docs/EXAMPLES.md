# Kovanica Contract Examples

Worked end-to-end flows for the contract crates. All amounts are in
**atoms** (base units): `1 KUNA = 1_000_000_000 atoms` (9 decimals). Addresses use the canonical PRO-21 64-character base32 format (`kvnc...dag`).

> **CLI status.** `kovanica-cli` provides native contract subcommands (`htlc`,
> `vault`, `multisig`, `token`) implemented in `crates/kovanica-cli/src/contracts.rs`
> that map 1:1 onto the contract entry points in [`docs/CONTRACTS.md`](CONTRACTS.md)
> and JSON-RPC methods. Flows can be executed via the CLI or directly via JSON-RPC.

---

## HTLC atomic swap (KVP-104)

Alice locks KUNA for Bob; Bob claims with the preimage before expiry, or
Alice refunds after expiry.

```bash
# 1. Alice locks 100 KUNA for Bob.
#    hash-lock = BLAKE3-256(preimage) as hex; expiry = unix timestamp.
kovanica htlc create \
  --claimer <Bob-address> \
  --amount 100000000000 \
  --hash-lock <64-hex-chars> \
  --expiry <unix-seconds>

# → returns swap_id (32 bytes, hex)

# 2. Bob claims with the preimage before expiry.
kovanica htlc claim \
  --id <swap_id> \
  --preimage <hex>

# 3. (Alternative) If Bob never claims, Alice refunds after expiry.
kovanica htlc refund --id <swap_id>
```

Notes:

- `create` moves the amount into a per-record escrow address
  (`hash(b"htlc/escrow" || id)`); `claim` pays the designated claimer only.
- `hash-lock` is `kovanica_common::hash(preimage)` (BLAKE3-256).
- `refund` may be triggered by anyone once `expiry` has passed.

## Time-lock vault (KVP-105)

```bash
# Create: beneficiary can claim after an absolute unlock time.
kovanica vault create \
  --beneficiary <address> \
  --amount 50000000000 \
  --unlock-at <unix-seconds>

# Beneficiary claims the vested amount (repeat as vesting progresses).
kovanica vault claim --id <vault_id>

# Creator cancels and reclaims everything not yet claimed.
kovanica vault cancel --id <vault_id>
```

`--unlock-at` selects the `Absolute` schedule; a `--start/--end/--cliff`
form selects `Linear` vesting (see `VestingSchedule` in
[`docs/CONTRACTS.md`](CONTRACTS.md) §3).

## Multisig (KVP-101)

```bash
# Create a 2-of-3 wallet.
kovanica multisig create \
  --owners <addr1,addr2,addr3> \
  --threshold 2

# Any owner proposes (proposer auto-confirms) → returns tx_id.
kovanica multisig propose \
  --id <multisig_id> \
  --to <address> \
  --amount 1000000000

# Another owner confirms.
kovanica multisig confirm --id <multisig_id> --tx-id <tx_id>

# Anyone executes once the threshold is met; funds leave the multisig
# contract's own account.
kovanica multisig execute --id <multisig_id> --tx-id <tx_id>
```

## Token (KVP-102)

One contract instance is one token, so `token create` deploys the token contract
and then calls `token_create` — two signed transactions, both reported (see
[`docs/CONTRACTS.md`](CONTRACTS.md) §7a).

```bash
# Create a token; the keystore's address becomes owner + minter and receives the
# supply. `--icon` takes an https:// URL or a data:image/… URI (≤8 KiB).
kovanica token create --name "My Token" --symbol MTK --decimals 9 \
  --initial-supply 1000000000000000 --icon 'https://example.com/mtk.png'

# Standard ERC-20-style flows, all against the contract address create printed.
kovanica token transfer --contract <kvnc…> --to <address> --amount 5000000000
kovanica token approve  --contract <kvnc…> --spender <address> --amount 5000000000
kovanica token mint     --contract <kvnc…> --to <address> --amount 1000000000   # minter only
kovanica token burn     --contract <kvnc…> --amount 1000000000

# Read-only: decodes the contract's own state, no transaction.
kovanica token balance --contract <kvnc…> --address <address>
```

## KUNA ↔ token pool (P0a)

A token contract can host **one** constant-product pool against native KUNA.
Conversion is reserve-backed: no KUNA is minted or burned and the token supply
never changes (see [`docs/CONTRACTS.md`](CONTRACTS.md) §7b).

```bash
# 1. Seed the pool. `--kuna` is sent as the call's attached value; `--tokens`
#    moves out of the owner/minter's balance. Fee is in basis points (default 30).
kovanica token init-pool --contract <kvnc…> --kuna 1.5KUNA \
  --tokens 1000000 --fee-bps 30
# → prints the LP shares minted, floor(sqrt(kuna_in * tokens_in))

# 2. Buy tokens with KUNA. `--min-out` is the slippage floor in token base units.
kovanica token buy --contract <kvnc…> --kuna 0.5KUNA --min-out 300000

# 3. Sell tokens back for KUNA. `--min-out` is a KUNA amount here.
kovanica token sell --contract <kvnc…> --tokens 300000 --min-out 0.4KUNA

# 4. Read-only: reserves plus locally computed quotes for both directions.
kovanica token reserves --contract <kvnc…>
```

Notes:

- `--kuna` accepts the same units as every other amount (`1.5`, `1.5KUNA`,
  `5atoms`), and so does `sell --min-out`; `--tokens` and `buy --min-out` are
  token base units.
- Each mutation waits for its nonce, then re-reads the pool at
  `blake3("kvnc/v1/pool")` and the token state at `blake3("kvnc/v1/state")`.
  It **exits non-zero** unless the committed reserves and the caller's token
  balance satisfy the swap — exact deltas, `token_reserve <=
  balances[contract]`, unchanged `lp_total`. A ledger above the reserve is a
  token donation (warning, those tokens are trapped); a ledger below it is
  corruption and fails. Comparisons against the
  pre-submit snapshot (the local quote, the reserve movement this trade alone
  implies) are printed as `warning:` lines and never change the exit code: on a
  live pool another trade can land between the snapshot and the commit. Events
  are for indexers, not for this check. A rejected body still pays the fee, so
  a silent success would be a lie.
- `--min-out` defaults to `0`, i.e. no slippage protection; the swap still
  runs, but the CLI prints a warning that any pool movement between submit and
  commit is accepted.
- Pricing is `out = in_after * r_out / (r_in + in_after)` with
  `in_after = in * (10_000 - fee_bps) / 10_000`, all integer and floored; the
  fee stays in the reserve for LPs.
- Only one pool per token contract, and it cannot be re-seeded. Removing
  liquidity, token↔token pools and pause control are not implemented (PR-2/PR-3).

---

## Indexer hooks

Every flow above emits events with the canonical topics defined in
`kovanica_common::events` (see [`docs/CONTRACTS.md`](CONTRACTS.md) §9). Match on
the topic bytes and decode the data payload — e.g. `htlc_created` carries the
32-byte swap id, `multisig_proposed` carries the 8-byte tx id (little-endian),
and the token topics (`token_created`, `transfer`, `approval`, `mint`, `burn`,
`token_pool_initialized`, `token_swap_buy`, `token_swap_sell`) carry empty data.
