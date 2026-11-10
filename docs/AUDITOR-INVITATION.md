# Poziv za eksternu reviziju — kovanica

Status: **Interna verifikacija i specifikacija završena** (`docs/AUDIT-SCOPE.md`, `docs/AUDIT-CHECKLIST.md`); eksterni audit **nije izvršen** — obavezan prije mainnet $T_0$ lansiranja.

Identitet projekta: **kovanica** (nezavisni Rust Layer-1, od-scratch; NE kovanica-protocol, NE stariji Kovanica repo).
Repozitorij: `https://github.com/KovanicaDAG/kovanica` (`main` branch).
Datum poziva: 2026-11-09.

---

## 1. Arhitektonske osnove (zaključano Toni 2026-11-05)

- **Konsenzus:** Hibridni PoW + GHOSTDAG (`TARGET_BLOCK_MS = 4000`, `GHOSTDAG_K = 5`). **BEZ uloga (stake) u konsenzusnom putu.**
- **PoW algoritam:** **KovanicaHash** (BLAKE3 + 2 MiB memorijski mix scratchpad; memorijski zahtjevan, otporan na ASIC ubrzanja, deterministička verifikacija).
- **Izvršavanje:** Wasmi deterministički WebAssembly engine s mjerenjem goriva (fuel/gas metering) i memorijskim limitom od 16 stranica (1 MiB).
- **Tokenomika:** Nativni coin **KVNC** (9 decimala, tvrdi limit **90 200 000 KVNC**, početna nagrada bloka 2,5 KVNC s decay-om $\times \frac{3}{4}$ svakih 8 100 000 visina).
- **Adrese:** PRO-21 kanonski format: `kvnc` + 57 base32 znakova (bech32 abeceda) + `dag` = 64 znaka. Hex adrese strogo zabranjene na mainnetu (`chain_id = 1`).

---

## 2. Što eksterni auditor mora provjeriti (prioriteti)

### P0 — Konsenzus, PoW i otpornost na DoS (blokira Mainnet $T_0$)
1. **KovanicaHash PoW algoritam**: `crates/kovanica-crypto/src/pow.rs` — determinizam na svim arhitekturama (x86_64, aarch64), otpornost na kolizije, 2 MiB memory scratchpad profil.
2. **GHOSTDAG K=5 committer**: `crates/kovanica-consensus/` i `crates/kovanica-dag/` — deterministički linearni poredak blokova, blue set kalkulacija, mergeset evaluacija.
3. **DAA (Difficulty Adjustment Algorithm)**: stabilnost prilagodbe težine za ciljano vrijeme bloka od 4 s, otpornost na manipulaciju vremenskim oznakama (timestamp warping).
4. **Edge Anti-DoS ulazna validacija**: `validate_pow_target` — provjera ciljne težine (target) prije propagacije kroz gossip i prije zapisa u bazu (DAG store).
5. **Wasmi determinističko izvršavanje**: `crates/kovanica-execution/` + `crates/kovanica-runtime/` — determinističko oduzimanje goriva, limit od 16 stranica memorije (1 MiB), sigurno rukovanje trap-ovima i zaštita od reentrancy napada.

### P1 — Tokenomika, kriptografija i adrese (blokira Mainnet $T_0$)
6. **Tokenomics invarijante**: tvrdi cap 90 200 000 KVNC; 2,5 KVNC nagrada po potvrđenom bloku; raspodjela proračuna (82M rudarenje, 8M riznica, 0 genesis validatori, 0,2M premine). Zabrana `funded_accounts.toml` na mainnetu (`chain_id = 1`).
7. **Ed25519 kripto & PRO-21 adrese**: `crates/kovanica-crypto/` — striktna verifikacija potpisa (`dalek v2.1`), provjera kanonskog $S$, odbijanje slabih ključeva. Kanonski PRO-21 64-znakovni format (`kvnc...dag`) s 29-bitnim BLAKE3 kontrolnim zbrojem.
8. **Keystore v2**: `crates/kovanica-keystore/` — Argon2id enkripcija, radna memorija se čisti (`zeroize`), sigurne dozvole datoteka (`0600`).

### P2 — P2P mreža i otpornost na napade (blokira mrežnu stabilnost)
9. **P2P transport & Gossipsub**: `crates/kovanica-network/` — kanonski port **8000**, TCP + Noise + Yamux; ocjenjivanje peer-ova (peer scoring), otpornost na Sybil i eclipse napade.
10. **DNS seed redundantnost**: grey-cloud razrješenje (bez Cloudflare proxyja) za `testnet.kovanica.online` i `seed1/2/3.kovanica.online`.
11. **Sistemska izolacija (Systemd sandboxing)**: `ops/systemd/kovanica-node.service` (`MemoryMax=3G`, `ProtectSystem=strict`, sistemski filtri) za siguran rad unutar 2–4 GB RAM budžeta.

---

## 3. Što auditor treba dostaviti (izlaz)

- Pismeni izvještaj s klasifikacijom nalaza: `CRITICAL` / `HIGH` / `MEDIUM` / `LOW` / `INFORMATIONAL`.
- Za svaki nalaz: točna lokacija (`file:line`), opis prijetnje, predloženi popravak, verifikacija.
- Potvrda da tvrdi cap tokenomike (90.2M KVNC) i raspodjela nagrada ne mogu biti narušeni.
- Potvrda da je Wasmi izvršavanje determinističko i memorijski izolirano.
- Potvrda da je KovanicaHash determinističan i memorijski zahtjevan.
- Preporuka za spremnost za mainnet (`Mainnet Readiness`: DA / NE / UVJETNO).

---

## 4. Kontakt i koordinacija

- Repozitorij: `https://github.com/KovanicaDAG/kovanica`
- Dokumentacija opsega revizije: [`docs/AUDIT-SCOPE.md`](AUDIT-SCOPE.md)
- Kontrolna lista: [`docs/AUDIT-CHECKLIST.md`](AUDIT-CHECKLIST.md)
- Eksterni auditor mora biti neovisan i potpisati vlastiti izvještaj.
