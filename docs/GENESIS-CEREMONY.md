# Genesis Ceremony Protocol

Status: Active procedure for multi-party genesis generation and bootstrap coordination.
Consensus Impact: Ledger initialization; executed once before chain start.
Architecture: **Hybrid PoW + GHOSTDAG** (NO stake for consensus; target block 4 s, GHOSTDAG $K=5$; PoW Hash: KovanicaHash).

Purpose: Coordinate the genesis initialization so that all bootstrap seed nodes (recommended: 3–5 seeds, bounded 1..=20) agree on initial state, staking allocations, treasury address, founder premine, and chain ID.

---

## 1. Inputs

- **`validators.json`** — JSON array of initial bootstrap seed nodes:
  `{ "address": "<canonical PRO-21 address>", "public_key": "<32-byte hex>", "stake": <atoms> }`
- **Treasury Address** and **Founder Address** — Canonical PRO-21 addresses (`kvnc...dag`, 64 characters).
  - Treasury: 8,000,000 KVNC (3-of-5 multisig custody; 8-year linear vesting).
  - Founder: 200,000 KVNC premine.
- **Chain ID** — `1` Mainnet, `2` Testnet (legacy), `3` Devnet, `4` Testnet v2, `5` Hybrid Testnet, `1337` Local.

---

## 2. Procedure (`ops/genesis/ceremony.sh`)

Usage:
```bash
./ops/genesis/ceremony.sh --network mainnet --validators validators.json \
    --treasury-address <treasury-address> \
    --founder-address <founder-address> \
    --output ./genesis [--force]
```

Steps executed:
1. **Generate Genesis:** Calls `kovanica-node genesis` to generate deterministic `genesis.json`.
2. **Generate Node Keys:** Generates bootstrap seed node credentials in `--validator-keys-out <output>/keys`.
3. **Emit `genesis_validators.toml`:** Populates bootstrap seed node addresses and allocations.
4. **Emit `bootnodes.toml`:** Configures initial DNS seed addresses (`testnet.kovanica.online:8000` or `mainnet.kovanica.online:8000`).

Outputs placed in `--output`: `genesis.json`, `genesis_validators.toml`, `bootnodes.toml`, and `keys/`.

---

## 3. Configuration Formats

### 3.1 `genesis_validators.toml`

```toml
chain_id = 1

[[validator]]
address = "kvnc1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdag"
stake = 50000000000000
public_key = "4a7b..."
```

### 3.2 Mainnet Bootstrap Seed Node Bounds (1..=20)

Under **Hybrid PoW + GHOSTDAG**, consensus requires **NO stake in the consensus path**: blocks are mined via Proof-of-Work (KovanicaHash) and ordered via GHOSTDAG ($K=5$). 

- The legacy 15-validator BFT committee quorum floor is retired.
- Mainnet genesis requires at least **1** bootstrap seed node and at most **20** (the ceiling of the 1,000,000 KVNC allocation split across the 50,000 KVNC minimum stake).
- A lean deployment of **3–5 bootstrap seed nodes** (`seed1`, `seed2`, `seed3.kovanica.online`) is recommended for public discovery.
- `kovanica-node --check-config` enforces this bound on Mainnet when ceremony files are present.

### 3.3 Prohibition of `funded_accounts.toml` on Mainnet

```toml
[[account]]
address = "kvnc1..."
balance = 1000000000000
```

Pre-funding accounts from `funded_accounts.toml` is permitted only on test/dev networks (chains 2, 3, 4, 5, 1337). On **Chain 1 (Mainnet)**, this file is **strictly prohibited**; every KVNC must originate from the published emission allocations (founder premine, treasury vesting, genesis validator allocation, mining subsidy). The node aborts on startup if `funded_accounts.toml` is detected on Mainnet.

---

## 4. Genesis Digest Guard (Startup Behaviour)

On initial startup, the node computes and persists a cryptographic digest of the genesis state:
- **Match:** Startup proceeds immediately.
- **Mismatch:** Startup aborts with an explicit configuration error to prevent accidental chain forks.
- **Chain State without Digest:** Refuses startup (requires wiping legacy state).

---

## 5. Network Safety Rules

- Keep seed DNS records strictly **grey-cloud** (no Cloudflare proxying or CDN).
- P2P transport operates on port **8000** (TCP + Noise + Yamux).
- Faucets are disabled on seed nodes (`FAUCET=0`).
- Mining nodes operate permissionlessly with standard PRO-21 addresses via `KOVANICA_MINING_AUTHOR`.
