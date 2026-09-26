---
title: "Testing Plan for PoA Migration"
category: 60-Planning
source: kovanica-poa-migration/07-TESTING-PLAN.md
synced: 2026-09-26
---
# Testing Plan for PoA Migration

## 1. Unit tests

- [ ] `AuthoritySet::active(slot)` returns correct key for many slots
- [ ] Authority signature verification accepts valid sig, rejects wrong key / wrong slot
- [ ] Authority Update transaction:
  - valid with sufficient signatures
  - rejected with insufficient signatures
  - rejected when signatures are from non-members
  - correctly replaces the live Authority UTXO
- [ ] Slot calculation with different `SLOT_DURATION` values
- [ ] Block with correct authority sig is accepted by the PoA validator
- [ ] Block with wrong authority sig is rejected
- [ ] Old PoW blocks are rejected after PoA activation (if using activation height)

## 2. Integration / multi-node tests

- [ ] 3-validator private network produces blocks in round-robin order
- [ ] 4-validator network tolerates one offline validator (liveness)
- [ ] Authority Update is accepted and subsequent blocks use the new set
- [ ] GHOSTDAG colouring and selected chain remain correct under PoA
- [ ] Re-org / parallel blocks still handled by GHOSTDAG as before
- [ ] SPV client can verify a transaction against PoA headers
- [ ] State (UTXO set) after many blocks matches expected balances

## 3. Resource tests (important for original goal)

- [ ] CPU usage of a non-producing node stays near idle
- [ ] CPU usage of a producing validator is low (only signing + gossip)
- [ ] RAM footprint compared with old PoW mode (should be noticeably lower)
- [ ] Disk growth rate unchanged (or better, thanks to no mining side effects)

## 4. Acceptance criteria

Migration is considered successful when:

1. A clean 3- or 4-validator network runs stably for ≥ 24 h.
2. Authority set can be updated on-chain and the change takes effect.
3. Existing transaction types (transfer, multisig, multi-asset, stealth, HTLC, vault) continue to work.
4. Light / SPV verification works with the new signature rule.
5. Measurable reduction in CPU and RAM versus the previous PoW + VRF mode.

## 5. Rollback plan

- Keep the old consensus path behind a feature flag / env var.
- In case of serious issues, restart the testnet with `KOVANICA_CONSENSUS=pow` (or perform a new genesis).
