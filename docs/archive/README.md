# Kovanica Documentation Archive

This directory contains historical documentation, completed rollout plans, incident runbooks, superseded draft skeletons, and finalized technical investigation reports that have served their original purpose.

In accordance with requirement **R2** ("Archive obsolete material"), no files are deleted. Instead, they are preserved here in `docs/archive/` for historical context, forensic auditability, and institutional memory.

---

## Archived Documents Index

| Archived Document | Original Date / Phase | Status | Primary Reason for Archiving | Superseding Active Reference |
|---|---|---|---|---|
| [`RENAME-CUTOVER-PLAN.md`](./RENAME-CUTOVER-PLAN.md) | 2026-10-15 | Completed | One-off operational rollout plan for the testnet rename cutover to Kovanica (`KOVANICA_*`, `@kovanica/*`). The cutover was fully executed and finalized on 2026-10-16. | Master codebase, `AGENTS.md`, and `README.md` |
| [`SEED-RECOVERY-RUNBOOK.md`](./SEED-RECOVERY-RUNBOOK.md) | 2026-10-15 | Resolved Incident | Specific incident response runbook for recovering validator quorum on the stalled testnet at commit `d0a26b11`. Root causes (VRF leader-rotation integer saturation, round continuation on restart, vote gossip transport) were permanently fixed in protocol code. | `docs/TESTNET-RESET.md`, `crates/kovanica-consensus/` |
| [`DEV-STATUS-AND-TASKS.md`](./DEV-STATUS-AND-TASKS.md) | 2026-10-13 | Superseded | Historical point-in-time developer snapshot taken at `main` @ `98d88cc` (19 crates, 3 TS packages). Superseded by subsequent development and the master roadmap. | `TASKLIST.md`, `README.md`, `AGENTS.md` |
| [`VRF-PROOF-LENGTH.md`](./VRF-PROOF-LENGTH.md) | 2026-10-21 | Finalized Analysis | Technical investigation report resolving TASKLIST item 15 regarding ECVRF proof length (96 B ristretto255 + SHA3 vs 80 B RFC 9381 Edwards25519). The analysis concluded that 96 B is genuine and correct for the `ecvrf` crate. | `crates/kovanica-crypto/src/vrf.rs`, `docs/AUDIT-SCOPE.md` |
| [`A8.2-DELEGATION-FULL.md`](./A8.2-DELEGATION-FULL.md) | Phase 18 Sprint | Implemented | Early sprint scratchpad and implementation checklist for Phase 18 delegation lifecycle tasks. All tasks (delegation, unbonding, slashing evidence, deterministic epoch committee rotation) are completed and tested in code. | `crates/kovanica-staking/`, `docs/GOVERNANCE.md` |
| [`B-TREASURY-PROPOSAL.md`](./B-TREASURY-PROPOSAL.md) | Phase 25 Early Draft | Superseded | Preliminary 38-line skeleton outlining treasury spend proposal fields. Completely replaced and expanded by the comprehensive Phase 25 Governance Specification. | `docs/GOVERNANCE.md` |
| [`CANONICAL-TESTNET-2.md`](./CANONICAL-TESTNET-2.md) | 2026-11 (M8 → canonical testnet) | Implemented | Point-in-time migration design that promoted Chain 2 to the canonical Hybrid PoW + GHOSTDAG testnet and retired the isolated Chain 5 design. Fully implemented; later commits removed the remaining Chain 5 / Chain 4 code paths. | `crates/kovanica-types/src/signing.rs`, `docs/dev/HYBRID-TESTNET-M8.md` |
| [`AUDIT_REPORT_2026.md`](./AUDIT_REPORT_2026.md) | 2026-10-24 (base `dd3aff9`) | Historical | Point-in-time documentation/config audit of the corpus at commit `dd3aff9`. Its registry and address tables describe the pre-cleanup chain set (including chain 4), which no longer matches the code. | `docs/AUDIT-SCOPE.md`, `README.md` |

---

