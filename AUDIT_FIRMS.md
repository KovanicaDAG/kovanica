# Recommended Audit Firms for KVNC

Based on the codebase characteristics (Mysticeti-style DAG consensus, Wasmi WASM runtime, libp2p networking, Rust/Ed25519/ECVRF cryptography), the following firms have relevant expertise:

---

## Tier 1 — Deep Consensus + Cryptography + Rust Expertise

| Firm | Relevant Experience | Why KVNC |
|------|---------------------|----------|
| **Trail of Bits** | Rust (soroban, reth, foundry), DAG consensus (Avalanche, IOTA), WASM (wasmi, wasmtime), libp2p, VRF/threshold crypto | Best-in-class for Rust + consensus + WASM; published research on DAG safety |
| **Sigma Prime** | Ethereum consensus (Lighthouse), libp2p, networking, Rust, formal verification | Deep libp2p & gossip expertise; Ethereum beacon chain experience transfers to DAG commit logic |
| **Zellic** | Solana (Turbine/PoH), Move/Aptos, Rust, WASM, high-throughput consensus | High-performance DAG-like consensus (Solana), WASM runtime security, fee market analysis |
| **Spearbit** | Modular audit marketplace; can assemble team with DAG + Rust + WASM + libp2p specialists | Flexible team composition; many ex-Trail/Sigma auditors; good for multi-phase |
| **OpenZeppelin** | Rust (soroban, fuel), WASM, cryptography, formal methods | Strong cryptography review; formal verification capabilities for consensus invariants |

---

## Tier 2 — Strong Rust/Blockchain, Good Consensus Coverage

| Firm | Relevant Experience |
|------|---------------------|
| **Halborn** | Rust, Substrate, Cosmos SDK, WASM, DeFi |
| **Certora** | Formal verification (CERTORA prover), Rust, consensus specs |
| **Runtime Verification** | K framework, formal verification, Rust, blockchain semantics |
| **Pashov Audit Group** | Rust, Solana, Move, high-throughput chains |
| **OtterSec** | Solana, Rust, WASM, tokenomics |

---

## Tier 3 — Specialized / Boutique

| Firm | Niche |
|------|-------|
| **Ackee Blockchain** | Rust, WASM, cryptography, consensus |
| **Dedaub** | Static analysis + manual, Rust, formal methods |
| **Veridise** | Formal verification (Viper/Verisol), Rust, Move |
| **Quantstamp** | Broad coverage, Rust, libp2p |

---

## Recommended Engagement Model

### Phase 1: Consensus + Crypto (4–6 weeks, 2–3 auditors)
**Scope:** `kvnc-consensus`, `kvnc-crypto`, `kvnc-types` (consensus types)
- Mysticeti commit logic, VRF leader election, GHOSTDAG k=3
- Ed25519 strict verify, ECVRF, vote/commit seal domain separation
- **Firms:** Trail of Bits, Sigma Prime, Zellic

### Phase 2: Mempool + P2P + Staking (3–4 weeks, 2 auditors)
**Scope:** `kvnc-mempool`, `kvnc-network`, `kvnc-staking`
- Fee market, RBF, in-flight timeout, re-insertion
- Gossipsub validation, peer scoring, block/state sync
- Emission schedule, slashing, delegation, treasury
- **Firms:** Sigma Prime (libp2p), Spearbit (modular team)

### Phase 3: WASM Runtime + Contracts + RPC (3–4 weeks, 2 auditors)
**Scope:** `kvnc-execution`, `kvnc-runtime`, `kvnc-rpc`, contract crates
- Wasmi fuel/memory metering, host ABI, trap handling
- Native contract logic (HTLC, Vault, Multisig, Token, LightNode)
- JSON-RPC rate limiting, auth, WS subscriptions
- **Firms:** Trail of Bits (WASM), Zellic (runtime)

