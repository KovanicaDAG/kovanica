# Kovanica Protocol — Plan šest nadogradnji (2026-09-05)

> Izvor: razgovor s Grok-on; usaglašeno s [[UPGRADE-PHASES]] (mainnet on-sleep-off) i [[ROADMAP]]. Odluke: redosled **3 → 6 → 5**, DeFi bez VM-a, tokeni prvo.
> **Links:** [[UPGRADE-PHASES]] · [[ROADMAP]] · [[DECISIONS]] · [[FACTS]]

## A. Master plan (svih 6 skupa)

| # | Tačka | Status | Osnova u kodu | Pušta na testnet |
|---|-------|--------|---------------|------------------|
| 1 | Stabilizacija & hardening (pre-)mainneta | soak ACTIVE | OPERATIONS.md, Prometheus, cargo-audit, p2p hardening | kontinuirano, mainnet ostaje dormant |
| 2 | Performanse & node softver | ogranak `net/headers-first-topo-sync` u toku | net.rs, LedgerStore, reachability | kontinuirano |
| 3 | Scripting / token standardi | **START** | tx.rs, utxo.rs, ledger.rs, šablon multisig aktivacije | aktivacioni gate |
| 4 | Interoperabilnost (bridge) | dokument samo | RFC-001 multisig, SPV header chain | — (odluka posle 3A) |
| 5 | Pametni ugovori / DeFi | posle 3+6 | script v2, stake.rs | aktivacioni gate |
| 6 | Privatnost | posle 3A (isti bump) | TxOutput bump, keys.rs | aktivacioni gate |

**Kritični put zavisnosti:**
```
3A tokeni ─┬─► 3B script v2 ─► 5.1 HTLC ─► 5.2 vault ─► 5.3 token staking
           └─► 6.1 stealth (isti format bump) ─► 6.2 CoinJoin
1.x soak / 2.x bench: teku paralelno kroz sve
4.x: dokument nakon 3A; 5.4/5.5/6.3: dokumenti (mogu rano)
```

**Cross-cutting (svaka konsenzus slice):** RFC u `docs/`, aktivacioni gate po blue score-u (šablon `MULTISIG_ACTIVATION_SCORE`), snapshot bump (>v5) + checkpoint bump (>v3) + `BlockRecord` flag byte, FFI + regenerisani bindings + drift-guard CI, explorer/web, SPV filteri, deterministic+adversarial testovi, `AGENTS.md` u istom change-u, `unsafe` zabranjeno, tie-break po `BlockId`.

---

## Tačka 1 — Stabilizacija & hardening (pre-)mainneta

> Konstanta iz UPGRADE-PHASES: **mainnet NE pokrećemo** — switch ostaje on-sleep-off. Ovo planira *sve što mainnet čini mogućim*, bez lansiranja.

- **1.1 Soak nastavak** (trenutno ACTIVE): VPS Prometheus scrape (orphan rate, propagation, fork/reorg, disk) — TODO.md „Next session" #5; tuning review posle ~2 nedelje (k, finality, pruning, difficulty) — #6.
- **1.2 Ops hardening:** backup/restore drill (OPERATIONS.md), 4. off-box seed + tunnel alerting, web deploy automation, release pinning (UPGRADE-PHASES G1–G4, A8).
- **1.3 Security:** cargo-audit CI (postoji), proširiti fuzz na net granice, pravila revizija peer scoring/ban (postoji), kripto revizija ed25519 + VRF.
- **1.4 Dormant mainnet profil (A1):** održava se; genesis/tokenomika/inflacija/governance = poseban dokument *za odluku*, ne za ovu fazu.
- **Exit kriterijum:** soak stabilan ≥ definisani period + odluka o mainnetu (van ovog plana).

## Tačka 2 — Performanse & node softver

- **2.1 Finalizovati headers-first topo sync** (ogranak `net/headers-first-topo-sync`, fix `d9502a9`): PR + regresioni test za divergenciju (MissingParent livelock).
- **2.2 Criterion benchmark suite:** `benches/` za insert/ghostdag/mergeset/linearize, mempool, net en/dek; baseline uporediti sa seed3 metrikama.
- **2.3 Smanjenje memorije/rado:** ledger per-block state O(n²) trade-off → pruning na osnovu UTXO undo loga (B3 je poslan); API pagination (C2 deo).
- **2.4 Hot path profil:** reachability reindex metrike (već postoje), blake3, snapshot I/O.
- **2.5 Load-testing:** multi-node scenariji; tuning metrika/alerta iz soak podataka.

## Tačka 3 — Jednostavniji scripting / token standardi

- **3.1 (Slice 4A) Native tokeni · RFC-002-NativeTokens.md** — Cardano multi-asset stil: `AssetId = BLAKE3(policy_id ‖ name)`; `TxOutput.assets: Vec<Asset>`; konzervacija po asset-u; coinbase samo KVNC; mint/burn autoritet preko tag-konvencije (šablon `stake.rs`). Bump: encoding, sighash, utxo, ledger, snapshot/checkpoint, BlockRecord, FFI, explorer, SPV. Aktivacioni gate. Testovi: `tests/native_tokens.rs` (mining, konzervacija, double-spend preko paralelnih blokova, prag, roundtrip).
- **3.2 (Slice 4B) Script v2 · RFC-003-ScriptV2.md** — deterministički, ne-Turing-potpun, Address **v0x02** = `BLAKE3(script)`: `ED25519_VERIFY`, `CHECKLOCKTIMEVERIFY` (BIP-65), `CHECKSEQUENCEVERIFY` (BIP-112), `HASH_BLAKE3`+`EQUAL`, AND/OR, threshold, budžet izvršavanja. Testovi: `tests/script_v2.rs`.

