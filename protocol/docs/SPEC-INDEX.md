# Kovanica Specification Index

> **Purpose:** Single reference for all KVP (Kovanica Protocol) specifications and RFCs.
> Updated: 2026-09-20 | Network: `kovanica-testnet` | Status: Testnet (mainnet dormant)

---

## KVP Specifications

| KVP | Title | Status | Activation | Related RFC | Key Documents |
|-----|-------|--------|------------|-------------|---------------|
| **KVP-101** | Multisig (M-of-N P2SH) | ✅ Active | Blue score 0 | [RFC-001](RFC-001-Multisig.md) | Spec, tests, address format `kvnc...dag` (version 0x01) |
| **KVP-102** | Native Tokens (Multi-asset) | ✅ Active | Blue score 0 | [RFC-002](RFC-002-NativeTokens.md) | Spec, 1-byte asset flag, per-asset conservation, fee in native KVNC |
| **KVP-103** | Stealth Addresses + Script v2 | ✅ Active | Blue score 0 | [RFC-003](RFC-003-ScriptV2-and-Stealth.md) | Version 0x02 (script), 0x03 (stealth), ECDH one-time keys, CLTV/CSV/threshold ops |
| **KVP-104** | HTLC / Atomic Swaps | ✅ Active | Blue score 0 | [RFC-004](RFC-004-Htlc.md) | Version 0x04 (HTLC), Tier Nolan atomic swap, 100-byte template |
| **KVP-105** | Vault / CSV (Relative Locktime) | ✅ Active | Blue score 0 | [RFC-005](RFC-005-Vault.md) | Version 0x05 (Vault), per-UTXO creation height, checkpoint v6 |

---

## RFC Documents

| RFC | Title | KVP | Status | Document | Implementation |
|-----|-------|-----|--------|----------|----------------|
| **RFC-001** | Multisig (M-of-N P2SH) | KVP-101 | ✅ Shipped | [RFC-001-Multisig.md](RFC-001-Multisig.md) | `kovanica-state/src/multisig.rs`, 35-test suite |
| **RFC-002** | Native Tokens (Multi-asset) | KVP-102 | ✅ Shipped | [RFC-002-NativeTokens.md](RFC-002-NativeTokens.md) | `kovanica-state/src/tx.rs`, `utxo.rs`, `ledger.rs`, 29-test suite |
| **RFC-003** | Stealth + Script v2 | KVP-103 | ✅ Shipped | [RFC-003-ScriptV2-and-Stealth.md](RFC-003-ScriptV2-and-Stealth.md) | `kovanica-state/src/script_v2.rs`, `stealth.rs`, 25-test suite |
| **RFC-004** | HTLC / Atomic Swaps | KVP-104 | ✅ Shipped | [RFC-004-Htlc.md](RFC-004-Htlc.md) | `kovanica-state/src/htlc.rs`, `kovanica-node/src/atomic_swap.rs`, 23-test suite |
| **RFC-005** | Vault / CSV | KVP-105 | ✅ Shipped | [RFC-005-Vault.md](RFC-005-Vault.md) | `kovanica-state/src/vault.rs`, per-UTXO creation height, 26-test suite |
| **RFC-006** | Tokenomics (Emission Curve) | — | ✅ Core Done | [RFC-006-EmissionCurve.md](RFC-006-EmissionCurve.md) | `kovanica-state/src/ledger.rs`, smooth α=¾, MAX_SUPPLY 90.2M |

---

## Address Version Registry

| Version | Name | Payload | Rendering | Activation |
|---------|------|---------|-----------|------------|
| `0x00` | P2PK (Single-key) | 32-byte Ed25519 pk | `kvnc...dag` (base58) | Genesis |
| `0x01` | P2SH (Multisig) | BLAKE3(redeem_script) | `kvnc...dag` | RFC-001 (score 0) |
| `0x02` | Script v2 | BLAKE3(script_bytes) | `kvnc...dag` | RFC-003 (score 0) |
| `0x03` | Stealth (published) | scan_pk \|\| spend_pk | 65-byte hex | RFC-003 (score 0) |
| `0x03` | Stealth (on-chain) | BLAKE3(scan_pk \|\| spend_pk) | `kvnc...dag` | RFC-003 (score 0) |
| `0x04` | HTLC | BLAKE3(template) | `kvnc...dag` | RFC-004 (score 0) |
| `0x05` | Vault | BLAKE3(template) | `kvnc...dag` | RFC-005 (score 0) |

