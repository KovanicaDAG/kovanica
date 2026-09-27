# Kovanica Hybrid PoW/PoS + VRF + SPV — Document Package

**RFC:** RFC-008 (proposed public name KVP-107)  
**Date:** 2026-09-17  
**Audience:** Core developers, node operators, light-client / wallet implementers, project planners

This zip contains the design and planning artefacts needed to change Kovanica consensus from pure PoW GHOSTDAG to **Hybrid PoW/PoS admission** with **VRF sortition** and a first-class **light-node / SPV path**.

---

## Contents

| File                              | Purpose |
|-----------------------------------|---------|
| `RFC-008-HYBRID-POW-POS-VRF.md`   | Formal RFC draft — motivation, rules, parameters, activation, security |
| `HYBRID-TECHNICAL-DESIGN.md`      | Engineering blueprint (header, VRF, fixed work, stake ledger, node loops, testing) |
| `LIGHT-NODE-SPV-AND-VRF.md`       | Light-client verification algorithm, data requirements, UniFFI notes, finality UX |
| `HYBRID-PROJECT-PLAN.md`          | Layered decomposition, milestone table, risk register, sequencing vs RFC-006 |
| `HYBRID-CHECKLIST.md`             | Concrete implementation & activation checklist (copy into tracking issue) |
| `README.md`                       | This file |

---

## Critical reading order

1. **RFC-008** — understand the consensus rules and non-goals.
2. **HYBRID-PROJECT-PLAN** — see the required sequencing (especially the hard gate on RFC-006).
3. **HYBRID-TECHNICAL-DESIGN** — implementers start here.
4. **LIGHT-NODE-SPV-AND-VRF** — anyone building mobile / browser light clients.
5. **HYBRID-CHECKLIST** — day-to-day tracking.

---

## Design summary (one paragraph)

GHOSTDAG(k=3) and blue-work remain the sole chain-selection mechanism. Blocks may now be admitted either by classical PoW or by a VRF proof of eligibility derived from locked stake. Staked blocks receive a **fixed nominal work** value so they cannot cheaply inflate blue weight. Light-nodes follow the selected-parent header chain, verify VRF proofs, and use blue-score depth as the finality signal. All existing RFC-006 supply, maturity and fee-burn rules apply equally to both block types. Activation is a hard fork (testnet reset recommended while the network is still early).

---

## Hard prerequisites before any consensus code

- RFC-006 tokenomics fully activated and residual gaps closed.
- Stable pure-PoW testnet with deterministic tips.
- Spec freeze (RFC-008 + Technical Design accepted).

Starting hybrid work earlier creates two concurrent consensus forks and is strongly discouraged.

---

## Suggested next actions

1. Review and comment on the open questions in RFC-008 §10.
2. Freeze the initial consensus parameter table.
3. Land `kovanica-vrf` (pure crypto, zero consensus risk) as the first code artefact.
4. Only then open header / admission changes in `kovanica-dag`.

---

## Related existing artefacts

- `ghostdag-notes.md` (skill) — already records Stage-3 hybrid admission vision
- RFC-005 — vaults (recommended stake lock mechanism)
- RFC-006 — tokenomics / MAX_SUPPLY / maturity (must stay intact)
- Project plan playbook — three-layer (consensus / ledger / client) discipline

---

## Licence / status

Internal design documents for the Kovanica protocol. Treat as Draft until the RFC is marked Final and parameters are frozen for an activation.
