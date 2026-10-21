# VRF proof length — 96 B vs 80 B — verdict

**Status:** resolved 2026-10-21 (TASKLIST item 15). **No code change was required.**
**Subject:** the ECVRF proof carried in `StatementBlock::statements` and in the
`SubmitLightWitness` transaction.

## TL;DR

| Claim | Verdict |
|---|---|
| "The Kovanica VRF proof is 96 bytes" | **True.** The vendored `ecvrf` 0.4.3 crate emits exactly 96 bytes and the compiler enforces it (`VrfProof::to_bytes() -> [u8; 96]`). |
| "RFC 9381 says the proof is 80 bytes" | **True, but it does not apply here.** 80 bytes is the RFC 9381 Edwards25519 proof length, i.e. a *different proof format* (challenge truncated to 16 bytes) that Kovanica does not implement. |
| "We implement ECVRF-ED25519-SHA512-Elligator2 per RFC 9381" | **False.** The crate is a bespoke Ristretto255 + SHA3 construction. It is **not** any RFC 9381 suite. |

**Verdict: the code is right, the label was wrong.** The 96-byte length stays.
The 80-byte figure must **not** be used to "fix" the tree — doing so would be a
wire-format break in two consensus-relevant places (block statements and the
light-witness transaction body).

## 1. What the code actually uses

`crates/kovanica-crypto/src/vrf.rs` calls the `ecvrf` crate (dependency
`ecvrf = "0.4"`, resolved to **0.4.3** in `Cargo.lock:1213-1217`).