## Detailed Archival Rationales

### 1. `RENAME-CUTOVER-PLAN.md`
- **Context**: During the transition to the Kovanica network branding, a comprehensive multi-phase cutover plan was drafted covering crate renames, environment variable updates (`KOVANICA_*`), npm package namespace migration (`@kovanica/*`), and seed node redeployments.
- **Archival Rationale**: All migration phases (D1 through D5) were 100% completed and verified by 2026-10-16. Retaining the migration plan in active documentation caused ambiguity regarding current operational procedures.

### 2. `SEED-RECOVERY-RUNBOOK.md`
- **Context**: On 2026-10-15, the early public testnet encountered a quorum stall on commit `d0a26b11`. This runbook guided the manual restart and state injection of seed validators.
- **Archival Rationale**: The underlying consensus edge cases (VRF integer saturation during round rotation, late-join vote sync, and round continuation state persistence) were solved in the protocol engine (see RFC-009 and RFC-011). Routine operational procedures are documented in `docs/TESTNET-RESET.md` and `docs/TESTNET-SEED-NODES.md`.

### 3. `DEV-STATUS-AND-TASKS.md`
- **Context**: Authored as an intermediate status summary on 2026-10-13 when the repository comprised 19 crates and 3 TypeScript packages.
- **Archival Rationale**: Completely superseded by subsequent architectural expansion (now 27 crates, 5 packages, 6 apps) and the active `TASKLIST.md` tracking document.

### 4. `VRF-PROOF-LENGTH.md`
- **Context**: Detailed cryptographic evaluation comparing the 96-byte proof produced by the `ecvrf` 0.4.3 crate against the 80-byte proof defined in RFC 9381.
- **Archival Rationale**: The verdict was confirmed on 2026-10-21 (TASKLIST item 15): Kovanica uses the Ristretto255 + SHA3-512 construction whose proof is `Gamma(32) || c(32) || s(32) = 96` bytes. This is frozen and normative; the exploratory research report is archived for reference.

### 5. `A8.2-DELEGATION-FULL.md`
- **Context**: Informal development scratchpad for the Phase 18 delegation lifecycle sprint.
- **Archival Rationale**: All tracked implementation items (bond/unbond state machines, reward distribution, validator commission, and double-signing evidence) are fully merged and tested in `kovanica-staking` and `kovanica-execution`.

### 6. `B-TREASURY-PROPOSAL.md`
- **Context**: Initial 38-line draft skeleton exploring the structure of on-chain treasury proposals.
- **Archival Rationale**: Replaced in full by `docs/GOVERNANCE.md`, which defines the complete governance lifecycle, stake-weighted voting thresholds, and execution mechanisms.

### 7. `CANONICAL-TESTNET-2.md`
- **Context**: Migration design (status "Implemented — Option A") that proposed promoting Chain 2 to the canonical Hybrid PoW + GHOSTDAG testnet, introducing the isolated Chain 5 testnet track, and inventorying every chain-id-dependent code path at a fixed commit.
- **Archival Rationale**: The migration is complete and the design has since been superseded: Chain 2 is the canonical hybrid testnet, and the Chain 5 and Chain 4 code paths were removed entirely. The document is a point-in-time snapshot (it cites line numbers and code snippets that no longer exist), so it is preserved here for historical context rather than kept as active guidance. Current hybrid behaviour is described in `docs/dev/HYBRID-TESTNET-M8.md` and `docs/MINING.md`.

### 8. `AUDIT_REPORT_2026.md`
- **Context**: A 2026-10-24 audit of the documentation corpus, example configs, and repository identity, recorded against base commit `dd3aff9`.
- **Archival Rationale**: The report is a point-in-time snapshot: it cites commit `dd3aff9`, line ranges, and a chain registry that predates the canonical cleanup (it still lists chain 4 as active). The findings were acted on at the time; the roster is now authoritative in `AGENTS.md`, `README.md`, and the code. Preserved here for forensic auditability rather than as live guidance.
