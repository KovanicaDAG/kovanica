# kovanica – OpenCode Agent Instructions

> **Project identity (NE MJEŠAJ S DRUGIM PROJEKTIMA)**

**kovanica** is a clean, from-scratch Rust Layer-1 blockchain.

**Naming (zaključano):** chain = **Kovanica**, native coin ticker = **KVNC** (9 decimals; preimenovano natrag iz KUNA per Toni 2026-11-05),
codebase/binary/env prefiks = **kovanica** (`kovanica-*` crate-ovi, `kovanica-node`, `KOVANICA_*`). Adresa je
`kvnc` + 57 base32 znakova (bech32 abeceda, samo mala slova) + `dag` = 64 znaka (PRO-21: 32 B ključ ‖ prvih 29 bitova blake3(ključ)); `FromStr` je samo taj oblik. Legacy `kvnc<hex>dag` i hex (heks znamenke case-insensitive, affixi i `0x` mala slova) na chainovima 3 i 1337, odbijeni na chainu 1, 2 i svakom drugom id-u. Adresni prefiks `kvnc`...`dag` je usklađen s KVNC tickerom.

**Chain ID Registry (3 Canonical Chains):**
- `1` = **Mainnet** (`MAINNET`) — canonical PRO-21 base32 address, Hybrid PoW + GHOSTDAG (NO stake for consensus, KovanicaHash, target block 4 s, GHOSTDAG K=5). `funded_accounts.toml` strictly forbidden.
- `2` = **Testnet** (`TESTNET`) — official Hybrid PoW + GHOSTDAG testnet (KovanicaHash, target block 4 s, GHOSTDAG K=5, NO stake for consensus; canonical PRO-21 base32 addresses strictly enforced; `funded_accounts.toml` and faucet allowed; legacy alias HYBRID_TESTNET=2).
- `3` = **Devnet** (`DEVNET`) — devnet; legacy hex addresses allowed, dev parameter overrides.
- `1337` = **Local devnet** (`LOCAL`) — single-node / local cluster testing (legacy hex allowed).

*Params SoT:* `kovanica-params`. `funded_accounts.toml` allowed on every chain except mainnet (1).

| Layer              | Choice                                      |
|--------------------|---------------------------------------------|
| Consensus          | Hybrid PoW + GHOSTDAG (NO stake for consensus; target block 4 s, GHOSTDAG K=5; PoW hash: KovanicaHash) |
| Execution          | Wasmi (deterministic WASM)                  |
| Account model      | Account-based                               |
| Resource target    | Comfortable on 2–4 GB RAM VPS               |
| Native token       | KVNC (9 decimals)                           |

## HARD RULES (NEKRŠIVA)

1. **NE** uvozi, kopiraj, referenciraj ili miješaj bilo kakav kod/dizajn iz `kovanica-protocol`, starijih Kovanica repozitorija ili bilo kojeg drugog lanca.
2. Ovo je **nezavisan** codebase. Sve odluke moraju biti konzistentne s ovim dokumentom i postojećim crate-ovima.
3. Preferiraj **male, fokusirane, auditabilne** promjene. Jedan PR = jedna jasna namjera.
4. Ne diraj tokenomics parametre bez eksplicitnog odobrenja.
5. Ciljaj uvijek na 2–4 GB RAM footprint.

## Consensus & PoW Hash (zaključano Toni 2026-11-05)