Vendored source (read, not remembered):
`~/.cargo/registry/src/<registry-hash>/ecvrf-0.4.3/src/lib.rs`
(observed on this machine:
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ecvrf-0.4.3/src/lib.rs`).

The 0.4.3 crate is **not generic** — there is no `VrfSuite` / `EcVrf<…>` type
parameter. It exposes one concrete construction. The crate's own manifest
describes it as *"A curve25519+SHA3 verifiable random function"*
(`ecvrf-0.4.3/Cargo.toml:18`) — it never claims RFC 9381 compliance:

| Property | Value in `ecvrf` 0.4.3 | Source (vendored `lib.rs`) |
|---|---|---|
| Group | ristretto255 (prime order; cofactor handled by the encoding) | `use curve25519_dalek::ristretto::{RistrettoPoint, CompressedRistretto}` (L5) |
| Hash-to-curve | `RistrettoPoint::hash_from_bytes::<Sha3_512>` → **SHA3-512** | L133, L157 |
| Challenge `c` | **full 32 bytes** (SHA3-256, *not* truncated to `cLen = 16`) | L136-147 (`SHA3` is aliased to `Sha3_256`, L2) |
| Response `s` | 32-byte scalar, `s = k − c·sk` | L149 |
| Output point | `Gamma = h · sk` (compressed Ristretto, 32 B) | L134 |
| Nonce `k` | random `Scalar::random(&mut OsRng)` → **non-deterministic proofs** | L135 |
| Output hash `beta` | `SHA3-256(compress(Gamma))` → **32 bytes** | L7-16, L150 |
| Suite domain separation | **none** (no `suite_string`) | L132-152 |
| Proof encoding | `Gamma (32) ‖ c (32) ‖ s (32)` = **96 bytes** | L79-90 (`to_bytes`), L91-106 (`from_bytes`) |

Consequences that matter for this item:

* The proof length is **not** a free parameter — `copy_from_slice(&proof.to_bytes())`
  into a `[u8; 96]` would not compile against any other length, and
  `VrfProof::from_bytes(&[u8; 96])` would not parse a different length.
* The crate's `VrfProof` has **no** `pi` / cofactor-clearing field, and none is
  needed: ristretto255 is a prime-order group, so the cofactor problem that
  forces the extra field in the Edwards25519 suites does not exist here.
* The challenge is **not** truncated to 16 bytes, which is the single biggest
  structural difference from RFC 9381 (and the reason an 80-byte proof is
  impossible without changing the crate).

## 2. Byte breakdown (what Kovanica actually puts on the wire)

```
VRF proof (96 bytes)
  ├─ Gamma          32 B   compressed Ristretto point
  ├─ c              32 B   SHA3-256 challenge (untruncated)
  └─ s              32 B   response scalar

Block statements payload (137 bytes)
  ├─ magic           9 B   b"KVNC-VRF1"
  ├─ VRF hash       32 B   beta = SHA3-256(compress(Gamma))  (deterministic)
  └─ VRF proof      96 B   Gamma ‖ c ‖ s                     (non-deterministic)

Light-witness transaction body (SubmitLightWitness)
  ├─ epoch           8 B
  ├─ VRF hash       32 B
  └─ VRF proof      96 B   ← raw proof bytes, no magic prefix
```

## 3. RFC 9381 comparison

RFC 9381 (*The Elliptic Curve VRF (ECVRF) Suite for Cryptography Verifiable
Random Functions*, IRTF, August 2023):

* **§5.1 step 8** — `pi_string = point_to_string(Gamma) || int_to_string(c, cLen) || int_to_string(s, qLen)`,
  i.e. the proof has **exactly three fields**.
* **§5.4.4** (`ECVRF Decode Proof`) — reads back only `gamma_string`,
  `c_string`, `s_string`. Proof length = `ptLen + cLen + qLen`.
* **§5.5** (`ECVRF Ciphersuites`) — for the Edwards25519 suites:
  `ptLen = fLen = 32`, `cLen = 16`, `qLen = 32`, `Hash = SHA-512` (`hLen = 64`),
  `cofactor = 8`.
  ⇒ `32 + 16 + 32 =` **80 bytes**.

So in RFC 9381:

| RFC 9381 suite | `suite_string` | Proof length |
|---|---|---|
| `ECVRF-EDWARDS25519-SHA512-TAI` (§5.5) | `0x03` | 80 B (`Gamma 32 ‖ c 16 ‖ s 32`) |
| `ECVRF-EDWARDS25519-SHA512-ELL2` (§5.5) | `0x04` | 80 B (same layout; only `encode_to_curve` differs) |

**RFC 9381 defines no 96-byte proof.** The often-quoted "Elligator2 = 96 B
(`c 16 ‖ s 32 ‖ y 32 ‖ pi 16`)" breakdown does not correspond to RFC 9381: the
challenge is 16 bytes there and the proof carries only three fields — there is
no trailing 16-byte `pi`. RFC 9381 also renamed the draft-era suite
`ECVRF-ED25519-SHA512-Elligator2` to `ECVRF-EDWARDS25519-SHA512-ELL2`.

### Deltas vs. the RFC 9381 Edwards25519 suites

| Aspect | RFC 9381 (TAI / ELL2) | Kovanica (`ecvrf` 0.4.3) |
|---|---|---|
| Group | edwards25519, cofactor 8 | ristretto255, prime order |
| Hash | SHA-512 (`hLen = 64`) | SHA3-512 (h2c) + SHA3-256 (challenge, output) |
| `cLen` | 16 (truncated) | 32 (untruncated) |
| `qLen` | 32 | 32 |
| `ptLen` | 32 | 32 |
| Proof | 80 B | 96 B |
| Output `beta` | 64 B | 32 B |
| `suite_string` separation | yes | no |
| Nonce | RFC 8032 §5.1.6 derivation | random `OsRng` (⇒ non-deterministic proof) |
| Test vectors | RFC 9381 App. B.3 / B.4 | none |

## 4. Every in-tree site that hardcodes 96 or 137

### 137-byte `KVNC-VRF1` framing

| Site | What it does |
|---|---|
| `crates/kovanica-crypto/src/vrf.rs:145` | doc comment: magic 9 + hash 32 + proof 96 = 137 |
| `crates/kovanica-crypto/src/vrf.rs:148` | `encode_statements` — `Vec::with_capacity(137)` |
| `crates/kovanica-crypto/src/vrf.rs:149-151` | `encode_statements` — writes magic, hash, proof |
| `crates/kovanica-crypto/src/vrf.rs:157` | `decode_statements` — gate `data.len() < 137 \|\| &data[..9] != b"KVNC-VRF1"` |
| `crates/kovanica-crypto/src/vrf.rs:162-163` | `decode_statements` — `hash` from `data[9..41]`, `proof` from `data[41..137]` |
| `crates/kovanica-types/src/block.rs:78` | `compute_digest_statements` — statements enter the block digest only if `len >= 137 && &s[..9] == b"KVNC-VRF1"` |
| `crates/kovanica-dag/src/block_manager.rs:356-358` | `create_block` — gate before recording the verified VRF output |
| `crates/kovanica-dag/src/block_manager.rs:374` | `create_block` — gate before passing statements into the digest |
| `crates/kovanica-dag/src/block_manager.rs:480-490` | `validate_block_content` — `decode_statements` + `vrf_verify` on inbound blocks |
| `crates/kovanica-network/src/validation.rs:210` | `verify_block` — same 137-byte gate for digest recomputation |

### 96-byte proof

| Site | What it does |
|---|---|
| `crates/kovanica-crypto/src/vrf.rs:14-24` | `VRFOutput` doc comment — states the real layout and the RFC 9381 non-applicability |
| `crates/kovanica-crypto/src/vrf.rs:29` | `VRFOutput::proof: [u8; 96]` |
| `crates/kovanica-crypto/src/vrf.rs:34` | `VRFOutput::new(hash, proof: [u8; 96])` |
| `crates/kovanica-crypto/src/vrf.rs:44` | `VRFOutput::proof()` accessor |
| `crates/kovanica-crypto/src/vrf.rs:99-101` | `vrf_prove` — `out_proof.copy_from_slice(&proof.to_bytes())` |
| `crates/kovanica-crypto/src/vrf.rs:112` | `vrf_verify` — `VrfProof::from_bytes(&output.proof)` |
| `crates/kovanica-crypto/src/vrf.rs:161` | `decode_statements` — `let mut proof = [0u8; 96]` |
| `crates/kovanica-types/src/crypto.rs:87-89` | serde adapter doc: "96-byte ECVRF proof" |
| `crates/kovanica-types/src/crypto.rs:96-97` | `vrf_proof_bytes::serialize(&[u8; 96])` |
| `crates/kovanica-types/src/crypto.rs:104-105` | `vrf_proof_bytes::deserialize -> [u8; 96]` |
| `crates/kovanica-types/src/crypto.rs:111, 114, 121-122, 124, 133, 137` | visitor / length checks (`"96-byte ECVRF proof"`) |
| `crates/kovanica-types/src/transaction.rs:79-81` | `TransactionKind::SubmitLightWitness { vrf_proof: [u8; 96] }` + serde attr |
| `crates/kovanica-types/src/transaction.rs:201` | **transaction body encoding** — `data.extend_from_slice(vrf_proof)` |
| `crates/kovanica-staking/src/light.rs:56-57` | light-node set stores `vrf_proof: [u8; 96]` |
| `crates/kovanica-light/src/lib.rs:176` | `sign_light_witness(..., vrf_proof: [u8; 96], ...)` |
| `crates/kovanica-execution/src/lib.rs:1289` | `execute_light_witness(..., vrf_proof: [u8; 96])` |
| `crates/kovanica-execution/src/lib.rs:1305` | `VRFOutput::new(vrf_hash, vrf_proof)` → `vrf_verify` on the light-witness path |
| `crates/kovanica-rpc/src/chain_methods.rs:216, 221` | RPC passes the proof through as hex (length not re-asserted there) |

## 5. Explicit statement about the 80-byte figure

**80 bytes is not applicable to this codebase.** It is the RFC 9381
Edwards25519 proof length (`Gamma 32 ‖ c 16 ‖ s 32`) for the suites
`ECVRF-EDWARDS25519-SHA512-TAI` and `ECVRF-EDWARDS25519-SHA512-ELL2`
(RFC 9381 §5.1 step 8, §5.4.4, §5.5). Kovanica does not implement either
suite: its challenge is a full 32 bytes, its hash is SHA3-based, its group is
ristretto255, and its proof carries no suite separation. Sizing our proof at 80
bytes would require replacing the `ecvrf` dependency and changing both the
137-byte block-statements framing and the `SubmitLightWitness` transaction body
— a coordinated wire-format change, not a doc fix.

## 6. Known inaccurate statements (three fixed here, two still open)

These are documentation/label defects. They do **not** change behaviour. This
item authorised edits inside `docs/**` plus one comment in
`crates/kovanica-crypto/src/vrf.rs`; the two code-side label fixes are listed
as *open* because they were out of the authorised edit scope.

| Location | Current text | Suggested replacement |
|---|---|---|
| `crates/kovanica-crypto/src/vrf.rs:3` (module header) | `Implements ECVRF-ED25519-SHA512-Elligator2 per RFC 9381 via `ecvrf` crate.` | `Implements a Ristretto + SHA3 ECVRF via the `ecvrf` crate (NOT an RFC 9381 suite; 96-byte proof). See docs/VRF-PROOF-LENGTH.md.` — **open** (needs a one-line code edit outside this item's scope) |
| `crates/kovanica-crypto/src/vrf.rs:8` | `proof` = 96-byte VRF proof | keep — accurate; see the `VRFOutput` doc comment (added in this change) for the layout |
| `docs/VALIDATOR-ONBOARDING.md:51` | `(ECVRF-ED25519-SHA512-Elligator2 per RFC 9381)` | `(ecvrf 0.4.3, Ristretto255 + SHA3; see docs/VRF-PROOF-LENGTH.md)` — **fixed in this change** |
| `docs/AUDIT-SCOPE.md:91, 180` | `ECVRF (ecvrf crate v0.4)` / `ecvrf = "0.4.0"` | suite + 96-byte proof + `KVNC-VRF1` framing / `ecvrf = "0.4.3"` — **fixed in this change** |
| `docs/rfcs/RFC-B1-B2-VRF-AND-MIN-DIGEST.md:20` | `implements ECVRF-ED25519` | add a pointer to this document — **fixed in this change** (see the 2026-10-21 update note at the top of the draft) |

### Follow-ups for the owner (not actioned here)

1. **Suite naming.** Decide whether Kovanica wants a *named* RFC 9381 suite.
   If yes, that means replacing `ecvrf` 0.4.3 with an RFC 9381 implementation,
   re-deriving the proof layout, re-framing the 137-byte statements payload and
   the `SubmitLightWitness` body, and shipping it as a coordinated activation
   (hard-fork height). That is a consensus/wire-format change and was
   deliberately **not** done under this item.
2. **No test vectors.** The crate ships no RFC test vectors and the tree has
   none either; a deterministic proof (RFC 6979 / RFC 8032 nonce derivation)
   would also make blocks reproducible. Out of scope, flagged only.
3. **Proof malleability.** Because the nonce is random, two proofs of the same
   input differ. `beta` stays deterministic (verified by
   `crates/kovanica-crypto/src/vrf.rs` test `vrf_hash_deterministic`), so leader
   selection is unaffected.
4. **Licence of the dependency (not previously flagged).** `ecvrf` 0.4.3 is
   **GPL-3.0-or-later** (`ecvrf-0.4.3/Cargo.toml:20`), while the project root
   `LICENSE` is **Apache-2.0**. The crate is statically linked into
   `kovanica-node`/`kovanica-cli`, so this needs an owner/legal review — and an
   external auditor should be told about it. Not mentioned in
   `AUDIT_REPORT.md` / `AUTOMATED_SECURITY_CHECKS.md`; see
   `docs/AUDIT-SCOPE.md` §2.6.
5. **Pinned `curve25519-dalek` 3.2.0.** `ecvrf` 0.4.3 depends on dalek 3.2.0
   (the tree otherwise uses 4.1.3 / 5.0.0), which carries **RUSTSEC-2024-0344**
   (timing variability in `Scalar` subtraction) — and the VRF response is
   exactly `s = k − c·sk`. Already tracked in `.cargo/audit.toml`,
   `AUDIT_REPORT.md:224` and `AUTOMATED_SECURITY_CHECKS.md:20`; re-stated here
   because it sits on the leader-election critical path. The audit ignore
   comment itself already names the exit condition: "remove once the VRF moves
   to a curve25519-dalek >= 4.1.3 implementation" — i.e. item 1 above.

## 7. How to re-verify

```sh
# vendored crate source
ls ~/.cargo/registry/src/*/ecvrf-0.4.3/src/lib.rs

# proof length is enforced by the type system
grep -n "to_bytes\|from_bytes" ~/.cargo/registry/src/*/ecvrf-0.4.3/src/lib.rs

# in-tree framing
grep -rn "KVNC-VRF1\|\b137\b" crates/ --include="*.rs"

cargo check -p kovanica-crypto
cargo test  -p kovanica-crypto --lib
```

See also: `docs/rfcs/RFC-B1-B2-VRF-AND-MIN-DIGEST.md` (VRF leader election),
`docs/SIGNATURE_FORMAT.md` (sibling wire formats).
