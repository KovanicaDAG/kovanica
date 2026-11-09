# External Audit / Seed / DNS — Checklist

Updated to reflect canonical Hybrid PoW + GHOSTDAG consensus, KovanicaHash, KVNC tokenomics, and PRO-21 address requirements (Toni 2026-11-05).

## 1. Seed / DNS audit (gating before public launch)
- [x] Automated DNS probe script (`scripts/check-dns-seeds.sh`) implemented and verified
- [x] Verify 3+ **kovanica-native** DNS seeds (`testnet.kovanica.online`, `seed1/2/3.kovanica.online`)
- [x] Verify grey-cloud origin IP resolution (no Cloudflare HTTP proxying / CDN)
- [ ] Ensure TCP port 8000 reachability across all deployed VPS seeds (via `scripts/cutover-vps-systemd.sh`)
- [ ] Publish each seed's libp2p PeerId and confirm `/p2p/` in the multiaddr matches
- [x] Confirm `KOVANICA_BOOTNODES` defaults stay empty in code; seeds configured via env/TOML

## 2. External audit scope (mandatory before mainnet $T_0$)
- [ ] **Consensus (Hybrid PoW + GHOSTDAG, K=5, target 4s):**
  - [ ] Verify KovanicaHash PoW algorithm (BLAKE3 + 2 MiB memory mix scratchpad) determinism and memory-hardness
  - [ ] Verify Difficulty Adjustment Algorithm (DAA) stability, time warp resistance, and target bounds
  - [ ] Verify GHOSTDAG K=5 coloring, blue set selection, and topological DAG linear order
  - [ ] Verify edge anti-DoS ingress: PoW target verification before gossip relay (`validate_pow_target`)
  - [ ] Verify that consensus is strictly leaderless PoW + GHOSTDAG (NO stake in consensus path)
- [ ] **Execution (Wasmi deterministic WASM engine):**
  - [ ] Verify deterministic fuel metering per instruction and host call
  - [ ] Verify memory bounding (16 pages = 1 MiB limit) and trap cleanup on out-of-gas / stack exhaustion
  - [ ] Verify native & WASM contract host dispatch (HTLC, Vault, Multisig, Token)
- [ ] **Tokenomics & Emissions:**
  - [ ] Verify hard cap: 90 200 000 KVNC total supply
  - [ ] Verify block reward: 2.5 KVNC minted to committed block author (`block_reward(height)`)
  - [ ] Verify subsidy decay curve: ×3/4 every 8 100 000 committed heights; total subsidy budget 81 000 000 KVNC
  - [ ] Verify treasury vesting: 8 000 000 KVNC over 8 years (1M/year linear)
  - [ ] Verify genesis validator allocation: 1 000 000 KVNC (mainnet ceremony: 15–20 validators)
- [ ] **Cryptography & Address Format:**
  - [ ] Verify Ed25519 signatures (strict dalek v2.1: canonical S check, small-order rejection)
  - [ ] Verify PRO-21 addresses: `kvnc` + 57 base32 chars + `dag` (64 chars; 32B Ed25519 ‖ 29-bit BLAKE3 checksum)
  - [ ] Confirm legacy hex format is strictly rejected on mainnet (chain 1)
  - [ ] Verify keystore v2: Argon2id (64 MiB, 3 iter, 4 parallelism) + XChaCha20-Poly1305
- [ ] **P2P Networking & Node Security:**
  - [ ] Verify eclipse / Sybil resistance, peer discovery, gossip limits (port 8000 default)
  - [ ] Confirm no keys or seed material passed to or stored by the node daemon
  - [ ] Verify systemd sandboxing (`MemoryMax=3G`, `ProtectSystem=strict`, syscall filters)
- [ ] **Licensing & Copyleft:**
  - [ ] Verify GPL-3.0-or-later project relicensing (PRO-36 Option 2, written owner decision Toni 2026-11-03)
  - [ ] Verify cargo-deny CI enforcement for advisories, bans, licenses, and sources

## 3. Live node verification (repeat before release)
- [ ] Run `curl -s http://<kovanica-node>:8545` and check mining/sync status
- [ ] Confirm node memory usage stabilizes within the 2–4 GB VPS budget under soak
- [ ] Confirm `ALLOW_RESET=0`, `MINE=0` (on non-mining nodes), and `OPERATOR=0` on participant nodes
- [ ] Verify persistence across node daemon restarts without state corruption

Status: Scoping complete (`docs/AUDIT-SCOPE.md`); ready for external auditor engagement.