### Phase 4: Integration + Cross-Cutting (2 weeks, 1–2 auditors)
**Scope:** `kvnc-node`, `kvnc-keystore`, cross-component invariants
- Genesis, recovery, committee build, chain_id pinning
- Keystore Argon2id/XChaCha20, v1 migration
- Supply accounting, execution idempotency
- **Firms:** Any from above

---

## Budget Estimates (Industry Ranges, 2026)

| Phase | Firms (Tier 1) | Firms (Tier 2) |
|-------|----------------|----------------|
| Phase 1 | $80k–$150k | $50k–$100k |
| Phase 2 | $60k–$120k | $40k–$80k |
| Phase 3 | $60k–$120k | $40k–$80k |
| Phase 4 | $30k–$60k | $20k–$40k |
| **Total** | **$230k–$450k** | **$150k–$300k** |

*Retest/verification typically 15–25% of original engagement.*

---

## Pre-Audit Preparation Checklist

- [ ] Freeze `main` branch at audit commit
- [ ] Provide `docs/AUDIT-SCOPE.md` + `AUDIT_REPORT.md` template
- [ ] Share `Cargo.lock` + dependency tree
- [ ] Document all environment variables / config options
- [ ] Provide genesis/testnet parameters
- [ ] Set up dedicated testnet for auditor validation
- [ ] Assign technical lead for auditor questions (Slack/Email)
- [ ] Prepare fuzzing corpus + CI artifacts

---

## Questions for Audit Firms (RFP)

1. **Consensus experience:** Have you audited Mysticeti / Narwhal-Tusk / GHOSTDAG / DAG-based consensus before? Provide references.
2. **VRF/ECVRF:** Experience with `ecvrf` crate or similar VRF implementations in consensus leader election?
3. **WASM runtime:** Audited `wasmi` or `wasmtime` host implementations? Gas metering, trap handling, reentrancy?
4. **libp2p/gossipsub:** Audited peer scoring, eclipse resistance, gossip validation layers?
5. **Rust tooling:** Use `cargo-audit`, `cargo-fuzz`, `miri`, `kani`, `cargo-contract`? Custom static analysis?
6. **Team composition:** Who would lead consensus review? Who leads WASM/runtime?
7. **Timeline:** Earliest start date for Phase 1?
8. **Retest policy:** Included in quote? Timebox?
9. **Report format:** Markdown + SARIF? Public disclosure timeline?
10. **References:** 2–3 recent Rust/consensus audits with public reports (or private references).

---

## Decision Matrix

| Criterion | Weight | Trail of Bits | Sigma Prime | Zellic | Spearbit | OpenZeppelin |
|-----------|--------|---------------|-------------|--------|----------|--------------|
| DAG consensus | 30% | ★★★★★ | ★★★★☆ | ★★★★★ | ★★★★☆ | ★★★★☆ |
| Rust/WASM | 25% | ★★★★★ | ★★★★☆ | ★★★★★ | ★★★★☆ | ★★★★★ |
| libp2p/P2P | 15% | ★★★★☆ | ★★★★★ | ★★★★☆ | ★★★★☆ | ★★★☆☆ |
| Cryptography | 15% | ★★★★★ | ★★★★☆ | ★★★★☆ | ★★★★☆ | ★★★★★ |
| Tokenomics/staking | 10% | ★★★★☆ | ★★★☆☆ | ★★★★☆ | ★★★★☆ | ★★★★☆ |
| Availability/timeline | 5% | ★★★☆☆ | ★★★★☆ | ★★★★☆ | ★★★★★ | ★★★☆☆ |
| **Weighted Score** | 100% | **4.7** | **4.3** | **4.5** | **4.3** | **4.4** |

*Subjective assessment — validate with RFP responses.*

---

## Next Steps

1. **Week 1:** Send RFP to top 3 firms (Trail of Bits, Sigma Prime, Zellic)
2. **Week 2:** Evaluate proposals, check references, negotiate scope/timeline
3. **Week 3:** Sign engagement letter, provide repository access
4. **Week 4:** Kickoff Phase 1

---

*Document prepared for KVNC audit preparation — not for public distribution.*
