# kovanica-brain-vault

> **Canonical Knowledge Base for Kovanica Protocol** — Obsidian vault (layers 00–06) containing all protocol specs, RFCs, operational runbooks, design decisions, and institutional memory. **Private repository.**

---

## ⚠️ Access Restricted

This repository is **private** and contains:
- Unpublished RFC drafts
- Security-sensitive operational procedures
- Mainnet authority-set governance discussions
- Commercial/partnership documentation
- Internal design decision logs

**Do not attempt to clone without explicit authorization.**

---

## Vault Structure (Obsidian Layers)

```
kovanica-brain-vault/
├── 00-Foundation/           # Immutable axioms
│   ├── consensus-invariants.md
│   ├── cryptographic-primitives.md
│   ├── tokenomics-axioms.md
│   └── network-assumptions.md
├── 10-Protocol/             # Core protocol specifications
│   ├── GHOSTDAG/
│   │   ├── k-parameter.md
│   │   ├── mergeset-theory.md
│   │   ├── linearization-proof.md
│   │   └── adversarial-analysis.md
│   ├── UTXO-Ledger/
│   │   ├── state-model.md
│   │   ├── per-block-state.md
│   │   ├── finality-pruning.md
│   │   └── snapshot-format.md
│   ├── Admission/
│   │   ├── PoA-authority-set.md
│   │   ├── slot-schedule.md
│   │   ├── authority-rotation.md
│   │   └── PoW-removal-log.md
│   └── P2P/
│       ├── seed-policy.md
│       ├── eclipse-resistance.md
│       ├── DHT-design.md
│       └── gossip-protocol.md
├── 20-RFCs/                 # All RFCs (shipped + draft)
│   ├── RFC-001-Multisig.md
│   ├── RFC-002-NativeTokens.md
│   ├── RFC-003-Stealth-ScriptV2.md
│   ├── RFC-004-HTLC.md
│   ├── RFC-005-Vault-CSV.md
│   ├── RFC-006-Tokenomics.md
│   ├── RFC-007-NFT.md
│   ├── RFC-POA-Migration.md
│   └── RFC-008-...md
├── 30-Operations/           # Runbooks & procedures
│   ├── seed-operations.md
│   ├── testnet-reset-playbook.md
│   ├── mainnet-launch-checklist.md
│   ├── incident-response.md
│   ├── backup-restore.md
│   ├── monitoring-alerting.md
│   └── deploy-procedures.md
├── 40-Design/               # Design decisions & trade-offs
│   ├── ADR-001-GHOSTDAG-k3.md
│   ├── ADR-002-UTXO-vs-Account.md
│   ├── ADR-003-Ed25519-only.md
│   ├── ADR-004-PoA-over-PoW.md
│   ├── ADR-005-No-libp2p.md
│   └── ADR-006-...
├── 50-Commercial/           # Partnership & commercial docs
│   ├── RWA-framework.md
│   ├── exchange-integration.md
│   ├── custody-partners.md
│   └── enterprise-features.md
├── 60-Meta/                 # Project governance
│   ├── roadmap.md
│   ├── team.md
│   ├── contributors.md
│   ├── security-policy.md
│   └── trademark-guidelines.md
└── templates/               # Document templates
    ├── RFC-template.md
    ├── KVP-template.md
    ├── ADR-template.md
    └── runbook-template.md
```

---

## Sync to Other Systems

| Target | Method | Frequency |
|--------|--------|-----------|
| `kovanica-agent` (RAG) | Git pull + markdown parsing | On-demand / webhook |
| `kovanica-protocol/docs/` | Manual curation (PR) | Per-RFC |
| `docs.kovanica.online` | CI publish (MkDocs) | On merge to main |
| Internal Notion/Linear | Manual | As needed |

---

## Key Documents (Reference)

| Document | Layer | Purpose |
|----------|-------|---------|
| `consensus-invariants.md` | 00 | GHOSTDAG safety/liveness proofs |
| `tokenomics-axioms.md` | 00 | RFC-006 hard rules (90.2M cap, maturity, fee burn) |
| `PoA-authority-set.md` | 10 | Authority set mechanics, rotation, dissolution |
| `testnet-reset-playbook.md` | 30 | Step-by-step testnet genesis procedure |
| `mainnet-launch-checklist.md` | 30 | P0/P1/P2 gates for mainnet |
| `RFC-POA-Migration.md` | 20 | Canonical PoA migration policy (§0 ratified) |
| `seed-policy.md` | 10 | DNS-only, grey-cloud, never orange-cloud |

---

## Contributing (Authorized Personnel Only)

1. **Clone privately** — `git clone git@github.com:KovanicaDAG/kovanica-brain-vault.git`
2. **Open in Obsidian** — `File → Open vault as workspace`
3. **Follow templates** — Use `templates/` for new documents
4. **Link bidirectionally** — `[[wiki-links]]` for knowledge graph
5. **Tag layers** — `#layer/00` through `#layer/60` in frontmatter
6. **PR to main** — Linear history, signed commits required

---

## Security

- **No secrets in vault** — API keys, seeds, private keys stored in 1Password/Bitwarden, referenced by label only
- **Encrypted at rest** — GitHub private repo + GPG-signed commits
- **Access logging** — All clones/pulls audited
- **Retention** — Immutable history; no force-push, no rebase on main

---

## Related Repositories

| Repo | Relationship |
|------|--------------|
| `kovanica-agent` | Consumes vault for RAG knowledge base |
| `kovanica-protocol` | Source of truth for shipped RFCs (mirrored from vault) |
| `docs.kovanica.online` | Public documentation site (published from vault subset) |

---

## License

**Proprietary / Internal Use Only** — Not open source. Contents governed by KovanicaDAG confidentiality policy.