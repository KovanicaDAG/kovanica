# NOTED — Chain 4 launch snapshot (distilled)

Source: `docs/archive/sync/sync-2026-10-26-0550.md` (point-in-time). Topology and IPs: use `docs/TESTNET-SEED-NODES.md`, not this note.

## What was true at that sync

- Network: Kovanica Testnet v2, `chain_id = 4`, network tag `4-0eacd5699210b145` shared across nodes.
- Five validators on three seed hosts; faucet live on seed3 and dispensing test KUNA.
- Genesis tokenomics invariants: 90.2M cap, 9 decimals, 81M mining budget, 8M treasury, 1M genesis validators (5 × 200k with 100k staked + 100k liquid), 200k founder premine (`KAT_ZERO`), faucet funded account via `funded_accounts.toml`.

## Deploy fixes called out then

- Faucet Axum 0.7 `ConnectInfo` requires `into_make_service_with_connect_info`.
- Faucet RPC URL normalized to `/rpc`; balances/nonces use canonical address string (PRO-21).
- Loopback RPC auth disabled for local faucet/CLI tooling via systemd env on validator units.

## How to use this file

Keep lasting invariants here or in `STATE.md`. Refresh ops reality from seed docs and live checks. Do not paste credential status or public IP tables into SoT.
