# Kovanica Authority Key Ceremony

**Status:** DRAFT — procedure written, **not yet performed**. Closes the *procedure*
half of `TESTNET-RESET-POLICY.md` §0.2 gate 1; it does not close the gate.
**Consensus impact:** **consensus-breaking** (see §1).
**Applies to:** `kovanica-testnet` (gate 1) and `kovanica-mainnet` (gate 4).

> **Read this first.** This document is *planning*, which §0.1 explicitly
> authorises. Performing the ceremony commits real secret key material to real
> custody. Do not start until the gate-1 preconditions in §2 are met.

---

## 1. Why an authority key ceremony is a hard gate

Under PoA, a block is admitted on the strength of an Ed25519 signature from a
key in the genesis-committed `KVA1` authority set. That makes the set the entire
admission control. There is no PoW work to burn and no stake to bond, so:

- **Possession of a key is sufficient authority.** No stake, no penalty, no
  slashing. A leaked key is a total compromise of the chain's block production.
- **The set is fixed at genesis.** The PoA genesis coinbase tag is
  `KVA1 || authority_set_hash` (RFC-POA §1). The tag is inside the coinbase
  transaction id, which is inside the genesis block payload, so the set
  determines the genesis **block id**. Changing the set later is not a config
  change — it is a different chain.

The testnet placeholder set (`AUTHORITY_PLACEHOLDER_BASE = 9001`) is derived
from a public constant in the source tree: `KeyPair::from_u64(n)` is
`seed = n_le64 || 0*24`, so anyone can regenerate all three keys and forge
authoritative blocks at will.

**This is why gate 2 (the 24h soak) cannot run first on placeholders.** A green
soak on publicly-derivable keys exercises an *unauthenticated* PoA. It cannot
detect key compromise, key reuse between operators, or a bad ceremony, because
there is no secret to compromise. The soak must run on the keys this ceremony
produces, or it proves nothing about the thing that matters.

### Consequence: the ceremony must precede the soak, and the reset must come last

```
key ceremony (this doc)  ->  PoA genesis on real keys  ->  24h soak  ->  reset sign-off
                              \___________________________________________/
                                 both need the real set committed at genesis
```

There is exactly one genesis. Once it is committed with the real set, the soak
runs against it, and `TESTNET-RESET-POLICY.md` §0.2 gates 2 and 3 close against
the same chain. Trying to change the authority set afterwards requires wiping
`KOVANICA_DATA` and resetting again.

---

## 2. Gate-1 preconditions

Do not begin until all of these are true. Each is a decision, not a task, so each
needs a named human owner.

| # | Precondition | Why |
|---|--------------|-----|
| P1 | Named key custodians identified, one per authority key, in writing | §4 has to hand out secrets to someone accountable for them |
| P2 | Custody mechanism chosen and provisioned (see §4.2) | Generation is worthless if the seeds land in a shell history |
| P3 | Testnet authority count and threshold agreed | §3.1 — this is an **open design input** per RFC-POA-Migration §0.7.2, not a default to accept |
| P4 | An isolated signing host available, offline-capable | §4.1 |
| P5 | A place to publish the public commitment | §6 |
| P6 | Two humans available for the whole ceremony | Four-eyes on every step; a solo operator cannot meaningfully attest to their own ceremony |

> **P3 is genuinely open.** RFC-POA-Migration §0.7.2 lists initial set choice,
> eligibility, threshold `t`, expansion, and dissolution as unsettled. A testnet
> ceremony can pick provisional values to unblock the soak, but the values
> chosen here become a *de facto* mainnet precedent unless §3.1 records them as
> provisional. Make that explicit.

---

## 3. Parameters

### 3.1 Set size and threshold

Consensus bounds are hard: `MIN_AUTHORITIES = 3`, `MAX_AUTHORITIES = 16`,
`MIN_THRESHOLD = 2`, `t <= n` (`kovanica-dag/src/authority.rs`). Anything outside
those is rejected at genesis construction, not at admission.

| Option | n | t | Effect |
|--------|---|---|--------|
| Single-operator | 3 | 3 | Every slot needs all three keys. No failover. Useless for a soak. |
| **Core team (suggested for testnet)** | **3** | **2** | One key can be offline; the chain still produces. Exercises real failover. |
| Broad | 5 | 3 | Tolerates two losses; more operators to recruit for the soak. |

