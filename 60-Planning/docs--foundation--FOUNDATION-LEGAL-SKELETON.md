---
title: "Kovanica Protocol — Foundation & Legal Skeleton"
category: 60-Planning
source: docs/foundation/FOUNDATION-LEGAL-SKELETON.md
synced: 2026-09-26
---
# Kovanica Protocol — Foundation & Legal Skeleton

**Structure · Documents · Decentralisation Path**  
**Date:** September 2026 · Pre-Mainnet  
**Status:** DRAFT — awaiting counsel engagement (NOT started)

> This document is **not legal advice**. It is a practical outline for counsel to turn into binding documents.

---

## 1. Core Design Principle

Kovanica is intended to become progressively decentralised. In the early years, however, concentrated and accountable control under a legal Foundation is required so that treasury, domains, trademarks, security incidents, and protocol upgrades can be handled quickly and transparently.

The Founder remains the primary steward, but the legal and on-chain structures are designed so that control can be diluted over time without breaking continuity or trust.

---

## 2. Recommended Legal Structure

### Entity

- **Name:** Kovanica Foundation (or local equivalent)
- **Preferred jurisdiction:** Croatia / EU (founder-friendly, EU regulatory clarity)
- **Form:** Non-profit foundation or association with clear purpose clause tied to the protocol
- **Alternative if EU is slow:** Swiss Foundation or Cayman non-profit (higher cost, more crypto precedent)

### Governance of the Entity

- Initial Board: Founder only (or Founder + 1–2 trusted advisors)
- Board expansion planned after mainnet stability (target: 3–5 members by end of year 1)
- Reserved matters requiring Board super-majority: treasury spends above threshold, IP licensing, major protocol parameter changes

### Assets Held by the Foundation

- All primary domains (`kovanica.online` and subdomains)
- Trademarks "Kovanica", "KVNC", logo and brand assets
- GitHub organisation(s) and package namespaces
- On-chain treasury wallets and vesting vaults
- Any future IP, patents, or licensing agreements

---

## 3. On-Chain Treasury & Founder Allocation

- RFC-006 already defines: **0.2M KVNC** founder premine + **10M KVNC** treasury in 10 × 1M RFC-005 vaults
- All treasury movements public and preferably multi-sig
- Founder personal wallets strictly separated from Foundation wallets
- **No "private seed mining then market selling"** — emissions follow the published curve only
- Vesting schedules published and enforced by vault scripts

---

## 4. Documents to Produce (Checklist)

- [ ] Foundation Articles / Bylaws / Statutes
- [ ] IP Assignment Agreement (Founder → Foundation) for code, brand, domains
- [ ] Treasury Policy (spending rules, multi-sig thresholds, reporting cadence)
- [ ] Conflict of Interest & Related-Party Transaction Policy
- [ ] Public "Who is behind Kovanica" page (Founder bio + social links + Foundation status)
- [ ] Trademark applications (at least EU / Croatia)
- [ ] Privacy Policy + Terms for wallet, explorer, playground
- [ ] Bug-bounty / Responsible Disclosure Policy (even if small rewards at first)

---

## 5. Decentralisation Timeline (Indicative)

### Year 0 (now → mainnet + 6 months)

- Founder + Foundation control fully documented and public
- All critical keys and domains under Foundation ownership
- Transparent tokenomics and vesting already live

### Year 1

- Expand Board with independent members
- Launch public Developer Grant Program funded from treasury
- Node-runner and early contributor recognition programs

### Year 2+

- Soft governance processes → formal on-chain proposal mechanism
- Progressive reduction of Founder unilateral power
- Never claim "fully decentralised" while a single party still controls domains or upgrade keys

---

## 6. Immediate Legal Actions

1. Engage Croatian / EU counsel experienced in foundations + crypto
2. Decide jurisdiction and entity form within 30–45 days
3. Execute IP assignment and domain transfer to the new entity
4. Publish the public Founder + Foundation page on kovanica.online
5. Open multi-sig (or at minimum dual-control) for treasury as soon as second trusted party exists

---

*Update this file as decisions are made. Monorepo home: `/docs/foundation/`.*