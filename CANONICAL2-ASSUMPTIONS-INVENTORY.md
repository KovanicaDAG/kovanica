# Canonical2 Proposal: Chain & Network Assumptions Inventory (`kovanica-web`)

This document inventories all chain, network, consensus, and numeric assumptions across `kovanica-web` adapters, server configuration, UI components, links, fixtures, and documentation to prepare for the separate `canonical2` proposal.

Per the acceptance constraints:
- **Identities vs. Arbitrary Numbers** are strictly distinguished below.
- **No blanket `5`-to-`2` replacement**, live-endpoint changes, genesis/storage edits, or deployment configuration changes have been made in this PR.

---

## 1. Protocol & Chain Identities (Candidate Targets for `canonical2`)

These values represent network names, chain IDs, domain tags, and consensus graph parameters that identify the chain or consensus topology.

| Category | Location | Symbol / Literal | Current Value | Role & `canonical2` Impact |
| :--- | :--- | :--- | :--- | :--- |
| **Chain ID Mapping** | `src/lib/networkConstants.ts:251` | Spec text chain ID table | `mainnet=1, testnet=2, devnet=3, testnet_v2=4, hybrid_testnet=5, local=1337` | Canonical numeric chain ID mapping. `hybrid_testnet=5` vs `testnet=2` must be reconciled by the `canonical2` spec. |
| **Chain ID Fallback Label** | `src/components/docs/docs-view.tsx:74,389` | `boot.network ?? "Chain 5"`, `"Chain 5 hybrid testnet verification"` | `"Chain 5"` | UI fallback/copy referring to Chain 5 (`hybrid_testnet=5`). Do not blanket-replace without `canonical2` chain ID confirmation. |
| **Network Identifier** | `src/lib/networkConstants.ts:8-9`, `src/lib/ledger/types.ts:78` | `NETWORK_ID`, `MAINNET_ID`, `NETWORK` | `"kovanica-testnet"`, `"kovanica-mainnet"` | Canonical network string identifiers returned by `/api/head`, `/api/bootstrap`, and `getNetworkId()`. |
| **Legacy Hex Chain Rule** | `src/lib/networkConstants.ts:352` | Spec text wallet address rule | `chains 3, 4, 1337` | Documents which numeric chain IDs permit legacy hex addresses versus strict PRO-21 `kvnc…dag`. |
| **GHOSTDAG Parameter `k`** | `src/lib/networkConstants.ts:28,265`, `src/components/network/network-view.tsx:307`, `src/components/docs/docs-view.tsx:145,260-265,284`, `src/components/explorer/explorer-view.tsx:86` | `K = 5`, `k = 5`, `k=5` | `5` | **Consensus Anticone Parameter (`GHOSTDAG K = 5`)**, NOT a chain ID! Must **never** be hit by a blanket `5`-to-`2` replacement unless consensus `k` itself changes. Note: local mock ledger (`src/lib/ledger/types.ts:79`) uses `K = 3` for lightweight browser simulation. |
| **Sighash Domain Tag** | `src/components/docs/docs-view.tsx:167,223`, `vendor/kovanica-sdk-wasm/` | `KUNA-TX-v1` | `"KUNA-TX-v1"` | Cryptographic BLAKE3 sighash domain separator (`kovanica-consensus` compatibility). Changing this breaks transaction signature verification. |
| **Address Format & HRP** | `src/lib/wallet/address.ts`, `src/components/docs/docs-view.tsx:186` | `kvnc` + 57 base32 + `dag` | 64 chars (PRO-21) | Canonical PRO-21 address encoding with 29-bit BLAKE3 checksum; network-agnostic across Kovanica chains. |
| **Key Derivation (ADR-0002)** | `src/lib/wallet/keys.ts:11-20`, `tests/fixtures/vectors.json` | 24-word BIP-39 entropy = 32-byte Ed25519 seed | Direct entropy seed | One mnemonic phrase maps deterministically to one Ed25519 public key and PRO-21 address across all networks. |

---

## 2. Server Configuration, Adapters & Surface Links

