---
title: "Kovanica Protocol Standards (KVP)"
category: 10-Protocol
source: protocol/docs/KVP.md
synced: 2026-09-26
---
# Kovanica Protocol Standards (KVP)

**KVP** (*Kovanica Protocol*) is the public numbering for consensus and ledger
standards on Kovanica. Each **KVP-N** maps 1:1 to an internal **RFC** document
that remains the normative engineering reference.

Numbering uses a **100 + RFC ordinal** scheme so public labels read closer to
familiar token-standard brands (e.g. ERC-20) while staying tied to the RFC series:

| KVP | RFC | Title | Status |
|-----|-----|-------|--------|
| **KVP-101** | [[10-Protocol/protocol--docs--RFC-001-Multisig|RFC-001]] | Multisig (M-of-N P2SH) | Shipped |
| **KVP-102** | [[10-Protocol/protocol--docs--RFC-002-NativeTokens|RFC-002]] | Native multi-asset tokens | Shipped |
| **KVP-103** | [[10-Protocol/protocol--docs--RFC-003-ScriptV2-and-Stealth|RFC-003]] | Stealth addresses + script v2 | Shipped |
| **KVP-104** | [[10-Protocol/protocol--docs--RFC-004-Htlc|RFC-004]] | HTLC atomic swaps | Shipped |
| **KVP-105** | [[10-Protocol/protocol--docs--RFC-005-Vault|RFC-005]] | Time-lock vault / escrow (real CSV) | Shipped |

## Native coin vs token standard

- **KVNC** — native currency of the ledger (`asset_id = None` on outputs).
- **KVP-102** — the *standard* for non-native (and the rules governing all
  multi-asset outputs), not a second ticker.

> KVNC is native. Other assets are **KVP-102** assets under RFC-002.

## Documents

| Public standard | Normative RFC | Notes |
|-----------------|---------------|-------|
| [[10-Protocol/protocol--docs--KVP-102-NativeTokens|KVP-102]] | [[10-Protocol/protocol--docs--RFC-002-NativeTokens|RFC-002]] | Token / multi-asset standard |

Additional **KVP-10x** overview pages may be added for 101 / 103 / 104; until
then the RFC file is the full specification.

## Compatibility note

KVP-102 is **not** an ERC-20 (or BEP-20) contract interface. Value lives in the
UTXO set with an optional 32-byte `asset_id`. Fees are always paid in **KVNC**.
