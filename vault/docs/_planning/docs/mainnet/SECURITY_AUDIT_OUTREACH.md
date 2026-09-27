# Security Audit Outreach Package

## 1. Email Template

**To:** security@trailofbits.com, audit@nccgroup.com, security@kudelskisecurity.com
**Subject:** Security Audit RFP — Kovanica Protocol Mainnet Genesis (30-day engagement)

---

Dear [Firm Name] Team,

Kovanica Protocol is preparing for mainnet genesis in Q4 2026 and is seeking a 30-day security audit of our consensus-critical codebase.

**Project:** Kovanica Protocol — GHOSTDAG BlockDAG (k=3), UTXO ledger, Ed25519 signatures, native token KVNC
**Codebase:** ~50k LOC Rust (consensus-critical)
**Repos:** Private GitHub (KovanicaDAG/kovanica) — access granted post-NDA
**Target Start:** 2026-10-06
**Budget:** To be discussed based on scope

### In-Scope (Consensus-Critical Only)
- GHOSTDAG consensus (k=3) — blue score, selected parent, anticone, reorg bounds
- UTXO ledger & transaction validation
- RFC-006 tokenomics enforcement (MAX_SUPPLY 90.2M KVNC, 100-block maturity, 75% fee burn, subsidy curve)
- P2P gossip & sync (headers-first, full-dump with topological ordering)
- Block production (PoA + PoW transition)
- Script VM & sighash (Ed25519, no malleability)
- RFC features: Multisig (KVP-101), Multi-asset (KVP-102), Stealth+Script v2 (KVP-103), HTLC (KVP-104), Vaults/CSV (KVP-105)

### Out of Scope
- Explorer web frontend (TypeScript/TanStack)
- Wallet UI / CLI UX
- Documentation

### Deliverables
1. Kickoff call (1h)
2. Threat model (STRIDE per component) — Day 7
3. Interim findings — Day 15
4. Final report (PDF + Markdown) — Day 30
5. Fix verification — Day 37

### Code Access
- Private repo: github.com/KovanicaDAG/kovanica (branch: fix/secret-scan-working-tree)
- Testnet live at: https://testnet.kovanica.online (PoA, height ~100+)
- Reproducible build instructions provided

### Recent Fixes to Verify
- `net.rs:apply_decoded` topological sort (prevents parent-delta-missing panic)
- `node.rs:verify_header_body` PoA authority_sig handling (was using Block::new instead of Block::new_with_authority)

### Contact
Technical Lead: [Name] — [Signal/Email]
Project Coordinator: [Name] — [Signal/Email]

All communications encrypted. NDA executed before repo access.

Please confirm interest and availability by **2026-10-03**.

---

## 2. NDA Template

```
MUTUAL NON-DISCLOSURE AGREEMENT

This Mutual Non-Disclosure Agreement ("Agreement") is entered into as of [Date] by and between:

Kovanica Foundation ("Disclosing Party")
[Firm Name] ("Receiving Party")

1. PURPOSE: Security audit of Kovanica Protocol consensus codebase for mainnet genesis.

2. CONFIDENTIAL INFORMATION: All non-public code, architecture, keys, and findings.

3. OBLIGATIONS: Receiving Party shall:
   - Use Confidential Information only for the audit
   - Protect with same care as own confidential info (min. reasonable care)
   - Not disclose to third parties without written consent
   - Return/destroy upon completion or request

4. EXCEPTIONS: Public info, independently developed, rightfully received from third party.

5. TERM: 2 years from disclosure.

6. NO LICENSE: No IP rights granted.

7. GOVERNING LAW: [Jurisdiction]

8. SIGNATURES:

Kovanica Foundation                    [Firm Name]
By: _________________________          By: _________________________
Name: _________________________        Name: _________________________
Title: _________________________       Title: _________________________
Date: _________________________        Date: _________________________
```

---

## 3. Technical Briefing Document

### Architecture Overview
- **Consensus:** GHOSTDAG k=3 (BlockDAG, not blockchain)
- **Ledger:** Pure UTXO, no accounts
- **Signatures:** Ed25519 (64-byte, 128 hex)
- **Token:** KVNC (1 KVNC = 100M atoms)
- **P2P:** Plaintext TCP:9000 (no libp2p), DNS seed: seed.kovanica.online:9000

### Consensus-Critical Files
| File | Lines | Purpose |
|------|-------|---------|
| `kovanica-dag/src/ghostdag.rs` | ~2000 | GHOSTDAG ordering |
| `kovanica-dag/src/dag.rs` | ~1500 | DAG insertion, PoA |
| `kovanica-dag/src/authority.rs` | ~1000 | PoA authority set, SW-PoA |
| `kovanica-state/src/validation.rs` | ~2000 | TX validation |
| `kovanica-state/src/ledger.rs` | ~3000 | UTXO ledger |
| `kovanica-node/src/node.rs` | ~4000 | Block production, RPC |
| `kovanica-node/src/net.rs` | ~1000 | P2P sync |

### Key Invariants to Verify
1. **MAX_SUPPLY ≤ 90.2M KVNC** (9_020_000_000_000_000 atoms) at all heights
2. **Coinbase maturity = 100 blocks** — immature spends rejected
3. **Fee burn = 75%** — exactly 75% burned, 25% to producer
4. **Subsidy curve** — s₀=10 KVNC, era=2M blocks, α=3/4 geometric decay
5. **Fee floor** — max(1, subsidy/500_000) atoms/byte
6. **GHOSTDAG k=3** — blue score, selected parent, anticone bounds
6. **PoA admission** — authority sig verification, slot scheduling, threshold enforcement

### How to Run Tests
```bash
cd protocol
cargo test -p kovanica-dag -p kovanica-state -p kovanica-node
```

### Build Reproducibly
```bash
docker build -t kovanica-node -f Dockerfile .
# Verify SHA256 matches published hash
```

### Testnet Access
- Explorer: https://testnet.kovanica.online
- API: https://testnet.kovanica.online/api/head
- WebSocket: wss://testnet.kovanica.online/ws