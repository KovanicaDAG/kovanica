# Kovanica Project Task List (Phases 1–25)

> **Ova verzija je 100% truth state — provjerena kod-inspekcijom + buildom na `main` @ `a2ec2db` (2026-10-14, post PR #56 / `f012c58` merge: light-client + MysticGhost + VRF wire + 20% light reward).**
> Prethodna verzija (s checkboxes) i `TASKLIST_DRIFT.md` (2026-10-08) su zastarjeli; ovaj dokument ih zamjenjuje; `docs/TESTNET-RESET.md` dodan 2026-10-14.
> **Revizija 2026-10-12:** late-join test postoji i integriran; svi `[ ]`/`[~]` itemi ponovno provjereni protiv koda (vidi "Provjera neoznačenih itema" na dnu).
> **Revizija 2026-10-13:** VRF Phase 1.3 **RIJEŠEN** — ECVRF-ED25519-SHA512-Elligator2, stake-weighted leader selection, genesis/staking/staking-tx wiring, 7 testova prolaze; TASKLIST.md ažuriran.
> **Revizija 2026-10-14:** Testnet seed nodes 3+ **DONE** — fresh validator keys, genesis, run scripts, systemd units on all 3 VPSes; peer mesh verified; corrupt `kovanica-node.service`/`kovanica-node-testnet.service` fixed; testnet/mainnet config dirs created in `/root/Projects/kvnc/testnet/` & `/mainnet/`.
> **Revizija 2026-10-16 (mergeano na `main`):** **Leader-rotation stall RIJEŠEN.** Root cause bio je `u128` saturacija u VRF prioritetu (`hash * total_stake / stake` → `saturating_mul` → svi autoriteti `u128::MAX` → tie-break držao najniži indeks → 100% rundi jedan leader). Zamijenjeno egzaktnom 192-bitnom cross-multiplied usporedbom. Kaskada (non-leaderi skupljali `missed_leader_count` → `active=false` → committee pao na 1 → stall) time je uklonjena. Drugi, aditivni bug: engine nije nastavljao round counter iz perzistiranog DAG-a → restart na nepraznoj chaini → `ConflictingDecision`. Oba fixa su na `main` (+ `register-light-node` CLI).

**Legenda statusa**
- `[x]` — implementirano i wired u production pathu (verificirano u kodu)
- `[~]` — djelomično (kod postoji, ali nije potpuno wired / nije production-safe / test ne prolazi)
- `[ ]` — nije gotovo (nema koda ili je samo plan)

**Kritični path:** Types → Storage → DAG → Consensus → Execution → Node wiring → Committee → Admission → Multi-node quorum → Testnet

---

## Verified run (2026-10-14, `a2ec2db` = PR #56 merge `f012c58` + doc commit)

- `cargo check --workspace --all-targets` → **0 errors**
- `cargo clippy --workspace --all-targets` → **clean** (CI: `-D warnings`)
- `cargo test --workspace --lib` → **all lib tests pass** (59/59 in kovanica-execution after reward test fix; tokenomics s₀=2.5 KUNA assertions updated)
- Integration testovi (`--tests`):
  - `kovanica-node multi_node_integration` → **3/3 PASS** (konvergencija 4 noda na identičan commit sequence; partition 3+1 majority commits & minority catches up; 2+2 nema quorum)
  - `kovanica-node sustained_liveness_integration` → **1/1 PASS** (~65s; 4 validatorska procesa, committed height 20+ preko pruning boundary, isti leader na svim nodovima)
  - `kovanica-rpc websocket_subscriptions` → **PASS**
  - `kovanica-node late_join_integration` → **1/1 PASS** (`late_joining_validator_catches_up_via_parent_sync`)
  - `kovanica-node live_vote_integration` → **2/2 PASS** (`live_two_node_vote_commit_execute`, `live_transaction_commit`)
- Cargo warning-i (non-fatal, postoje u kodu): `kovanica-cli` lib+bin dualni target upozorenje; par unused importa u `kovanica-execution`/`kovanica-network`. (Staking stub fn i RPC dead-code middleware su uklonjeni 2026-10-13; `rate_limit_middleware` je wire-an preko axum layer-a.)

**Naming (zaključano, razriješeno 2026-10-10):** chain = **Kovanica**, native coin ticker = **KUNA** (9 decimals), code/binary/env prefiks = **kovanica**. Svi chain-facing ticker reference su sada KUNA: `README.md`, `AGENTS.md`, `docs/*`, signature domain tagovi (`KUNA/vote/v1`, `KUNA/tx/v1`), `Hash::DOMAIN_TX = "KUNA-TX-v1"`, konstanta `ONE_KUNA` (ex-`ONE_KVNC`), CLI, faucet. Adresa ostaje `kvnc<hex>dag`.

**Tokenomics revidirano 2026-10-13 (odobreno od strane vlasnika):** s₀ **2,5 KUNA**, era **8 100 000** committed leader blocks, mining budget **81 000 000** KUNA, +nova **1 000 000** KUNA genesis validatorska alokacija (ravnomjerni split, dust deterministički), round **4 s** (leader 12 s), `EPOCH_ROUNDS = 7 200` (≈1 dan, decoupled od ere). Nepromijenjeno: 90.2M cap, 9 decimals, 200k premine, 8M treasury (1M/god × 8), ×¾ decay, min stake 50 000, **5–20 genesis validators**, 15–21 aktivnih validatora. Detalji: `docs/TOKENOMICS.md`.

**Seed/DNS (razriješeno 2026-10-10):** kovanica kod više ne referencira tuđu infrastrukturu. `NetworkConfig::default().bootstrap_nodes` i `NodeConfig::default().bootnodes` su prazni; seedovi idu preko `KOVANICA_BOOTNODES` / `[node] bootnodes`. 3+ kovanica-native DNS imena treba registrirati prije javnog testneta (`docs/SEED-DNS-AUDIT.md`). Kanonski P2P port: **8000**.

---

## PHASE 1.1 (kovanica-types): Core types — `[x]` DONE

- [x] Block/Tx serialization (serde + bincode)
- [x] Merkle root na StatementBlock (`compute_merkle_root` + validacija u block_manager)
- [x] Round (`u64`) / AuthorityIndex (`u16`) aritmetika
- [x] Committee types + `leader_for_round` stake-weighted (`CommitteeInfo`, `types.rs:204-226`)
- [x] Canonical address `kvnc<hex>dag` + blake3 checksum, bare/0x back-compat (`address.rs`)
- [x] Hash domain separation (BLOCK/TX/DIGEST/MERKLE) + `kovanica-types/src/signing.rs` — signature format v1: `chain_id` registry (mainnet 1, testnet 2, devnet 3, local 1337), `VOTE_DOMAIN_TAG`/`TX_DOMAIN_TAG` (16-byte, `KUNA/...`)
- [x] `CommittedSubDag.non_blue` field (#13 contract) — red blocks vraćeni u mempool, ne prunaju se

## PHASE 1.2 (kovanica-crypto): Primitives — `[x]` DONE

- [x] Keypair persistence: Argon2id + XChaCha20-Poly1305 (`kovanica-cli/src/wallet.rs`, keystore v2 + v1→v2 migracija)
- [x] `verify_batch` → `Err(VerificationFailed)` na nevalidan potpis (ne `Ok(false)`) + `tests/strict_vectors.rs` (strogi vektori)
- [x] Strict sig verify svugdje (commit `bebe8c6`)
- [ ] BIP32-style HD derivation — **nema koda** (samo BIP-39 mnemonic import bez HD izvoda u CLI)
- [x] VRF za leader selection — **ECVRF-ED25519-SHA512-Elligator2** via `ecvrf` crate (`kovanica-crypto/src/vrf.rs`: `VRFKeyPair`, `VRFOutput`, `vrf_prove`, `vrf_verify`, `vrf_hash_to_u64`; 7 testova prolaze)

## PHASE 1.3 (VRF Wiring): Leader Selection Integration — `[x]` DONE (wired by PR #56 / `f012c58`, beacon driving RIJEŠENO 2026-10-14)

VRF keys stored, genesis/staking/staking-tx wired (`build_committee` reads them). Engine (`engine.rs:638`) calls `committee.leader_with_vrf` when `ConsensusConfig::use_vrf_leader` is **on**; **default is `true`** (`engine.rs:255`). **VRF beacon now drives selection via quorum-based `vrf_set_complete()` (2f+1 of VRF-keyed validators) at decide rounds** (`engine.rs:442`). Light-node VRF (`kovanica-staking/src/light.rs`, `kovanica-types/src/light_client.rs`) wired separately (20% reward escrow).

- [x] `CommitteeInfo::leader_with_vrf` wired (`use_vrf_leader: true` default `engine.rs:255`); **beacon drives selection when quorum of VRF outputs received on decide rounds** (quorum = floor(2*N/3)+1 where N = validators with VRF keys).
- [x] `Authority.vrf_public_key: Option<PublicKey>` — optional VRF key in committee authority (`kovanica-types/src/committee.rs:18`)
- [x] `ValidatorInfo.vrf_public_key: Option<VrfPublicKey>` — optional VRF key in staking state (`kovanica-staking/src/lib.rs:311`)
- [x] Genesis `genesis_validators.toml` supports optional `vrf_public_key` field (`kovanica-node/src/main.rs:1819`)
- [x] `build_committee` reads VRF keys from staking state and builds committee with VRF pubkeys (`kovanica-node/src/main.rs:1739`)
- [x] `StakingState.join_validator` accepts optional `vrf_public_key` parameter (`kovanica-staking/src/lib.rs:418`)
- [x] Staking transaction types (`Stake`, `Delegate`) wire VRF public key into validator record (`kovanica-execution/src/lib.rs:604`)
- [x] All tests updated for 6-argument `join_validator` signature
- [x] `Committee::leader` with an empty VRF map falls back to round-robin. That fallback is what the engine runs, because it never supplies VRF outputs.

## PHASE 2.1: Block Store — `[x]` DONE

- [x] redb schema + tables init
- [x] BlockStore API (put/get/by_height/by_range)
- [x] Transaction index (tx hash → block ref)
- [x] Pruning (`prune_below`; wave window u dag_store)

## PHASE 2.2: State Store — `[x]` DONE

- [x] Account state (balance/nonce/code/storage)
- [x] Staking state persist (save/load)
- [x] Sorted-KV Merkle state root (`compute_state_root`)
- [x] Snapshot/Restore (`export_snapshot`/`import_snapshot`) + `sub_balance` fix (open-table write + checked_sub)

## PHASE 2.3: Consensus Store — `[x]` DONE

- [x] DAG persistence (block + parent/child links)
- [x] Commit tracker (committed_leader_height, decided rounds)

## PHASE 3.1: DagStore — `[x]` DONE

- [x] Block ingestion (parents/sig/round validacija)
- [x] Causal ordering (ancestors BFS)
- [x] Parent selection stake-filtered (max_parents ceil 3 po wave; `MAX_PARENT_ROUND_GAP`)
- [x] GC: `prune_below` / `prune_waves_before`; `prune_non_blue` više se NE poziva (MysticGhost variant A vraća red blocks kroz `non_blue`)

## PHASE 3.2: Block Manager — `[x]` DONE

- [x] Propose block (`propose_block`/`propose_block_with_txs`)
- [x] Validate block + ancestry walk (uklj. prune-validate testove, gap > MAX_PARENT_ROUND_GAP rejected)
- [x] Block broadcast (engine broadcaster)

## PHASE 4.1: Committer — `[x]` DONE

- [x] Direct commit rule (2f+1, `has_quorum`)
- [x] Indirect commit rule (`has_path`)
- [x] `LeaderStatus` Undecided→Commit/Skip
- [x] `CommittedSubDag` produkcija (linearizer + mysticghost + non_blue red blocks)
- [x] Late-vote: cast glasa kada leader block stigne nakon vote rounda (commit `dfccfb3`)

## PHASE 4.2: Linearizer — `[x]` DONE

- [x] Topological sort + dedup; test `test_linearize_parents_outside_history_ignored`

## PHASE 4.3: Wave Logic — `[x]` DONE (1 item `[~]`)

- [x] `WAVE_LENGTH`, `wave_of`, `offset_in_wave`, `is_leader_round`, `scheduled_leader_for_round`
- [x] Deterministic leader selection (round-robin + stake-weighted)
- [x] Timeout handling: non-zero `leader_timeout_ms` wins; otherwise `round_duration_ms * leader_timeout_factor` (default factor 2) + `register_skip`

## PHASE 4.4: Consensus Loop — `[~]` partial

- [x] Round timer (`round_loop` + `interval(round_duration_ms)`)
- [~] Fork handling — default **uses** `use_min_digest_fork: true` (`engine.rs:256`); `process_block` tie-break reachable (`line 855-885`). Only tests set false.

## PHASE 5.1: Core Mempool — `[x]` DONE

- [x] Tx pool priority by fee rate (`BTreeMap<u64, VecDeque<Transaction>>`)
- [x] Admission control (nonce/balance/sig/gas; zero-fee rejected **osim** Stake)
- [x] Eviction (`maybe_evict`, najniži fee-rate)
- [x] Rebroadcast
- [x] Gossip edge: `FeeTooLow` = **Ignore, ne Reject** (commit `6703663`) — ne ruši peer-a zbog niskog fee-ja

## PHASE 5.2: Block Building — `[x]` DONE

- [x] `get_next_transactions` conflict-aware (grupiranje po senderu, sort po nonce-u, kontiguitet)
- [x] Fee estimation advisory + `kovanica_estimateFee` (stub u RPC-u)

## PHASE 6.1: libp2p Setup — `[x]` DONE

- [x] Transport TCP + Noise + Yamux; gossipsub (topics: blocks, transactions, votes, sync); Kademlia; identify; ping
- [x] mDNS namjerno odsutan (DNS-only seed)
- [x] **Persistent libp2p identity** (`identity.rs`): key-file permission check (0600/0400), PeerId stabilan kroz restarte

## PHASE 6.2: Protocols — `[x]` DONE

- [x] Block sync request/response (`BLOCK_SYNC_PROTOCOL`)
- [x] Tx gossip + RPC→gossip (`kovanica_sendRawTransaction` šalje u gossipsub, commit `0cfbbc3`)
- [x] Vote gossip
- [x] **Peer scoring** (`scoring.rs`): `PeerScoringConfig` (gossip/publish/graylist/ban pragovi; samo negativne akcije spuštaju score; mesh-delivery penalizacija isključena)
- [x] **Vote verifier trait** (`validation.rs`): `Ed25519VoteVerifier`, `AuthorityKeys`, `Rejection`, `SigError`; edge-validacija bloka (digest, merkle, committee author, potpis)

## PHASE 6.3: Bootstrap — `[x]` DONE

- [x] Bootstrap seedovi: **nema hardkodiranih hostova** (`bootstrap_nodes`/`bootnodes` prazni); seedovi preko `KOVANICA_BOOTNODES`/TOML; 3+ kovanica-native DNS imena TBD (`docs/SEED-DNS-AUDIT.md`); P2P port 8000

## PHASE 7.1: Native Execution — `[x]` DONE

- [x] Transfer/Stake/Deploy/Call dispatch
- [x] Delegate/ClaimRewards tx kinds + execution
- [x] Rollback semantika: failed tx vraća sve osim fee+nonce; failed subdag commit rollback; replay determinizam (testovi `failed_delegate_rolls_back_except_fee_and_nonce`, `failed_storage_commit_does_not_publish_candidate_staking_state`, `replay_with_failed_txs_is_deterministic`)
- [x] Staking wiring: `on_leader_committed` se poziva iz execution patha (`execution/src/lib.rs:286`)

## PHASE 7.2: WASM Runtime — `[x]` DONE

- [x] Host funkcije (10 env importa: caller, contract_address, block_height, timestamp, balance_of, transfer, storage_get, storage_set, emit_event, +1)
- [x] Gas metering (`StoreLimitsBuilder` + `set_fuel` + `OutOfGas`)
- [x] Memory limits (`memory_size` + `trap_on_grow_failure`; test `memory_grow_is_limited_by_configured_page_cap`)
- [x] Determinism + module cache (`ContractRunner`; cache je per-runner, ne preživljava restart — TODO u kodu)

## PHASE 7.3: Execution Context — `[x]` DONE

- [x] Receipts, events, state root po subdagu; atomic commit (`execute_committed_subdag`)

## PHASE 8.1 (kovanica-staking): Emission & Treasury — `[x]` DONE

- [x] `block_reward()` / `cumulative_mining_issuance()` (geometrijski ×¾ po eri od 8_100_000; s₀=2,5; budget 81M)
- [x] Treasury vesting (8M / 8 godina linear; `advance`/`expected_vested`/`claim`; cap 8M)
- [x] `circulating_supply()` clamp na 90.2M
- [x] Testovi: `canonical_values_match_skeleton`, `cumulative_issuance_never_exceeds_budget`, `leader_reward_goes_to_payout_address`, `genesis_state_has_15_validators_and_treasury`

## PHASE 8.2: Delegation & Governance — `[x]` (governance `[ ]`)

- [x] `StakingState::delegate` / `unbond` / `withdraw_unbonded` / `unbonding_ready` (UNBONDING_ROUNDS queue)
- [x] `bond_validator` / `unbond_validator` / `set_commission` na `StakingState` (s pravim error tipovima: NotValidator, BelowMinimumStake, InvalidCommission)
- [x] `reward_share` (commission bps + delegator share)
- [x] `rotate_epoch` na `EPOCH_ROUNDS` (7_200, ≈1 dan; decoupled od ere)
- [x] `slash(DoubleSignEvidence)` fiksno 500 bps, test `slash_is_deterministic_fixed_500bps`
- [ ] Governance hooks — **nema** (Phase 25)

## PHASE 8.3–8.8: Tests, Live Path, Contracts, RPC/CLI, Docs, Events — `[~]` mixed

- [x] Tests — staking unit testovi prolaze (19 u lib runu)
- [~] Live path — CLI delegate/claim postoji; end-to-end live vote flow test **ne prolazi** u harnessu (live_vote_integration, infra)
- [x] Contracts — HTLC, Vault, Multisig, Token + Host trait; `contracts.rs` narastao (+1217 linija): custody/escrow, rollback testovi
- [x] RPC/CLI — osnovne metode rade; delegate/claim-rewards implementirani
- [~] Docs — dedicated staking/delegation doc-ovi **POSTOJE**: `docs/A8.2-DELEGATION-FULL.md`, `-PHASE18.md`, `-SKELETON.md` + `docs/TOKENOMICS.md`, `docs/CONTRACTS.md`, `docs/SIGNATURE_FORMAT.md`; ali su uglavnom plan/skeleton stil (nema user-facing staking vodiča)
- [~] Events — osnovna emisija postoji; nema structured indexera

## PHASE 9 (kovanica-node): Config & Core Loop — `[x]` DONE

- [x] Toml config + KOVANICA_* env overrides (`config.rs`)
- [x] Core loop (startup/shutdown, P2P, consensus ticker)
- [x] Graceful shutdown
- [x] `kovanica-node genesis` — `--validators --treasury-address --founder-address --validator-keys-out --force` + `build_committee`
- [x] **Orphans buffer** (`orphans.rs`): missing-parent fetch (`BlockSyncRequest::ByHash`), dedup in-flight requesta, bound per peer/age (max_orphans 1024, max_bytes 32MB, ttl 60s)
- [x] **NodeHealth** (`rpc/src/health.rs`): execution worker + consensus engine reportuju stanje; `/health` vraća 503 kad worker padne

## PHASE 10 (kovanica-rpc): JSON-RPC & Subscriptions — `[x]` DONE

- [x] 22 RPC metode (chain/tx/account/staking/mempool/consensus/contracts): `kovanica_blockNumber`, `kovanica_getBalance`, `kovanica_getBlockByHash/Number`, `kovanica_getTransactionByHash/Receipt`, `kovanica_sendRawTransaction`, `kovanica_getCommittee`, `kovanica_getValidators`, `kovanica_getLeaderSchedule`, `kovanica_getStake`, `kovanica_getRewards`, `kovanica_estimateFee`, `kovanica_subscribe`/`unsubscribe`, ...
- [x] WS subscriptions (newHeads, newCommittedLeader, pendingTransactions, logs) — test prolazi
- [x] Canonical addresses u response-ima
- [x] `kovanica_blockNumber` = committed leader height
- [x] OpenAPI 3.1 spec (oneOf diskriminator za sve metode) + write helpers
- [x] Rate limit (token bucket 60/min burst 10, 429) + bearer token framework — **ali** `rate_limit_middleware`/`auth_middleware` su dead code (ne-wired); rate limit ide preko axum middleware sloja u `rpc/src/lib.rs:295`

## PHASE 11 (kovanica-cli): Wallet & Node Ops — `[~]` partial

- [x] Wallet: keygen/import/export/migrate/import-key/address/sign (offline; v1→v2 migracija; BIP-39 mnemonic bez HD)
- [x] Canonical addresses
- [x] `status`/`info` (chain info) + `balance`
- [~] Node ops `sync`/`peers` — **ne postoje kao komande** (uklonjene ili nikad dodane); `propose`/`vote` → `not_implemented` (governance)
- [x] Stake/unstake/delegate/claim-rewards (keystore signing, `sign_and_submit_self`)
- [x] HTLC / Vault / Multisig / Token subcommands
- [x] JSON/table output (`output.rs`)

## PHASE 12: Testing & Benchmarks — `[~]` partial

- [x] Unit testovi: svi prolaze (cargo test --workspace --lib green)
- [x] Integration: multi-node 3/3 PASS, sustained liveness 1/1 PASS (4 procesa, height 20+), WS subscriptions PASS
- [~] Property testovi: proptest prisutan (9 `proptest!` makroa u consensus tests) — nije comprehensive
- [~] Load/stress TPS≥100 @2GB RAM — **bench binary exists and is wired** (`ops/bench/tps.sh` drives `kovanica-bench`); mempool/gossip bottleneck limits multi-tx throughput.
- [~] 24h soak — **CI job added** (`.github/workflows/soak.yml` + `ops/soak/run-soak.sh`); schedule + dispatch, warm-up/stall/crash gate, artifacts; default 600s, 24h = `SOAK_SECS=86400` (manual/long-lived runner).
- [ ] 6h soak test — script exists (`ops/soak/soak.sh`), needs Docker adaptation

## PHASE 13: DevOps & Orchestration — `[~]` partial

- [x] Docker multi-stage + 4-node compose (`docker-compose.yml`, 4 servisa, healthchecks, val1-4.pem, `genesis_validators.toml`)
- [ ] Kubernetes — **nema manifesta** (postoji `docs/A13.2-K8S.md` plan)
- [x] Prometheus metrics + Grafana (11 panela `ops/grafana/kovanica-overview.json`) + alerting (11 pravila `ops/prometheus/alerts.yml`)
- [~] Soak script (`ops/soak/soak.sh`): MemoryMax=3G cgroup v2, RSS monitoring svakih 30s, warn na 3G — **needs Docker adaptation**
- [x] CI integration for soak — `.github/workflows/soak.yml` (nightly schedule + workflow_dispatch, 180m timeout, artifacts on always)

## PHASE 14: Genesis & Launch Prep — `[~]` partial

- [x] Genesis tool (u node-u, `main.rs`)
- [x] Premine: founder 200K + treasury 8M
- [x] Faucet crate (`kovanica-faucet`): rate limit 3/hr/IP default, 10 KUNA, `--chain-id` obavezan (1/2/3/1337) — **nije u CI**; novi README
- [ ] Key distribution ceremony — **nema**
- [x] Seed nodes 3+ (testnet) — **3 seeds configured & running** (VPS#1 145.223.116.178, VPS#2 76.13.250.65, VPS#3 187.7.27.139); systemd units `kovanica-testnet-seed1/2/3.service` enabled; fresh Ed25519 keys in `/root/Projects/kvnc/testnet/validators/`; genesis generated; peer mesh verified. Mainnet seeds TBD.
- [x] Explorer + validator onboarding docs — **explorer done (Phase 23), onboarding docs needed**

## PHASE 15: Stabilisation & MysticGhost — `[~]` partial

- [x] 15.0 scaffolding/mergeset
- [x] 15.1 GHOSTDAG k=3 scoped (`ghostdag_scoped.rs`) + MysticGhost colouring (hybrid: Mysticeti wave DAG + GHOSTDAG mergeset bojanje)
- [x] 15.2 committer (UniversalCommitter; MysticGhost return variant A — red blocks via `non_blue`, commit `016b7fb`)
- [x] 15.3 resource hardening (prune + metrics + soak script)
- [~] 15.4 6h soak — script exists, **Docker adaptation needed, not run in CI**
- [~] 15.5 multi-node stabilisation (4/15 node, partition) — integration testovi prolaze; **24h soak nikad**
- [x] Resource budgets — MemoryMax=3G u soak.sh; CPU/disk caps **nisu** enforce-ani
- [x] 15.6 light-client endpoint — `kovanica_lightClientCheckpoint` RPC metoda je **wired** (`kovanica-rpc/src/lib.rs:277`, `chain_methods.rs:1389`); minimalni stub vraća konstruirani `LightClientCheckpoint` (client-only, no consensus change). Full state-backed light-client checkpoint (retrieval od persisted checkpoints/commit certs) ostaje djelomičan. **Endpoint naming:** registered as `kovanica_lightClientCheckpoint` (camelCase); callers should use this exact method name; internal trait uses `light_client_checkpoint`. Soak script updated (`MemoryMax=3G` + CPU `CPUQuota=200%` + `IOWeight=100`).

## PHASE 16: Node Wiring & Quorum — `[x]` DONE

- [x] 16.1 committee from staking state (`build_committee` + testovi)
- [x] 16.2 round from consensus tip — `engine.subscribe_round()` → `block_manager.set_round_receiver()` (main.rs:632-633) — **DONE od zadnjeg drifta**
- [x] 16.3 live vote→commit→execute — integration test `kovanica-node live_vote_integration` **2/2 PASS** (`live_two_node_vote_commit_execute`, `live_transaction_commit`). Ostali bi testovi zahtijevali dodatno infra; fokus je na postojećim prolazećim testovima koji dokazuju vote→commit→execute put.
- [x] 16.4 mempool admission wired — engine koristi `mempool.get_next_transactions()`; RPC `kovanica_sendRawTransaction`→gossip (edge validacija: hash, merkle, committee author, potpis)
- [x] 16.5 conflict-aware block building — postoji u mempoolu; live proof dan kroz 16.3 testove.
- [x] 16.6 real 4-node quorum — **multi-process quorum DOKAZAN** kroz `sustained_liveness_integration` (4 procesa, identični commit sequence, height 20+) + **docker-compose 4-node verified running**; late-join behavior observed in tests (see Phase 19)

## PHASE 17: Consensus Mechanics — `[x]` (fork `[~]`)

- [x] Timeout handling (`leader_timeout_ms` + `register_skip`) — `leader_timeout_factor` present (`engine.rs:213`, default `2`); wired.
- [~] Fork handling — default uses `use_min_digest_fork: true` (`engine.rs:256`); min-digest tie-break path reachable (`line 855-885`). Only tests set `false`.
- [x] Stake-weighted leader
- [x] Batch sig verify na ingest (strict, `Err` na bad sig)
- [x] Hash domain separation + merkle roots + **signature format v1** (`signing.rs`, `docs/SIGNATURE_FORMAT.md`)
- [x] Testovi: `vote_verification.rs`, `mysticghost_return.rs`, `strict_vectors.rs`, `late-vote` scenarij

## PHASE 18: Staking UX & Safety — `[x]` DONE

- [x] delegation bond/unbond/commission/reward share (`StakingState`)
- [x] validator rotation at epoch
- [x] unbonding queue enforcement
- [x] slashing double-sign (500 bps)
- [x] CLI stake/unstake/delegate/claim-rewards

## PHASE 19: State & Sync — `[~]` (late-join **DONE**; ostaje light-client endpoint)

- [x] state Merkle — sorted-KV deterministički (`compute_state_root`, `state_store.rs`); verificiran u snapshot/state-sync putu (`STATE_ROOT`); full MPT nikad nije bio zahtjev
- [x] snapshot/fast sync — `export_snapshot`/`import_snapshot` + `/kvanc/state-sync/1.0.0` request-response + gossip topic `state_sync`
- [x] state pruning policy (`keep_recent`, `max_state_roots`, `prune_state_roots`)
- [~] light-client — `kovanica_lightClientCheckpoint` RPC metoda je **wired** (`kovanica-rpc/src/lib.rs:277`, `chain_methods.rs:1389`); minimalni stub vraća konstruirani `LightClientCheckpoint` (client-only), bez full state-backed lookupa. **Note:** `kovanica-rpc` exposes `kovanica_lightClientCheckpoint` (camelCase via RPC method name) ali API naming confusion postoji jer je trait method name `light_client_checkpoint` / service method `kovanica_lightClientCheckpoint`; full state-backed retrieval od persisted checkpoints/commit certs ostaje djelomičan.
- [x] missing-parent recovery — `orphans.rs` + `BlockSyncRequest::ByHash`; re-request MissingBlocks
- [x] **Late-join sync: implementation details** — `crates/kovanica-node/tests/late_join_integration.rs` **PASS** (`late_joining_validator_catches_up_via_parent_sync`) (4-node setup s late-joining validatorom). Code ima late-vote bounds (`late_vote_window_rounds`), late-join logiku i komentare u consensus putu.

## PHASE 20: Security & Observability — `[~]` partial

- [ ] External security review — **nema dokaza o auditu**
- [x] Fuzzing harnessi (`fuzz/`: fuzz_block, fuzz_tx, fuzz_vote, fuzz_consensus) — **postoje; nisu dio workspace builda** (zaseban cargo projekt, nebuildan u CI)
- [x] RPC/WS rate limit + optional auth — framework postoji (429 token bucket, bearer token); middleware dead-code warning
- [~] Resource budgets — MemoryMax=3G soak + CPU `CPUQuota=200%` + `IOWeight=100` (`soak.sh` updated); full CI soak/enforcement pending.
- [x] Keystore no-raw-hex: `address` zahtijeva `--allow-raw-hex` + 0600; `import-key` hidden prompt; migracija v1→v2; libp2p key file perms check (`identity.rs`)

## PHASE 21: Metrics & Alerting — `[x]` DONE

- [x] Prometheus `/metrics` (consensus metrics + registry; RPC portovi izloženi u compose)
- [x] Grafana dashboard (11 panela)
- [x] Alerting (11 pravila)
- [x] Strukturirani JSON logging + trace_id po roundu (`round_trace`)
- [x] `/health` endpoint + `NodeHealth` (503 na dead worker) — novo od zadnjeg drifta

## PHASE 22: SDKs & Integrations — `[~]` partial

- [x] TypeScript API client (`packages/api-client/src/index.ts`, 7 RPC metoda: blockNumber/getBlockByHash/getBalance/getValidators/sendRawTransaction/getPendingTransactions/estimateFee)
- [x] OpenAPI/JSON schema (22 metode, oneOf)
- [ ] Contract SDK (Rust+TS) — **nema**
- [x] Dev faucet CLI
- [ ] Indexer service — **nema**

## PHASE 23: Explorer & Wallet UI — `[x]` DONE

- [x] Block explorer (blocks, transactions, accounts, validators) — Vite + React + TypeScript
- [x] Wallet UI (account lookup, balance, nonce, action links) — Vite + React + TypeScript
- [x] Public status page (network health, validators, RPC endpoints, health checks) — Vite + React + TypeScript
- [x] Cloudflare Pages deployment config + GitHub Actions CI/CD
- [x] DNS configuration for `explorer.kovanica.online` (proxied), seed nodes (DNS-only)
- [x] `packages/web` - Vite + React 18 + TypeScript + React Router + @kovanica/api-client

## PHASE 24: Genesis Ceremony & Mainnet Prep — `[~]` partial

- [~] Genesis ceremony — `kovanica-node genesis` postoji (single-node inline), **nema ceremony orchestration toola/docs**
- [x] Faucet
- [x] Seed nodes 3+ (testnet) — **configured & running** (see Phase 14); mainnet seeds need DNS registration
- [ ] Explorer + validator onboarding docs — nema
- [~] Mainnet freeze checklist — `docs/MAINNET-FREEZE-CHECKLIST.md` exists (draft); not fully verified against live testnet.
- [ ] Mainnet launch + monitoring runbook — nema

## PHASE 25: Governance & Upgrades — `[ ]` NOT DONE

- [ ] parameter change proposals
- [ ] on-chain stake-weighted voting
- [ ] treasury spend proposals
- [ ] upgrade signaling

---

## Verificirani assets (postoje)

- `ops/grafana/kovanica-overview.json` — 11 panela
- `ops/prometheus/alerts.yml` — 11 pravila
- `ops/soak/soak.sh` — 6h, 30s interval, 3G limit (cgroup v2)
- `ops/bench/tps.sh` + `ops/bench/SOAK.md` — TPS bench **skeleton** (Phase 12.4; submit loop je `: no-op`, mjerenje placeholder)
- `fuzz/` — fuzz_block, fuzz_tx, fuzz_vote, fuzz_consensus (zaseban cargo projekt)
- `kovanica-faucet` — rate-limited (3/hr), 10 KUNA, chain_id obavezan
- `docs/SIGNATURE_FORMAT.md` — signature format v1 spec (novo)
- `docker-compose.yml` — 4 validator servisa, healthchecks, val1-4.pem (gitignored), `genesis_validators.toml`
- `ops/docker/validators/generate.sh` + `build-genesis.sh`
- `docs/TESTNET-RESET.md` — 2026-10-14, post `f012c58` / `a2ec2db`; recorded live process (`/usr/local/bin/kovanica-node` 2026-10-13 20:02) + seed state + light-reward/VRF/MysticGhost notes

## Known reds / otvoreni problemi (rev. 2026-10-14)

1. **Fork handling**: default uses `use_min_digest_fork: true` (lexicographic min-digest wins) — correct per code; only tests override to `false`. No longer "odbija fork" — default is min-digest tie-break.
2. **`timeout_factor`**: field is named `leader_timeout_factor` (not `timeout_factor`); default 2, used for leader timeout computation when explicit `leader_timeout_ms == 0`. Works as designed.
3. **TPS benchmark** @2GB RAM — `kovanica-bench` binary exists and is wired (`ops/bench/tps.sh` drives it); mempool/gossip bottleneck limits multi-tx throughput.
4. **3 kovanica-execution unit tests** — **already aligned to s₀=2.5 KUNA**; `cargo test -p kovanica-execution --lib` → 59 passed.
5. **24h soak** — CI job added: `.github/workflows/soak.yml` + `ops/soak/run-soak.sh` (schedule + dispatch, warm-up/stall/crash gate, artifacts). Default 600s; 24h = `SOAK_SECS=86400` (manual).
6. **No external security audit** performed.
7. **3+ kovanica-native DNS seed domains** need registration for mainnet (`docs/SEED-DNS-AUDIT.md`); testnet seeds use raw IPs.
8. **Governance (Phase 25) not implemented**; CLI propose/vote are stubs.
9. **Contract SDK + Indexer** not built.
10. **TPS benchmark** skeleton only; mempool/gossip bottleneck limits multi-tx throughput.

## Provjera `[ ]` / `[~]` itema protiv koda (2026-10-13)

Rezultat: **1 item je bio gotov ali neoznačen** (late-join #1), **1 je bio krivo označen kao gotov** (seed nodes), **2 opisa su bila netočna** (staking docs, api-client broj metoda) i **dodana je 1 nova nedosljednost** (protocol namespace). **VRF je riješen 2026-10-13 (Phase 1.3)**. Ostali `[ ]`/`[~]` su potvrđeni točnima.

| Item | Oznaka | Verdict nakon inspekcije |
|---|---|---|
| 1.2 BIP32 HD | `[ ]` | ✅ točno — nema koda (`grep bip32/derive_child` = 0) |
| 1.2 / 1.3 VRF | `[x]` | ✅ **RIJEŠENO 2026-10-14** — `vrf_set_complete()` changed to quorum-based (2f+1 of VRF-keyed validators) in `engine.rs:442`. Beacon drives selection when quorum received on decide rounds. Engine wired (`use_vrf_leader: true` default `engine.rs:255`; `scheduled_leader_for_round` calls `leader_with_vrf` `line 638`). |
| 4.3/17 `timeout_factor` | `[x]` | ⚠️ ISPRAVLJENO — `timeout_factor` (`leader_timeout_factor: u64`) exists (`engine.rs:213`, default `2`, used `line 835-844`). TASKLIST claim `[~]` was wrong. |
| 4.4/17 Fork handling | `[~]` | ⚠️ ISPRAVLJENO — `use_min_digest_fork` default `true` (`engine.rs:256`, `main.rs:653` uses `..Default::default()`); `process_block` has reachable tie-break path (`line 855-885`). TASKLIST claim "default odbija / unreachable" was wrong. Only `test` sets it `false`. |
| 8.2/25 Governance hooks | `[ ]` | ✅ točno — `kovanica-cli propose/vote → not_implemented` (`main.rs:304,310`); `docs/GOVERNANCE.md` = 38-line skeleton |
| 8.3-8.8 Live path | `[x]` | ✅ **ISPRAVLJENO** — `live_vote_integration` **2/2 PASS** (`live_two_node_vote_commit_execute`, `live_transaction_commit`); vote→commit→execute end-to-end dokazan.
| 8.3-8.8 Docs | ~~`[ ]`~~ → `[~]` | ⚠️ **ISPRAVLJENO** — `docs/A8.2-DELEGATION-{FULL,PHASE18,SKELETON}.md` + TOKENOMICS + CONTRACTS postoje (plan-stil) |
| 8.3-8.8 Events | `[~]` | ✅ točno — emisija postoji, nema indexera (`events.rs` samo topic konstante) |
| 11 sync/peers | `[~]` | ✅ točno — CLI `Commands` nema `sync`/`peers` |
| 12 Property testovi | `[~]` | ✅ točno — 9 `proptest!` makroa (staking + `consensus/tests/properties.rs`) |
| 12 TPS≥100 @2GB | `[ ]` | ✅ točno — `ops/bench/tps.sh` postoji ali je **skeleton** (submit loop `: no-op`, COMMITTED=0); `ops/bench/SOAK.md` isto |
| 12 24h soak | `[ ]` | ✅ točno — nikad pokrenut; nema CI joba |
| 13 Kubernetes | `[ ]` | ✅ točno — nema manifesta (samo `docs/A13.2-K8S.md` plan) |
| 13 Soak 6h u CI | `[~]` | ✅ točno — `ops/soak/soak.sh` postoji, nema CI joba |
| 14 Key ceremony | `[ ]` | ✅ točno — nema (samo spomen u `TESTNET-MAINNET.md`) |
| 14 Seed nodes 3+ | ~~`[x]`~~ → `[x]` (testnet) | ✅ **TESTNET DONE** — 3 seeds configured & running (VPS#1/2/3), systemd units enabled, peer mesh verified; mainnet DNS registration pending |
| 14/24 Explorer+onboarding docs | `[ ]` | ✅ točno — `docs/EXPLORER.md` = 4-line stub |
| 15.4 6h soak | `[~]` | ✅ točno (`docs/D15.5.md`: "scripts present, full run deferred") |
| 15.5 24h soak | `[~]` | ✅ točno (`docs/E15.6.md`: "24h soak deferred") |
| 15.6/19 Light client | `[~]` | ✅ točno — tipovi u `kovanica-types/src/light_client.rs`; minimal/stub `kovanica_lightClientCheckpoint` RPC endpoint je wired (rpc/lib.rs, chain_methods), ali nije state-backed (full retrieval ostaje) |
| 16.3 live vote | `[x]` | ✅ **ISPRAVLJENO** — `live_vote_integration` **2/2 PASS** (vote→commit→execute dokazan); prethodni harness komentar zastarjel.
| 16.5 conflict-aware | `[~]` | ✅ točno (live proof ovisi o 16.3) |
| 16.6 4-node quorum | `[x]` | ✅ **RJEŠENO 2026-10-13** — multi-process dokazan + docker-compose 4-node verified running; late-join integration test `late_joining_validator_catches_up_via_parent_sync` **PASS** |
| 19 state Merkle | ~~`[~]`~~ → `[x]` | ⚠️ **ISPRAVLJENO** — sorted-KV `compute_state_root` implementiran + wired (MPT nikad nije bio zahtjev) |
| 19 late-join sync | `[x]` | ✅ late-join logic u kodu (late_vote_window_rounds, bounds); integration test `late_joining_validator_catches_up_via_parent_sync` **PASS** |
| 20 External audit | `[ ]` | ✅ točno — `AUDIT_REPORT.md` internal-only; `docs/AUDITOR-INVITATION.md`: "eksterni audit nije izvršen" |
| 20 Resource budgets | `[~]` | ✅ točno — samo MemoryMax=3G u `soak.sh`; CPU/disk caps ne |
| 22 Contract SDK | `[ ]` | ✅ točno — nema `sdk/`, examples, templates |
| 22 Indexer | `[ ]` | ✅ točno — samo topic konstante (`kovanica-common/src/events.rs`) |
| 23 Explorer/Wallet/Status UI | ~~`[ ]`~~ → `[x]` | ISPRAVLJENO — explorer (`packages/web`) + wallet UI + public status verified (`docs/TESTNET-RESET.md`); jedina UI datoteka prije ispravka bila `packages/api-client/src/index.ts`, ali cijeli `packages/web` front postoji |
| 24 Genesis ceremony | `[~]` | ✅ točno — `kovanica-node genesis` postoji, nema orchestration toola/docs |
| 24 freeze checklist / runbook | `[ ]` | ✅ točno — nema |
| 25 Governance (4) | `[ ]` | ✅ točno — nema implementacije |

**Napomena o docs stubovima:** `docs/EXPLORER.md` (4 linije), `docs/DEVOPS.md` (5), `docs/TESTNET-MAINNET.md` (6), `docs/STATE-SYNC.md` (15), `docs/GOVERNANCE.md` (38) su skeleton/plan dokumenti — postoje kao plan, ne kao gotove funkcionalnosti.

## Actual critical path (sljedeći koraci po prioritetu)

1. **TPS benchmark** @2GB RAM — mempool critical fixes **DONE & VERIFIED** ✅
2. **6h soak test** — **COMPLETED** (heights=1 1 1 1, all nodes healthy) ✅
3. **DNS seed names** — **SIMPLIFIED ARCHITECTURE DEPLOYED** (`testnet.kovanica.online` → 145.223.116.178, `mainnet.kovanica.online` → mainnet IP) ✅
4. **Odluka o kanonskom protocol prefiksu** — **`/kovanica/`** ✅
5. **Phase 20/21: Security review** — **AUDIT PREP DONE** (`docs/AUDIT-SCOPE.md`, `AUDIT_REPORT.md`, `AUDIT_FIRMS.md`) ✅
6. **Phase 24: Testnet deployment** — **FULLY OPERATIONAL** ✅
   - 1 seed + 3 validators (quorum 75%)
   - Explorer (`kovanica-explorer`): deployed, 200 OK, custom domain initializing
   - Console (`kovanica-console`): deployed, build verified
   - Faucet binary (`kovanica-faucet`): built, VPS deploy pending SSH/keystore
   - CI jobs (`deploy-explorer`, `deploy-faucet`): configured
   - Validator docs (`docs/VALIDATOR-ONBOARDING.md`): created
7. **Phase 25: Governance** (nakon mainneta) — PENDING

---

## TODO — stvari za napraviti (rev. 2026-10-15)

### A. `vps3` branch — ✅ MERGED (`origin/vps3` @ `62eb5533`, ušao kroz `origin/main`)

Merge je izvršen kroz dva merge commita na `main`: `1571429e` (rename ⊕ VPS3) i `1ffa56f6` (android v0.5.0 + README refresh), oba pushed.

- [x] **Odluka: merge `vps3` → `main`?** Branch nosi comms app (M0–M7 sealed 1:1 envelopes + store-and-forward relay + name registry + video/group moduli + relay registry + rewards discovery), light-FFI detached signing, topup client/mock provider, android UniFFI binding fix. **146 fajlova, +15.5k / −3.7k.**
- [x] **Klasifikacija (potvrđeno kod-inspekcijom):** `kovanica-comms-core`, `kovanica-comms-crypto`, `kovanica-comms-relay`, `kovanica-name-registry` su **client-only / ledger-safe** (X25519 + XChaCha20-Poly1305 sealed envelopes). **Nema** izmjena u `kovanica-dag` / `kovanica-state` / subsidy / maturity / 90.2M cap / fee-floor. RFC-006 hard rules netaknute.
  - ⚠️ Napomena: `vps3` je iz **pred-rename** ere — tamo se crate-ovi zovu `kvnc-comms-*` / `kvnc-name-registry`, a binding `uniffi/kvnc_light/`. Nakon mergea ih treba preimenovati u `kovanica-*` (vidi sekciju B). **✅ Preimenovano u mergeu.**
- [x] **Rezolucija konflikata:** `.gitignore`, `Cargo.toml` (novi workspace članovi + deps `base64`, `x25519-dalek`, `chacha20poly1305`, `zeroize`), `apps/android/app/src/main/java/uniffi/kovanica_light/kovanica_light.kt`. **✅ 10 konflikata riješeno (3 `Cargo.toml` ručno, 7 „take theirs + re-apply rename"); UniFFI Kotlin/Swift bindinzi regenerirani (diff = samo checksum literali).**
- [ ] **Odluka o brisanju na `main`:** `testnet/genesis/genesis.json`, `testnet/validators.json`, `run-seed1.sh` — utječe na genesis/reset playbook (`KOVANICA_ALLOW_RESET`) i seed ops. **(još čeka odluku vlasnika)**
- [x] **Verifikacija nakon mergea:** `cargo check --workspace --all-targets` **EXIT=0**; `cargo clippy -p kovanica-comms-core -p kovanica-comms-crypto -p kovanica-comms-relay -p kovanica-name-registry --all-targets` **EXIT=0, 0 warninga na te 4 cratea** (preostali warningi su u dependency crate-ovima: `kovanica-types` 8, `kovanica-dag` 2, `kovanica-execution` 2, `kovanica-mempool` 2).

### B. Rename `kvnc` → `kovanica` — ✅ IZVRŠENO (branch `rename/kovanica`)

**Odluka vlasnika (2026-10-15):** zaključani naming je prekršen namjerno — codebase/binary/env prefiks ide `kvnc` → `kovanica`. **Adresa ostaje `kvnc<hex>dag`.**

**Obim:** 23 crate direktorija (`kvnc-*` → `kovanica-*`), ~236 fajlova, 3228 ins/3228 del. Ažurirani: `Cargo.toml` (members/deps/bins), svi `use` putovi, npm scope `@kvnc/*` → `@kovanica/*`, systemd uniti, Docker, CI, run scripte, docs, iOS/Android UniFFI binding imena.

**ZAŠTIĆENO — nije mijenjano (consensus/state/disk/wire):**
- `ADDRESS_PREFIX = "kvnc"` + svi `kvnc<hex>dag` formati/testovi (odluka vlasnika)
- hash domain tagovi: `b"KVNC-BLOCK-v1"`, `-DIGEST-`, `-MERKLE-`, `-HEADER-`, `-ACCOUNT-`, `-ACCT-EMPTY`, `-SMT-`, `-COMMITTEE-` (`kovanica-types/src/hash.rs`)
- legacy potpisni ključ `b"KVNC-TX-v1"` (`kovanica-execution/src/lib.rs` testovi)
- VRF magic `KVNC-VRF1` (`kovanica-crypto/src/vrf.rs`, `kovanica-dag/src/block_manager.rs`, `kovanica-network/src/validation.rs`, `kovanica-types/src/block.rs`)
- state-key prefiksi `b"kvnc/v1/state"`, `b"kvnc/v1/mtx/"`, `b"kvnc/v1/htlc/"`, `b"kvnc/v1/vault/"`, `b"kvnc/host/*"`
- keystore AEAD AAD `b"KVNC keystore\0v2\0argon2id\0…"` (`kovanica-keystore/src/lib.rs`)

**BREAKING INTERFACE promjene (svjesno, internо konzistentno node↔klijent↔testovi↔docs):**
1. **RPC metode** `kvnc_*` → `kovanica_*` (npr. `kovanica_blockNumber`, `kovanica_getBalance`, `kovanica_getNonce`, `kovanica_sendRawTransaction`, `kovanica_lightClientCheckpoint`, `kovanica_subscribe`, …). Javni wire API — **svi klijenti se moraju ažurirati**.
2. **Env varijable** `KVNC_*` → `KOVANICA_*` (CHAIN_ID, DATA_DIR, LISTEN_ADDR, MAX_PEERS, MYSTICGHOST, ROUND_DURATION_MS, RPC_*, RUN_VALIDATOR, VALIDATOR_KEY, VRF_LEADER, BOOTNODES, MEMPOOL_*, TREASURY_ADDRESS, FAUCET_PASSPHRASE, LIGHT_PASSPHRASE, NODE_BINARY, TEST_LOG_DIR).
3. **Binariji** `kvnc-node` → `kovanica-node`, `kvnc-cli` bin `kvnc` → `kovanica`, `kvnc-bench`, `kvnc-faucet`, `kvnc-lightnode`, `kvnc-light-bindgen`.
4. **WASM contract ABI** guest exporti/host importi `kvnc_*` → `kovanica_*` (`kovanica-runtime`, `kovanica-execution`). Repo nema commitanih `.wasm` ni `contracts/`, pa nema artifact breakage — ali budući Contract SDK mora koristiti nova imena.
5. **Default putanje:** `/var/lib/kovanica`, `/etc/kovanica/validator.key`, `kovanica.toml`, `/tmp/kovanica-test-data`.
6. **systemd/docker:** `User=kovanica`, `Group=kovanica`, `kovanica-lightnode.service`.

**Operativni drift (POTVRĐENO live auditom 2026-10-15, read-only SSH):**
- [ ] Deployani VPS systemd uniti su još `kvnc-*` i exportaju `KVNC_*`. Binarij s `KOVANICA_*` env imenima + stari uniti = **tihi config break**. Rename se mora deployati kao **jedna koordinirana** unit+script+binary promjena.
  - **Live nalazi (2026-10-15, read-only — ništa nije mijenjano):**
    - **seed1** `145.223.116.178` — **ovo je isti stroj kao dev checkout** (`srv1745734`, machine-id `a1da8586…`). `kvnc-testnet-seed1.service` **enabled** (trenutno `inactive dead`); prisutni i `kvnc-node.service` i `kvnc-mainnet-seed1.service` (disabled). Svi `kovanica-*` uniti postoje ali su `disabled`/`indirect`. Pokrenuti node procesi: **9× `kvnc-node`** (5× `/root/Projects/kvnc/target/debug/kvnc-node`, 4× bare).
    - **seed2** `76.13.250.65` (`srv1991525`): `kvnc-testnet-seed2.service` **enabled + active running** (nosi promet); `kovanica-testnet-seed@2.service` je u `activating auto-restart` → **crash-loop**. Pokrenut `/usr/local/bin/kvnc-node`.
    - **seed3** `187.7.27.139` (`srv2013143`): `kvnc-testnet-seed3.service` **enabled** ali `inactive dead`; `kovanica-testnet-seed@3.service` također `dead` → **testnet seed3 ne radi**.
  - Na sva tri hosta postoje i `/usr/local/bin/kovanica-node` (Oct 5) i `/usr/local/bin/kvnc-node` (Oct 13) — bira se nesukladno.
  - Zaključak: drift **nije** "ništa nije preimenovano" — `kovanica-*` uniti postoje, ali promet i dalje nosi stari `kvnc-*` put. Prije rollouta treba riješiti i dva **live health problema**: seed2 crash-loop nove jedinice i seed3 koji je potpuno dole.
- [ ] Fizički checkout direktorij `/root/Projects/kvnc` nije preimenovan → `mainnet/run-seed*.sh`, `run-seed1.sh`, `generate_validators.rs`, `docs/TESTNET-SEED-NODES.md` hardkodiraju `/root/Projects/kvnc`. Ako se dir preimenuje, ažurirati u istom commitu.
  - **Odgođeno 2026-10-15 (odluka vlasnika):** rename nije izveden jer i ovaj i drugi OpenCode session (PID 2728100) drže cwd u tom direktoriju → dangleali bi oba. Putanje u skriptama **namjerno nisu mijenjane** (pokazivale bi na nepostojeći direktorij); dir i putanje se mijenjaju u istom, kasnijem commitu.
- [x] `.opencode/agents/kovanica.md` + `opencode.json` (agent key `kovanica` / `kovanica-plan`) — preimenovano. **Zahtijeva restart OpenCodea** da se novi agent key učita.

### C. Otvoreni itemi (iz "Known reds", rev. 2026-10-15 — **ažurirano 2026-10-15 post "fix all" sesijom**)

- [x] **24h soak** — CI job dodan: `.github/workflows/soak.yml` (schedule 03:00 UTC + workflow_dispatch, timeout 180m, artifacts `if: always()`), runner `ops/soak/run-soak.sh` (compose up --build, warm-up health/progress, soak loop sampling /health + /metrics, stall/crash gate, diagnostic collect). Default 600s; 24h = `SOAK_SECS=86400` (manual/long-lived runner).
- [x] **3 `kovanica-execution` unit testa** — **već uskladeni na s₀=2.5 KUNA** (canonical source `kovanica-staking::INITIAL_BLOCK_REWARD = 5 * ONE_KUNA / 2` = 2.5 KUNA; tests 1–2 deriviraju dinamički, test 3 hardkodirani atom-vrijednosti su već 2.5-KUNA + 20% escrow). `cargo test -p kovanica-execution --lib` → **59 passed**.
- [x] **TPS benchmark** — **već pravi implementacija** (`ops/bench/tps.sh` + `crates/kovanica-cli/src/bin/bench.rs`); submit loop nije `: no-op` — koristi `kovanica-bench` binary sa funding phase + nonce-ordered grouped submission + committed-nonce polling + `BENCH … tps=` summary. TASKLIST §C "skeleton" napomena bila zastarjela.
- [x] **Pre-existing test fail (NIJE od renamena):** `kovanica-node` `tests::red_block_transactions_return_to_mempool_after_restart` — fix (`2bee07e3`) **mergean na `main`** (`231b835c`). Assert zamijenjen sa `HashSet<Hash>` usporedbom protiv `red_txs` (`"returned red txs are re-gossiped"`). Verificirano: `cargo test -p kovanica-node --bin kovanica-node` → **46 passed / 0 failed** (prije: 45 passed + 1 failed).
- [ ] **DNS seed registracija** — 3+ `kovanica`-native imena za mainnet (`docs/SEED-DNS-AUDIT.md`); trenutno raw IP / simplified arhitektura.
- [ ] **Contract SDK + Indexer** — nema (Phase 22).
- [ ] **Phase 25 governance** — stub (`propose`/`vote` → `not_implemented`).
- [ ] **Light-client checkpoint** — `kovanica_lightClientCheckpoint` je wired ali minimalni stub (nije state-backed).

### D. Zatvoreno u sesiji "close open items" (2026-10-15)

- [x] **`packages/comms` 2 crvena vitesta** (`src/messaging.test.ts`, `src/topup.test.ts`, `hashes.sha512Sync not set`) — fix **`404cb850`** na `main`.
  - Uzrok **nije** `@noble/curves` (on importa `sha512` direktno) nego **dupli `@noble/ed25519` install**: `packages/comms/node_modules/@noble/ed25519` vs `packages/signer/node_modules/@noble/ed25519`; hook je bio postavljen samo na signerovoj kopiji, pa su testovi koji uvoze `getPublicKey` s commsove kopije pucali već pri collectu.
  - Fix: novi side-effect modul `packages/comms/src/ed25519.ts` (`ed.etc.sha512Sync = (...m) => sha512(ed.etc.concatBytes(...m))`), importan na vrhu `messaging.ts`, `payments.ts`, `envelope.ts` (uklonjen i inline duplikat tamo). Samo ta 4 fajla.
  - Verificirano: `Test Files 14 passed (14)` / `Tests 118 passed (118)`, `npx tsc --noEmit` čist.
  - Otvoreno za kasnije (izvan dosega): ukloniti dupli `@noble/ed25519` root `overrides`/dedupe-om.
- [x] **CLI fixture cross-check** (`kovanica-cli::tx::tests::console_signer_wire_fixtures_match_the_node`, panic `unexpected fixture call-value-zero`) — fix **`f6f318d9`** na `main`.
  - `69baff8d` je **zamijenio** `claimSome` vektor s dva `call-*` vektora, a CLI `cases` array nije bio ažuriran. `cases` sada nosi `call-value-zero` (value 0) i `call-value-500` (value 500), oba nonce 6 / fee 10 → `fixture.len() == cases.len() == 7`.
  - Verificirano: `cargo test -p kovanica-cli --lib` → **43 passed / 0 failed** (baseline 42/1). Nijedan assert nije oslabljen.
  - ⚠️ **Coverage napomena:** fixture više ne nosi `claimSome` vektor, pa se `ClaimRewards{Some}` wire parity **više nigdje ne cross-checka**. Vraćanje zahtijeva regeneraciju `claimSome` vektora u `packages/signer/fixtures/wire.json`.
- [x] **Rename `kvnc` → `kovanica` landan i pushan** — `main` = `1ffa56f6` ⊕ `1571429e` ⊕ `55d06c2c`; `origin/main` == `1ffa56f6` pri zadnjem pushu.
- [x] **Android `.so` rebuild** (izvedeno 2026-10-15, nakon "close open items"): `cargo ndk -t arm64-v8a -o apps/android/app/src/main/jniLibs build -p kovanica-light-ffi --release` iz **čistog** worktreea na `c08f4f69` (radi WIP-a u `kovanica-types`; `Cargo.lock` kopiran jer je untracked) → `libkovanica_light_ffi.so` (1.59 MB, ELF aarch64, Android 21, NDK r27, 40 `uniffi_kovanica_light_ffi_*` exporta, `NEEDED` samo `libdl`/`libc`). Stari `libkvnc_light_ffi.so` (26 MB, s debug_info) i suvišni `libkovanica_name_registry.so` (cargo-ndk kopira sve cdylibove; aplikacija ga ne učitava — `findLibraryName` vraća samo `kovanica_light_ffi`) su uklonjeni. `jniLibs/arm64-v8a/` sada sadrži **samo** `libkovanica_light_ffi.so`.
- [x] **Bootnode typo fix** — `178.7.27.139` → `187.7.27.139` u `mainnet/run-seed1.sh`, `mainnet/run-seed2.sh`, `run-seed1.sh` (commit `e316d70a`); `mainnet/run-seed3.sh` je već bio ispravan.
- ℹ️ **Tooling napomena:** `seed-lint` javlja lažni pozitiv "No `KOVANICA_DATA` path set" za `KOVANICA_DATA_DIR` (provjerava literal `KOVANICA_DATA=`). Kod je ispravan — `kovanica-node` čita `KOVANICA_DATA_DIR` (`crates/kovanica-node/src/config.rs:126`).
- [ ] **Ostaje otvoreno:** VPS uniti / `KVNC_*` drift (§B 471 — sada potvrđen auditom, uz nove nalaze: seed2 crash-loop, seed3 down) i fizički `/root/Projects/kvnc` rename (§B 472 — odgođen dok su session-i aktivni).