| Category | Location | Symbol / Literal | Current Value | Notes |
| :--- | :--- | :--- | :--- | :--- |
| **Source Resolution** | `src/lib/api/dispatch.server.ts:54-62`, `src/lib/api/client.ts:15-42` | `sourceOf(req)`, `readSource()` | `"testnet"` \| `"mainnet"` \| `"local"` | Resolves active source from `?source=`, `x-kovanica-source` header, `localStorage["kovanica.source"]`, or hostname (`detectNetwork()`). |
| **Mainnet Prelaunch Gate** | `src/lib/api/dispatch.server.ts:762-783` | `source === "mainnet"` | `503 "mainnet launching soon"` (reads), `403 "mainnet transactions disabled prior to 1.1.2027 launch"` (writes) | Strict server-side isolation preventing mainnet from falling back to testnet RPC or local mock state. |
| **Testnet RPC Adapter** | `src/lib/api/dispatch.server.ts:69-90` | `LOCAL_RPC` | `"http://127.0.0.1:8545/rpc"` | Local JSON-RPC 2.0 upstream used by the Nitro server for `source === "testnet"` (`kovanica_*` RPC methods). |
| **Testnet Faucet Upstream** | `src/lib/api/dispatch.server.ts:70` | `LIVE_FAUCET` | `"https://seed3.kovanica.online/faucet"` | Proxied by `POST /api/faucet` on `source === "testnet"`. |
| **Published Surface Map** | `src/lib/surfaces.ts:7-15`, `src/lib/network.ts:18-65`, `src/lib/networkConstants.ts:4-15` | `SURFACE`, `LIVE_EXPLORER`, `LIVE_WALLET`, `LIVE_MAP`, `LIVE_SITE`, `API_PUBLIC`, `DOCS_PUBLIC` | `kovanica.online`, `testnet.kovanica.online`, `mainnet.kovanica.online`, `explorer.kovanica.online`, `wallet.kovanica.online`, `api.kovanica.online`, `docs.kovanica.online` | Hostname routing and cross-surface links. |
| **P2P Bootstrap Seeds** | `src/components/network/network-view.tsx:192-204`, `src/components/docs/docs-view.tsx:315-319`, `src/lib/networkConstants.ts:380-385` | Seed hostnames & TCP port | `seed1.kovanica.online:8000`, `seed2.kovanica.online:8000`, `seed3.kovanica.online:8000` | Grey-cloud DNS TCP `:8000` P2P swarm bootstrap peers. |

---

## 3. Arbitrary Numbers, Economic Constants & Timeouts (Must NOT Be Treated as Chain IDs)

Many constants contain the digits `5` or `2` as economic quantities, gas/fee parameters, or UI polling intervals. **None of these are chain identifiers**, and none should be altered by a chain-ID migration:

| Location | Constant / Expression | Value | Meaning (Why It Is Not a Chain ID) |
| :--- | :--- | :--- | :--- |
| `src/lib/networkConstants.ts:19`, `src/lib/ledger/types.ts:77` | `SUBSIDY` | `2.5 * ATOM` (`2_500_000_000` atoms) | Initial block reward $s_0 = 2.5\text{ KVNC}$. Contains both `2` and `5`. |
| `src/lib/networkConstants.ts:27` | `MIN_FEE` | `Math.max(Math.floor(SUBSIDY / 500_000), 1)` (`5_000` atoms) | Fee-floor divider (`500_000`) yielding `5,000` atoms in spec calculation; node RPC/contract floor (`src/lib/api/contract.ts`) uses `10,000` atoms. |
| `src/lib/api/dispatch.server.ts:75` | `TOKEN_GAS_LIMIT` | `5_000_000` | Wasmi smart-contract fuel limit per token transaction (`5M` gas). |
| `src/lib/api/dispatch.server.ts:87` | RPC `AbortSignal.timeout(5000)` | `5000` ms | 5-second HTTP timeout for local JSON-RPC calls. |
| `src/components/network/network-view.tsx:92` | `setInterval(..., 15_000)` | `15_000` ms | 15-second UI auto-refresh interval on `/network`. |
| `src/lib/api/use-node.ts:5` | `MIN_POLL_MS` | `2000` ms | 2-second minimum polling interval for `/api/state`. |
| `src/components/docs/docs-view.tsx:266` | `"2 MiB memory mix"` | `2 MiB` | KovanicaHash PoW scratchpad size. |
| `src/lib/networkConstants.ts:20,22,23` | `FOUNDER_AMOUNT`, `HALVING_ERA`, `MAX_SUPPLY` | `200_000 * ATOM`, `8_100_000`, `90_200_000 * ATOM` | Tokenomics supply schedule constants. |
| `tests/browser-acceptance.test.ts:159`, `tests/wallet-sync.test.ts:21` | Mock balance `5_000_000_000` | `5 KVNC` (`5_000_000_000` atoms) | Deterministic test fixture balance for `kvnc8d4z00xwk6jz6c4r4rgz5mcdwdjny9thrh3y8f36cpy2rz6emg54rufktdag`. |
| `tests/fixtures/vectors.json` | Cryptographic test vectors | 3 vectors (`0000...`, `0001...`, `ffffffff...`) | ADR-0002 BIP-39 / Ed25519 / PRO-21 canonical derivation test vectors. |
