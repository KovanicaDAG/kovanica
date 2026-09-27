# Kovanica Protocol — Master Roadmap

**Version:** 0.9.1 (consistency pass 2026-09-23; monorepo import)  
**Date:** September 2026  
**Status:** Pre-Mainnet  
**Style:** Living internal plan

> **Doc version 0.9.x** is the planning document revision.  
> **Crate version** (`kovanica-sdk` 0.1.0-alpha.*) is independent and stays on 0.x until protocol 1.0.0.

---

## 1. Purpose Statement

> Kovanica is a high-performance, quantum-aware BlockDAG built for real-world payments, programmable assets, and sovereign financial infrastructure.

The network prioritises practical adoption (merchant payments first), developer experience, and long-term security hardening while remaining under clear Founder / Foundation stewardship during the critical early years.

---

## 2. Master Roadmap — Prioritised Phases

### PHASE 0 — Close & Secure (Now → Mainnet)

- Complete RFC-006 (tokenomics) — residual gauges, maturity/halving tests, merge & tag
- Testnet reset with new genesis; update TESTNET.md + public docs
- **Lock address format** (`kvnc`+base58+`dag`) and **sighash** (BLAKE3 witness-free) to match `kovanica-node`
- 12 / 24-word BIP-39 backup support in wallet + CLI (uses locked derivation) — **Done (M-01…M-11)**
- `kovanica-sdk` workspace scaffold (**done**) + core crates aligned to node encoding
- Create dedicated `kovanica-testnet` and `kovanica-mainnet` repositories (docs, configs, metrics, genesis)
- Publish clear Founder / Team page + social links
- Semantic versioning discipline: 0.x.y only until mainnet stabilises → 1.0.0

### PHASE 1 — Developer Experience & Legal Base (0–3 months post-mainnet)

- Kovanica Foundation legal entity (jurisdiction TBD — EU/Croatia preferred)
- On-chain treasury + multi-sig + vesting schedule for Founder allocation
- `kovanica-sdk` publish (crates.io + npm/WASM), cookbook, examples against live network
- Playground / Console (web IDE for assets, HTLC, multisig, fee simulation)
- Minimal merchant payment request + QR + webhook (`kovanica-pay` MVP)
- Public node-runner docs + Grafana dashboards + small incentive program

### PHASE 2 — Real Economy (3–12 months)

- Full merchant payment stack (WooCommerce / Shopify plugins, POS app)
- HTLC atomic-swap bridges (start with BTC + ETH)
- Token listings, transparent presales, airdrops to early stakers/miners
- NFT marketplace surface (KVP-106)
- Developer Grant Program (50–200k KVNC ranges)
- Light client / SPV release

### PHASE 3 — Expansion & Hardening (12–36 months)

- Post-quantum signature abstraction + migration path (Ed25519 → hybrid Dilithium/Falcon)
- Additional bridges (XRP, SOL, DOGE as demand justifies)
- RWA + healthcare / art / business verticals
- Institutional / banking rails exploration
- Formal governance evolution (soft → on-chain proposals)
- World-tier brand, audits, security disclosures, bug bounty

---

## 3. Technical Backlog Summary

See: **TECHNICAL-BACKLOG.md** and **TASK-BREAKDOWN-BIP39-SDK.md**

| Workstream       | Priority     | Target          | Depends On                         | Status        |
|------------------|--------------|-----------------|------------------------------------|---------------|
| Address + sighash lock | P0 Critical | Before broadcast     | Node vectors + encode_into     | Partial       |
| 12/24 BIP-39 backup | P0 Critical  | Before mainnet  | Address/derivation lock            | **Done**      |
| kovanica-sdk scaffold | P0        | Pre-mainnet     | —                                  | **Done**      |
| kovanica-sdk core + publish | P0   | Mainnet + 30d   | Address lock, stable RPC           | In progress   |
| Playground       | P1 High      | Mainnet + 60d   | SDK + faucet + testnet             | Todo          |

**Canonical address:** `kvnc`+base58+`dag` (see `ADDRESS-AND-SIGHASH-SPEC.md`). P2PK codec in SDK; full wire parity pending.

---

## 4. Foundation & Legal Skeleton Summary

See: **docs/foundation/FOUNDATION-LEGAL-SKELETON.md**

**Core principle:** Progressive decentralisation under an accountable Foundation during early years.

- Legal entity: Kovanica Foundation (Croatia / EU preferred)
- Founder = sole initial Board member + protocol steward
- Assets held by Foundation: domains, trademarks, GitHub orgs, treasury, brand
- On-chain treasury: multi-sig + RFC-005 vaults
- No private seed mining → market selling

---

## 5. Explicit Non-Goals / Risks to Avoid

- Centralised "seed mining then sell on our DEX" optics
- Over-promising bridges to every chain in year one
- Marketing "world-tier asset" before product-market fit and audits
- Mixing personal and Foundation wallets
- Skipping legal entity until after mainnet
- Shipping SDK that produces non-consensus addresses or signatures

---

## 6. Immediate Next Actions (Ordered)

1. Finish RFC-006 residual work + tag release  
2. **Finish address + sighash parity** with node vectors / `encode_into` (S-03b, S-03c)  
3. Complete SDK core S-02 → S-08 against locked encoding  
4. Spin up `kovanica-testnet` + `kovanica-mainnet` repos  
5. Engage Croatian/EU counsel for Foundation setup  
6. Publish Founder page + this roadmap publicly  

---

*Living plan. Consistency pass: see `CONSISTENCY-FIXES.md`.*