## Tačka 4 — Interoperabilnost (bridge-ovi)

- **4.1 RFC bridge dizajn (bez implementacije)** — komparacija tri modela:
  - **Federation/m-of-n escrow** — najprirodnije: već imamo M-of-N multisig (RFC-001)
  - **Light-client relay** — koristi postojeći SPV header chain (spv.rs)
  - **Optimistic** (fraud window) — najteži, za osigurane sume
- **4.2 Blokatori:** stabilan asset (3.1) + ekosistem/tržište; rizici: reorg, finality, peg-validator uzurpacija.
- **4.3 Odluka o ciljnom lancu** (BTC/ETH vs drugi DAG) — posle 3A i soak podataka; otvorena stavka ovog plana.

## Tačka 5 — Pametni ugovori i DeFi (bez VM-a)

- **5.1 HTLC / atomic swap** (na 4B) — hash-lock + timelock; biblioteka `atomic_swap.rs` + RPC/FFI; `tests/htlc.rs` (redeem/refund, tajming, adversary).
- **5.2 Time-lock vault / escrow** (CSV/CLTV); `tests/vault.rs`.
- **5.3 Token staking / sortition** (na 3.1 + stake.rs): `StakeState` prima asset-e (Algorand/Praos stil); `tests/token_staking.rs`.
- **5.4 DEX dizajn dok** — off-chain matching + on-chain atomska nagodba (bez order book-a na lancu).
- **5.5 Research gate: VM odluka** — Wasm vs script-v3, pisano obrazloženje, **explicitno ne EVM** (ne mapira se na DAG/k-cluster). Nema implementacije u ovom opsegu.

## Tačka 6 — Privatnost i napredne značajke

- **6.1 Stealth adrese** — Address **v0x03** (scan_pk ‖ spend_pk, 64B); izlaz nosi `R=r·G` + view_tag (CryptoNote stil, view-tag filter idealan za SPV); **u istom format bump-u kao 3.1**. Testovi: `tests/stealth.rs` (skeniranje, nonce reuse adversary, lažni pozitivnici).
- **6.2 CoinJoin batching** — van konsenzusa, node-level (uzorak `prepare_transfer`); `tests/coinjoin.rs`.
- **6.3 CT (Bulletproofs) — research izveštaj** `docs/research/confidential-transactions.md` (Pedersen + range proofs, determinizam, veličina), odluka po izveštaju.
- **6.4 P2P privatnost (Tor/i2p socks na relay/DHT)** — opciono, networking, van konsenzusa.

---

## B. Sekvencija izvođenja (Work Packages)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ W0: FOUNDATION  (paralelno sa svime, NE blokira WP1)                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ W0.1  Soak nastavak (ACTIVE) ──────────────────────→ Gate: tuning review    │
│ W0.2  Ops hardening (backup, 4. seed, deploy auto, pin)                    │
│ W0.3  Security (cargo-audit, fuzz net, crypto review)                      │
│ W0.4  Headers-first topo sync finalize (ogranak net/headers-first-topo-sync)│
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ WP1: CORE CONSENSUS (SEVENCIALNO — svaki gate = testnet deploy + aktivacija)│
├─────────────────────────────────────────────────────────────────────────────┤
│ WP1.1 3A Native Tokens (RFC-002)  ──────────────┐                            │
│   Format bump: snapshot>v5, checkpoint>v3       │  ISTI BUMP                │
│   Tests: native_tokens.rs                       │  (single TxOutput change) │
│                                                 ▼                            │
│ WP1.2 6A Stealth Addresses ──────────────────────┤                            │
│   Address v0x03, view-tag filter (SPV-friendly)  │                            │
│   Tests: stealth.rs                            ◄─┘                            │
│                                                 │                            │
│ WP1.3 3B Script v2 (RFC-003) ───────────────────┤                            │
│   v0x02, opcodes: CLTV/CSV/hash-lock/threshold  │                            │
│   Tests: script_v2.rs                          │                            │
└─────────────────────────────────────────────────┼────────────────────────────┘
                                    │
         Gate 2: Testnet soak with 3A+6A+3B active
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ WP2: DeFi PRIMITIVES (paralelno sa WP3)                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│ WP2.1 5A HTLC / Atomic Swap  → WP2.2 5B Vault  → WP2.3 5C Token Staking    │
│   atomic_swap.rs             CSV/CLTV logic       stake.rs extend          │
│   htlc.rs tests              vault.rs tests       token_staking.rs tests   │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
         Gate 3: DeFi primitives functional on testnet
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ WP3: DOCS + PERFORMANCE (može početi ODMAH posle WP1.1)                     │
├─────────────────────────────────────────────────────────────────────────────┤
│ 5D  DEX Design Doc                 6C  CT Research (Bulletproofs)          │
│ 5E  VM Research Gate (Wasm vs)     4.1 Bridge RFC (federation/SPV/optimist)│
│ 2.2 Criterion Benchmarks           2.3 Ledger Pruning (UTXO undo log B3)   │
│ 2.4 Hot Path Profiling             2.5 Load Testing multi-node             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ WP4: POST-SOAK / OPCIONO                                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│ 6B  CoinJoin batching (node, non-consensus)                                │
│ 6D  Tor/i2p P2P privacy                                                    │
│ 4.x Bridge implementation decision (nakon soak + 3A data)                 │
│     Mainnet launch decision (tokenomika/governance = poseban dokument)     │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Critical Path:** `3A → 6A+3B → 5A → 5B → 5C` (≈ 5-6 konsenzus slice-ova)

**Svi konzenzus slice-ovi:** RFC → implementacija → `tests/*.rs` (deterministički+adversarial) → `cargo fmt && clippy --all-targets && test` → feature branch → draft PR → testnet sa aktivacionim gate-om (blue score prag).