- **Product-of-record consensus:** Hybrid PoW + GHOSTDAG; **NO stake for consensus** (odluka Toni 2026-11-05).
- **PoW Hash:** **KovanicaHash** (striktno NE KunaHash; planiran blake3 + 2 MiB memory mix; audit obavezan prije mainneta).
- **Consensus defaults:** `TARGET_BLOCK_MS = 4000` (4 s) i `GHOSTDAG_K = 5`.
- **Legacy Mysticeti constants (still in tree until hybrid flag):** `WAVE_LENGTH = 3`, `EPOCH_ROUNDS = 7200`, `LEADER_INTERVAL_SECS = 12`, `ROUND_SECS = 4`.
- **Legacy / non-consensus under hybrid:** `MIN_VALIDATOR_STAKE`, `MIN_DELEGATION`, `JAIL_ROUNDS` (raniji Mysticeti-C parametri više nisu u konsenzusnom putu pod hibridom).
- **Launch gating:** Javni launch je odgođen s 25.11.2026 do prolaska hibridnog testneta (milestone M8).
- **PRO-7 Status:** FROZEN za hibridni konsenzus (Mysticeti VRF leader put je napušten u korist PoW + GHOSTDAG).

## Tokenomics (zaključano)

> Revidirano 2026-10-13 / Ažurirano 2026-11-05 / no-stake 2026-11-06: ticker KVNC, s₀=2,5 KVNC, era=8 200 000 rewarded blokova, budget=82M, genesis validatorska alokacija 0 (1M vraćen u mining emission), target block 4 s (`TARGET_BLOCK_MS = 4000`), GHOSTDAG K=5. Sve potvrđeno od strane vlasnika projekta. Chain 1 (mainnet) i chain 2 (testnet) su oba hybrid (`NoStake82M`); legacy `Legacy81M` konstante ostaju u kodu samo za postojeće baze i dev chainove.

| Parameter                  | Value                                          |
|---------------------------|------------------------------------------------|
| Total supply              | **90 200 000 KVNC**                            |
| Decimals                  | 9                                              |
| Founder premine           | 200 000 KVNC                                   |
| Treasury                  | 8 000 000 KVNC (1 M/year × 8 years, linear)    |
| Genesis validator alloc.  | **0 KVNC** (ukinuto — nema stakea u konsenzusu; **genesis bootstrap seeds 1..=20**, bez stakea i bez alokacije; bivši 1M vraćen u mining emission) |
| Mining subsidy budget     | **82 000 000 KVNC** (= 90.2M − 0.2M − 8M) |
| Initial block reward      | **2,5 KVNC** (paid to committed block author)  |
| Subsidy era length        | **8 200 000** rewarded blocks (`SUBSIDY_ERA_BLOCKS_NO_STAKE_82M`, legacy 8.1M ostaje za `Legacy81M`) |
| Decay                     | **× 3/4** each era                             |
| Target block time         | 4 s (`TARGET_BLOCK_MS = 4000`)                 |
| GHOSTDAG parameter K      | 5 (`GHOSTDAG_K = 5`)                           |
| Epoch (legacy Mysticeti)  | epoch = 8 h = 7 200 DAG rundi = 2 400 leader visina (legacy Mysticeti constant) |
| Min validator stake       | 50 000 KVNC (`MIN_VALIDATOR_STAKE`; legacy / non-consensus) |
| Min delegation            | 50 000 KVNC (`MIN_DELEGATION`; legacy / non-consensus)      |
| Jail duration             | 7 200 heights (`JAIL_ROUNDS`; legacy / non-consensus)       |

Reward se mint-a na committed blokove, autoru bloka. Genesis bootstrap seeds (1..=20) ne drže stake i ne dobivaju alokaciju; `[[validator]]` u genesis datoteci je samo zapis bootstrap identiteta (zaostali naziv tablice).

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

## Snapshot (2026-10-24, main @ dd3aff9)

- Workspace crates: 27
- Rust source: ~90.5k LOC (crates) / ~95.5k LOC (repo) across 163 `.rs` files
- TypeScript packages: 5 (`api-client`, `comms`, `console`, `signer`, `web`)
- Default P2P listen: `0.0.0.0:8000`
- Default RPC bind: `127.0.0.1:8545`
- Target block time: 4000 ms (`TARGET_BLOCK_MS`); legacy 12 s leader cadence is retired
- Max peers: 50 (`config.example.toml`)
- Workspace tests: 1,101+ tests passing (100% pass in verified crate suites)

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