**Note:** All non-P2PK addresses share the same `kvnc...dag` rendering (version byte + 32-byte hash). The version byte discriminates the address type at parse time.

---

## Consensus Parameters (Testnet)

| Parameter | Value | Source |
|-----------|-------|--------|
| GHOSTDAG **k** | 3 | `dag.rs` |
| Finality depth | 100 blocks | `ledger.rs` |
| Payload pruning depth | 1000 blocks | `dag.rs` |
| PoW | Opt-in, real | `pow.rs` |
| Difficulty | Opt-in, enforced | `difficulty.rs` |
| VRF | Opt-in, leader selection | `vrf.rs` |
| Hybrid admission | Opt-in (PoW + VRF-staked) | `ledger.rs` |

**All RFC activation scores = 0 (active from genesis on testnet).**

---

## Checkpoint Versions

| Version | Added | Description |
|---------|-------|-------------|
| v1 | Genesis | Basic DAG + UTXO |
| v2 | RFC-001 | Stake registry |
| v3 | RFC-001 | Stake registry length-prefixed |
| v4 | RFC-002 | Native token asset_id in UTXO |
| v5 | RFC-003 | Stealth flag + 65-byte extension |
| v6 | RFC-005 | Per-UTXO creation height (8 bytes) |
| **v7** | **RFC-006 + Operator Wallet** | **Current (operator wallet + treasury)** |

---

## Network & Deployment

| Network | Genesis Hash | Status | Seeds |
|---------|--------------|--------|-------|
| `kovanica-testnet` | `9565fc20cb465eec0198a65c07da6b825e4211c4060d581a2c7dac6c96bafc97` | **Active** | `seed.kovanica.online:9000`, `seed2.kovanica.online:9000` |
| `kovanica-mainnet` | TBD | Dormant | — |

**P2P Port:** 9000 (TCP, grey-cloud DNS only)

**DNS Seeds:** `seed.kovanica.online`, `seed2.kovanica.online`

---

## API & Client References

| Component | Interface | Document |
|-----------|-----------|----------|
| Line RPC | `execute_line` (stdin/stdout) | `kovanica-node/src/rpc.rs` |
| Explorer HTTP API | REST + WebSocket | `kovanica-node/src/explorer.rs` |
| FFI (UniFFI) | Kotlin / Swift bindings | `kovanica-ffi/src/light_node.rs` |
| SPV Light Sync | KVLS v1 blob + filters | `kovanica-node/src/spv.rs` |

---

## Related Documents

| Document | Purpose |
|----------|---------|
| [SECURITY.md](SECURITY.md) | Threat model, key handling, finality, incident response |
| [TOKENOMICS.md](TOKENOMICS.md) | Emission curve, supply parameters, fees |
| [NETWORK.md](../NETWORK.md) | Domain map, DNS, redirect rules |
| [OPERATIONS.md](../OPERATIONS.md) | Seed runbook, deploy pipeline, incident lessons |
| [LEGIT-BOARD.md](LEGIT-BOARD.md) | Public visibility checklist |
| [AUDIT-PLAN.md](AUDIT-PLAN.md) | Audit scope, firms, timeline |
| [MAINNET-CRITERIA.md](MAINNET-CRITERIA.md) | Exit checklist for mainnet launch |

---

## Quick Links (GitHub)

- **Protocol repo:** https://github.com/KovanicaDAG/kovanica-protocol
- **Node repo (mirror):** https://github.com/KovanicaDAG/kovanica-node
- **Web repo:** https://github.com/KovanicaDAG/kovanica-web
- **Security Advisories:** https://github.com/KovanicaDAG/kovanica/security/advisories
- **Issue Templates:** Bug / Feature / Security
- **Releases:** https://github.com/KovanicaDAG/kovanica/releases

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2026-09-20 | 1.0 | Initial spec index (Legit v1 bundle) |

---

*This document is part of the **Legit v1** public visibility bundle (C8). Authoritative source lives in `protocol/docs/SPEC-INDEX.md` and is mirrored byte-identical in the Obsidian vault.*