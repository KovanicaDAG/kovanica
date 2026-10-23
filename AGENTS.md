# kovanica – OpenCode Agent Instructions

> **Project identity (NE MJEŠAJ S DRUGIM PROJEKTIMA)**

**kovanica** is a clean, from-scratch Rust Layer-1 blockchain.

**Naming (zaključano):** chain = **Kovanica**, native coin ticker = **KUNA** (9 decimals),
codebase/binary/env prefiks = **kovanica** (`kovanica-*` crate-ovi, `kovanica-node`, `KOVANICA_*`). Adresa je
`kvnc` + 57 base32 znakova (bech32 abeceda) + `dag` = 64 znaka (PRO-21: 32 B ključ ‖ prvih 29 bitova blake3(ključ)); legacy `kvnc<hex>dag` i hex samo na chainu 4. Nikad ne koristi `KVNC` kao naziv valute.

| Layer              | Choice                                      |
|--------------------|---------------------------------------------|
| Consensus          | Mysticeti-style uncertified DAG (wave = 3)  |
| Execution          | Wasmi (deterministic WASM)                  |
| Account model      | Account-based + staked validators           |
| Validators         | 15–21 active, min stake 50 000 KUNA         |
| Resource target    | Comfortable on 2–4 GB RAM VPS               |
| Native token       | KUNA (9 decimals)                           |

## HARD RULES (NEKRŠIVA)

1. **NE** uvozi, kopiraj, referenciraj ili miješaj bilo kakav kod/dizajn iz `kovanica-protocol`, starijih Kovanica repozitorija ili bilo kojeg drugog lanca.
2. Ovo je **nezavisan** codebase. Sve odluke moraju biti konzistentne s ovim dokumentom i postojećim crate-ovima.
3. Preferiraj **male, fokusirane, auditabilne** promjene. Jedan PR = jedna jasna namjera.
4. Ne diraj tokenomics parametre bez eksplicitnog odobrenja.
5. Ciljaj uvijek na 2–4 GB RAM footprint.

## Tokenomics (zaključano)

> Revidirano 2026-10-13: s₀=2,5 KUNA, era=8 100 000, budget=81M, +1M genesis validatorska alokacija, round 4 s (leader 12 s). Sve potvrđeno od strane vlasnika projekta.

| Parameter                  | Value                                          |
|---------------------------|------------------------------------------------|
| Total supply              | **90 200 000 KUNA**                            |
| Decimals                  | 9                                              |
| Founder premine           | 200 000 KUNA                                   |
| Treasury                  | 8 000 000 KUNA (1 M/year × 8 years, linear)    |
| Genesis validator alloc.  | 1 000 000 KUNA (ravnomjerno na genesis validatore, **5–20 validators**) |
| Mining subsidy budget     | **81 000 000 KUNA** (= 90.2M − 0.2M − 8M − 1M) |
| Initial block reward      | **2,5 KUNA** (paid to committed leader author) |
| Subsidy era length        | **8 100 000** committed leader blocks          |
| Decay                     | **× 3/4** each era                             |
| Round / leader interval   | round 4 s → committed leader svakih 12 s       |
| Epoch (validator rotacija)| 7 200 leader blokova (≈1 dan), neovisno o eri   |
| Active validators         | 15–21                                          |
| Min validator stake       | 50 000 KUNA                                    |

Reward se mint-a **samo** na committed leader (CommittedSubDag → StakingState::on_leader_committed). Genesis validatorima se alocira 1M KUNA (ravnomjerni split, dust deterministički), od čega svaki stake-a min 50 000 KUNA.

## Workspace structure (poštuj postojeće)

```
crates/
├── kovanica-types
├── kovanica-crypto
├── kovanica-storage
├── kovanica-dag
├── kovanica-consensus
├── kovanica-mempool
├── kovanica-network
├── kovanica-runtime          # Wasmi
├── kovanica-execution
├── kovanica-staking
├── kovanica-node
├── kovanica-cli
├── kovanica-rpc
├── kovanica-common
├── kovanica-htlc
├── kovanica-vault
├── kovanica-multisig
├── kovanica-token
└── kovanica-faucet
```

Packages (TypeScript):
- `packages/api-client`
- `packages/console`
- `packages/signer`
- `packages/web`

Contracts (već implementirani): HTLC, Vault, Multisig, Token + Host trait.

## Snapshot (2026-10-13, main @ 98d88cc)

- Workspace crates: 19
- Rust source: ~53k LOC across 113 `.rs` files
- TypeScript packages: 3 (`api-client`, `console`, `signer`) + `web`
- Default P2P listen: `0.0.0.0:8000`
- Default RPC bind: `127.0.0.1:8545`
- Default round duration: 4000 ms (=> 12 s per committed leader)
- Max peers: 50 (`config.example.toml`)
- Workspace tests: 503+ passing in the latest partial `cargo test --workspace` run (`sustained_liveness` still running before timeout)

## Coding standards

- Rust 2021 / 2024 edition po Cargo.toml
- Prefer `thiserror` + `anyhow` gdje ima smisla
- Determinističko ponašanje svugdje (Wasmi, consensus, rewards)
- Unit + integration testovi za svaku novu logiku
- Ne dodavaj nepotrebne dependencies
- Dokumentiraj javne API-je (rustdoc)
- Imena: snake_case, jasna, bez prefiksa "kovanica"

## Preferred next focus areas (prioritet)

A. Network & Node hardening (P2P, sync, snapshot/pruning, genesis)
B. Staking & Validator lifecycle (bond/unbond, active set, commission)
C. Execution & Contract UX (gas metering, ABI, deployment)
D. Observability (logging, metrics, indexer hooks)
E. Testnet readiness (multi-node, faucet, chaos)

## Kako raditi s Grok-om

- Grok = arhitekt + reviewer + planer
- OpenCode (ti) = implementator (file edit + shell + testovi)
- Kad dobiješ plan od Grok-a → izvrši ga u ovom sessionu
- Nakon implementacije vrati diff / test output za review

## Zabranjeno

- Miješanje s kovanica-protocol
- Promjena tokenomics bez odobrenja
- Veliki refaktori bez plana
- Dodavanje novih crate-ova bez opravdanja
- Non-deterministički kod u consensus/execution putu
