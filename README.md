# kvnc ↔ OpenCode Integration Pack

Pripremljeni paket da **OpenCode** i **Grok** rade zajedno na **kvnc**-u bez miješanja poslova.

## Sadržaj

```
kvnc-opencode-setup/
├── AGENTS.md                          ← stavi u root kvnc repo-a
├── opencode.json                      ← project config
├── .opencode/agents/kvnc.md           ← specijalni agent
├── prompts/kvnc-system.txt            ← alternativni prompt
├── docs/SETUP-OPENCODE-KVNC.md        ← detaljne upute
└── README.md                          ← ovaj file
```

## Brza instalacija

1. Kopiraj cijeli sadržaj u **root** svog lokalnog kvnc klona.
2. Pokreni `opencode` unutar tog root-a.
3. Prebaci se na agenta **kvnc** (Tab ili `@kvnc`).

Detaljne upute: `docs/SETUP-OPENCODE-KVNC.md`

## Podjela uloga

| Tko          | Uloga                                      |
|--------------|--------------------------------------------|
| **Grok**     | Arhitekt, planer, reviewer, tokenomics, dizajn |
| **OpenCode (kvnc agent)** | Lokalna implementacija, file edit, testovi, git |

## Status pretplate

- Grok pretplata: još nije aktivna (prema tvojoj izjavi)
- OpenCode agent: potpuno spreman za korištenje odmah

Kad aktiviraš Grok pretplatu, workflow postaje:
**Grok plan → OpenCode implementacija → Grok review**.

---

*Generirano za kvnc projekt – ne miješati s kovanica-protocol.*

---
## Aktualni status (2026-10-08)
- `use_mysticghost` = `true` (default) — eksperimentalni flag.
- `init_genesis` proširen: `founder_premine.hex` + `validators.json` load.
- `kvnc-staking` batch verify (`verify_batch`) i `Address` sort popravljeni.
- `kvnc-storage` state root (sorted KV Merkle) + snapshot + `STAKING_STATE` / `CONTRACT_STORAGE` serijalizacija popravljena.
- `TASKLIST.md` ažuriran za završene faze (1.1, 2.2, 4.3/4.4, 5.1/5.2, 8.2, 9.1, 14, 16).
- Preostali `cargo test -p kvnc-node --test multi_node_integration`: blokiran preostalim `kvnc-execution` (`unbonding_queue`) i `kvnc-network` greškama (ne u mom edit opsegu).
- Tokenomics zaključan: `90.2M`, `200k premine`, `8M treasury`, `10 KVNC`, `2.05M era`, `×¾ decay`.