**Suggested for testnet: n = 3, t = 2.** The minimum viable set that can still
survive a single unavailable operator — which is what the soak is supposed to
test. Record the choice as provisional pending §0.7.2.

### 3.2 Slot duration

`KOVANICA_SLOT_DURATION_MS` defaults to 3000. The slot schedule is
`authorities[slot % len]` — strict round-robin, **not** threshold-signature
based. This matters for reading §3.1 correctly:

- **`t` governs `AuthorityUpdateTx`, not block production.** One designated
  authority signs each slot. A 2-of-3 set means *two keys can authorise a set
  change*, while *each individual key still produces its own slots*.
- Consequence for the soak: an operator holding 1 of 3 keys produces ~1/3 of all
  blocks. If a key holder goes offline, the chain **stalls**, it does not
  degrade to the remaining keys. Plan the soak so at least one redundant holder
  exists per key, or accept deliberate stalls as a soak observation.
- Consequence for custody: losing one key costs that operator's slots, not the
  chain — but the chain does not *automatically* redistribute them. Recovery is
  an `AuthorityUpdateTx`, which does need `t` signatures.

### 3.3 What is and is not committed at genesis

`KOVANICA_AUTHORITIES` (64-hex public keys, comma-separated) and
`KOVANICA_AUTHORITY_THRESHOLD` determine the set. `KOVANICA_AUTHORITY_KEY` (64-hex
**seed**, this node's own signing key) is node-local and never reaches genesis.
Keys are canonically ordered by the protocol, so listing them in a different
order yields the same set and the same genesis id.

---

## 4. Procedure

### 4.0 Rule zero

**No real authority seed is ever typed into a shell, written to a repo, pasted
into a chat, or stored in an env file that outlives the ceremony.** The
`KOVANICA_AUTHORITY_KEY` env var is a *deployment* mechanism, not a storage
mechanism — it is how an operator's node gets its key in production, and it is
covered by the same rules as any other secret in the runbook.

### 4.1 Isolated signing host

An air-gapped or offline-capable machine, not the node host. `seed1` runs the
genesis node; it is a public-facing machine and is the wrong place to type a
seed. The host needs only a Rust toolchain (or the release binary) and no
network egress for the generation step.

### 4.2 Generation

There is no `authority-keygen` subcommand yet — `kovanica-cli keygen` writes a
*wallet* key file and is not the right tool. Until one exists, generate with a
short throwaway program on the isolated host, or add the subcommand first
(tracked as a follow-up; the CLI change is client-only). What matters is:

- Entropy: `KeyPair::from_seed` takes a raw 32-byte seed, so the seed must come
  from a CSPRNG (`getrandom`/`/dev/urandom` — `rand` is already a workspace
  dependency). **Never `from_u64`**: that is the placeholder derivation.
- One seed per authority, generated and handled **one at a time**. Never three
  seeds in one process's memory or one terminal session.

For each authority `i` in `0..n`, record:

```
seed_i        64 hex chars   SECRET — custody per §4.4
pubkey_i      64 hex chars   public — goes into KOVANICA_AUTHORITIES
```

`pubkey_i` is the 32-byte Ed25519 public key, hex encoded. Verify each pair
agrees before moving on: sign a fixed message with `seed_i` and check it
verifies against `pubkey_i`. A mismatched pair is unrecoverable and wastes a
ceremony slot.

### 4.3 Assembly and verification

1. Assemble `KOVANICA_AUTHORITIES` as `pubkey_0,pubkey_1,pubkey_2` and
   `KOVANICA_AUTHORITY_THRESHOLD` as the §3.1 value.
2. Confirm the set is *not* the placeholder set. This is checkable: compare
   against `KeyPair::from_u64(9001 + i).public_key()` for `i` in `0..3`. Equal
   means the ceremony produced nothing.
3. Compute the set commitment — `AuthoritySet::hash()`, hex. The node writes
   this itself to `$KOVANICA_DATA/<node>.authorities` on first boot and refuses
   to boot on a mismatch thereafter, so it doubles as a tripwire for a
   misconfigured deployment. Record the expected value now (§6).
4. Both custodians confirm the assembled set matches what they were each handed.

### 4.4 Custody

Each seed goes to exactly one named custodian. Requirements:

- Stored offline (encrypted disk, paper, hardware signer) — not on the node host.
- Never committed to git. `.env` and `.env.*` are gitignored, but that is a
  backstop, not a control: a secret in git history stays in git history.
- Backed up, with the backup tested by actually restoring and re-deriving
  `pubkey_i`. An untested backup is not a backup.
- Independently recoverable by a different person than the primary holder, so a
  single lost key does not end the chain's ability to produce.

### 4.5 Deployment to operator nodes

Only `KOVANICA_AUTHORITIES` + `KOVANICA_AUTHORITY_THRESHOLD` go on **every**
node. `KOVANICA_AUTHORITY_KEY` goes only on the node that node's key signs for,
and comes from that custodian's secret store at boot — never baked into a unit
file, an image, or a config repo. See `SEED1_POA_DEPLOYMENT.md` for the unit
layout.

---

## 5. Failure modes

| Failure | Consequence | Recovery |
|---------|-------------|----------|
| A seed leaks | Attacker can forge authoritative blocks for that key's slots | Rotate the whole set via `AuthorityUpdateTx` (needs `t` signatures), then investigate. Assume the chain is untrusted until rotated. |
| A seed is lost and has no backup | That operator's slots stall | `AuthorityUpdateTx` to re-point its slots, if a spare key exists |
| A custodian quits / machine dies | Same as lost seed | Same |
| Placeholder set committed at genesis by mistake | PoA is unauthenticated; the whole soak is worthless | Wipe `KOVANICA_DATA`, redo with real keys. This is the cheapest moment to catch it — the `.authorities` commitment makes it loud. |
| Real set committed, then `KOVANICA_AUTHORITIES` edited on one node | That node forks or refuses to boot | The `.authorities` mismatch check refuses to boot. Restore the original env. |

A leaked **mainnet** key is not recoverable by any of the above at acceptable
cost. That asymmetry is the reason this document exists.

---

## 6. Public record

Publish, and commit to the repo:

- Each `pubkey_i` (public by definition)
- The threshold and `n`
- The set commitment (`AuthoritySet::hash()` hex)
- The date and the ceremony participants
- The resulting genesis block id, once committed

**Never publish, never commit:** any `seed_i`, any `KOVANICA_AUTHORITY_KEY`
value, any `KOVANICA_TREASURY_SEED` value, or any image/snapshot containing
them.

The public keys go in the repo because a key set nobody can check is a key set
nobody can hold accountable. The seeds stay out because the repo is not a
custodian.

Record the outcome in `TESTNET-RESET-POLICY.md` §5 History, per §1.3 there —
ceremonies are never ad-hoc.

---

## 7. Definition of done

Gate 1 is closed when **all** of the following are true and recorded:

- [ ] §2 preconditions P1–P6 satisfied, each with a named owner
- [ ] §3.1 parameters chosen and recorded as provisional or final
- [ ] `n` real seeds generated on an isolated host, each pair verified
- [ ] Custody established per §4.4, with a **tested** restore for each key
- [ ] Public record published per §6
- [ ] Testnet genesis committed with the real set (this is the
      `TESTNET-RESET-POLICY.md` §0.2 gate-1 evidence, and it is the step that
      makes gates 2 and 3 meaningful)
- [ ] `TESTNET-RESET-POLICY.md` §0.2 gate 1 flipped to closed, with the genesis
      id and the set commitment recorded
- [ ] Follow-up filed for the missing `authority-keygen` CLI subcommand

Gate 4 (mainnet) additionally requires §3.1 values settled per RFC-POA-Migration
§0.7.2, a mainnet-eligibility policy, and at least one key held by a party
outside the core team.

---

## 8. Related

- `TESTNET-RESET-POLICY.md` §0.2 — the four gates this procedure serves
- `RFC-POA-Migration.md` §0.7.2 — authority-set governance (open inputs)
- `SEED1_POA_DEPLOYMENT.md` — genesis node deployment
- `docs/LEGIT-BOARD.md` — roadmap checklist
