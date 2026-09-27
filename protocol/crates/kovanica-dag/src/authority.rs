//! Proof-of-Authority (PoA) authority set — KVP-201, RFC-POA §1.
//!
//! A fixed set of Ed25519 authorities produces blocks in slot round-robin
//! order. The set is represented on-chain as a single live **Authority UTXO**
//! (tag `KVA1` || `authority_set_hash`); updates are threshold-signed
//! [`AuthorityUpdateTx`]s that spend the current Authority UTXO and create the
//! next one.
//!
//! ## Consensus invariants (RFC-POA §1)
//!
//! - `MIN_AUTHORITIES <= len <= MAX_AUTHORITIES` (3–4 keys at launch, ≤ 16)
//! - `MIN_THRESHOLD <= threshold <= len` (2 ≤ t ≤ n)
//! - keys are distinct
//! - the set's identity is `hash()` — a BLAKE3 digest of the canonical
//!   encoding (threshold, count, keys) — which is what the on-chain `KVA1`
//!   Authority UTXO commits to
//! - keys are held in **canonical order** (ascending 32-byte encoding), so
//!   both `hash()` and the round-robin below are invariant to the order an
//!   operator listed the keys in local config
//! - the authority scheduled for a slot is `authorities[slot % len]`
//!   (deterministic, no tie-break needed)
//!
//! ## Update rule
//!
//! An [`AuthorityUpdateTx`] is valid only when it carries **≥ t distinct**
//! valid Ed25519 signatures from members of the *current* set over the
//! canonical update payload (`old_set_hash || new_set`). This is the
//! governance path: no single authority can change the set alone.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use std::fmt;

/// Minimum number of authorities in a set (RFC-POA §1: 3–4 keys at launch).
pub const MIN_AUTHORITIES: usize = 3;
/// Maximum number of authorities in a set (RFC-POA §1: n ≤ 16).
pub const MAX_AUTHORITIES: usize = 16;
/// Minimum update threshold (RFC-POA §1: 2 ≤ t ≤ n).
pub const MIN_THRESHOLD: usize = 2;
/// Default slot duration in milliseconds (RFC-POA §3; configurable via
/// `KOVANICA_SLOT_DURATION` at M3).
pub const SLOT_DURATION_MS: u64 = 3000;
/// On-chain Authority UTXO tag (RFC-POA §1): `KVA1` || authority_set_hash.
pub const AUTHORITY_UTXO_TAG: &[u8; 4] = b"KVA1";

/// Upper bound on the length of the precomputed SW-PoA slot schedule.
///
/// The schedule is one table entry per slot in a full weighting period, so a
/// pathological weight vector (large co-prime stakes such as `[65537, 65539]`,
/// whose smallest period is 131_076) would otherwise let an operator — or an
/// attacker minting a `KVA1` Authority UTXO — force every node to allocate an
/// arbitrarily large table and to grow it on every authority-set update.
///
/// Sets whose gcd-normalised period exceeds this are **rejected at
/// construction** rather than truncated: a silently shortened period would
/// quietly change the slot→authority mapping, which is a consensus rule.
pub const MAX_SCHEDULE_PERIOD: u64 = 65_536;

/// An Ed25519 authority public key (same type as address keys).
pub type AuthorityPublicKey = VerifyingKey;

/// Errors from authority-set construction, signature verification, and
/// authority-set updates.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AuthorityError {
    #[error(
        "authority set must have between {MIN_AUTHORITIES} and {MAX_AUTHORITIES} keys (got {0})"
    )]
    InvalidAuthorityCount(usize),
    #[error(
        "threshold must be between {MIN_THRESHOLD} and the number of authorities (got {0} of {1})"
    )]
    InvalidThreshold(usize, usize),
    #[error("authority set contains duplicate keys")]
    DuplicateAuthority,
    #[error("authority signature verification failed")]
    InvalidSignature,
    #[error("signature list contains a key not in the authority set")]
    UnknownSigner,
    #[error("signature list contains duplicate signers")]
    DuplicateSigner,
    #[error("not enough distinct signatures (need {0}, got {1})")]
    InsufficientSignatures(usize, usize),
    #[error("authority update references an unknown authority set hash")]
    UnknownAuthoritySet,
    #[error("authority set bytes are malformed")]
    MalformedEncoding,
    #[error("PoA admission is not enabled on this node")]
    PoANotEnabled,
    #[error("total stake overflows u64")]
    StakeOverflow,
    #[error(
        "stake weights imply a scheduling period of {period} slots, above the \
         MAX_SCHEDULE_PERIOD of {max}; express stake in units that share a \
         common factor"
    )]
    SchedulePeriodTooLarge { period: u64, max: u64 },
}

/// Build the precomputed SW-PoA slot schedule for `stakes`.
///
/// Returns one authority index per slot of a full weighting period, laid out
/// by **smooth weighted round-robin** (SWRR). SWRR is used rather than a
/// cumulative-interval walk because it satisfies both properties a consensus
/// scheduler needs at once:
///
/// 1. **Exact proportionality.** Over one period of `sum(weights)` slots,
///    authority `i` is selected exactly `weights[i]` times. This is the
///    defining property of SWRR: each step adds `w[i]` to every score, hands
///    the slot to the highest score, and charges that winner the full period
///    `total`. An authority therefore wins again only after its score has had
///    to climb back past its rivals' — which is what makes the count over a
///    full period land exactly on its weight.
/// 2. **Near-uniform gaps.** The gap between two consecutive selections of
///    `i` stays within 2 slots of the ideal `total / w[i]`, so no authority is
///    starved for long and none is over-served. A cumulative-interval walk
///    instead gives a 70%-stake authority 70 *consecutive* slots, which is a
///    liveness hazard: that one authority being down would stall the chain for
///    70 slots even though the others were healthy.
///
/// Stakes are gcd-normalised first so the table is as short as the *ratios*
/// allow. Stakes are usually quoted in KVNC atoms, so `[70·10⁸, 20·10⁸,
/// 10·10⁸]` normalises to `[7, 2, 1]` and a 10-slot table instead of a
/// 100_000_000-slot one.
///
/// `cur` is `i64` rather than `u64` on purpose: a score is charged `total`
/// when its authority wins, so scores run negative and an unsigned type would
/// underflow.
fn build_schedule(stakes: &[u64]) -> Result<Vec<u16>, AuthorityError> {
    let n = stakes.len();
    if n == 0 {
        return Ok(Vec::new());
    }
    debug_assert!(
        n <= usize::from(u16::MAX),
        "authority count is bounded by MAX_AUTHORITIES"
    );

    // Reject a stake sum that cannot be represented, rather than letting a
    // later `.sum()` wrap or panic in debug builds.
    let _total = stakes
        .iter()
        .try_fold(0u64, |acc, &s| acc.checked_add(s))
        .ok_or(AuthorityError::StakeOverflow)?;

    // Minimal integer weights: divide out the common factor so the period is
    // as short as the stake *ratios* permit.
    let gcd = stakes
        .iter()
        .fold(0u64, |acc, &s| gcd_u64(acc, s));
    if gcd == 0 {
        return Err(AuthorityError::StakeOverflow);
    }
    let weights: Vec<u64> = stakes.iter().map(|&s| s / gcd).collect();
    let period: u64 = weights.iter().sum();
    if period > MAX_SCHEDULE_PERIOD {
        return Err(AuthorityError::SchedulePeriodTooLarge {
            period,
            max: MAX_SCHEDULE_PERIOD,
        });
    }

    let total = i64::try_from(period).map_err(|_| AuthorityError::StakeOverflow)?;
    let mut cur = vec![0i64; n];
    let mut schedule = Vec::with_capacity(period as usize);
    for _ in 0..period {
        for (slot_score, &w) in cur.iter_mut().zip(&weights) {
            *slot_score += w as i64;
        }
        // `max_by_key` returns the first maximum, so equal scores resolve to
        // the lowest index — deterministic on every node.
        let winner = cur
            .iter()
            .enumerate()
            .max_by_key(|&(_, &score)| score)
            .map(|(i, _)| i)
            .expect("authority set is non-empty");
        cur[winner] -= total;
        schedule.push(winner as u16);
    }
    Ok(schedule)
}

/// Greatest common divisor of two `u64`s, Euclid's algorithm.
fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// A fixed set of Ed25519 authorities with a threshold for updates.
///
/// Constructed only through [`AuthoritySet::new`], which enforces the
/// consensus invariants above. The set's identity is [`AuthoritySet::hash`],
/// a BLAKE3 digest of the canonical encoding — deterministic across nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthoritySet {
    authorities: Vec<AuthorityPublicKey>,
    threshold: usize,
    hash: [u8; 32],
    /// Stake weights for SW-PoA (optional, None = classic PoA equal weight)
    /// Each entry corresponds to the authority at the same index in `authorities`.
    stakes: Option<Vec<u64>>,
    /// Precomputed SW-PoA slot schedule, one authority index per slot of a
    /// full weighting period. `None` for classic PoA.
    ///
    /// Derived purely from `stakes`, so it never affects
    /// [`AuthoritySet::hash`]; it is a cache of the slot→authority rule, not
    /// part of the set's identity.
    schedule: Option<Vec<u16>>,
}

impl AuthoritySet {
    /// Build a set from `authorities` and `threshold`, enforcing the
    /// consensus invariants (count bounds, threshold bounds, distinct keys).
    ///
    /// The keys are **canonically ordered** (ascending by their 32-byte
    /// encoding) before the set is built, so the set's identity
    /// ([`AuthoritySet::hash`]) and its slot round-robin
    /// ([`AuthoritySet::active_authority`]) depend only on *which* keys are in
    /// the set — never on the order an operator happened to list them in.
    ///
    /// This matters for consensus safety: the key list arrives from local
    /// configuration (`KOVANICA_AUTHORITIES`, a comma-separated string), not
    /// from the chain. Without a canonical order two nodes holding the same
    /// keys but listing them in a different order would
    ///
    /// 1. compute different [`AuthoritySet::hash`] values → commit different
    ///    `KVA1` Authority UTXOs → build different genesis blocks and
    ///    therefore be on **different chains**, and
    /// 2. schedule a *different* authority for the same slot → reject each
    ///    other's perfectly valid blocks as
    ///    `DagError::InvalidAuthoritySignature`, halting the chain.
    ///
    /// Sorting is stable in the consensus sense: once duplicates are rejected
    /// the keys are distinct, so the total order over 32-byte encodings is
    /// total and every node agrees. It also subsumes the distinctness check,
    /// which becomes a neighbour comparison on the sorted vector.
    pub fn new(
        authorities: Vec<AuthorityPublicKey>,
        threshold: usize,
    ) -> Result<Self, AuthorityError> {
        Self::new_with_stakes(authorities, threshold, None)
    }

    /// Build a set with optional stake weights for SW-PoA.
    ///
    /// If `stakes` is provided, it must have the same length as `authorities`
    /// and contain non-zero values. The stakes are used for weighted slot
    /// assignment in SW-PoA mode. If `None`, classic PoA equal-weight round-robin
    /// is used.
    pub fn new_with_stakes(
        authorities: Vec<AuthorityPublicKey>,
        threshold: usize,
        stakes: Option<Vec<u64>>,
    ) -> Result<Self, AuthorityError> {
        let n = authorities.len();
        if !(MIN_AUTHORITIES..=MAX_AUTHORITIES).contains(&n) {
            return Err(AuthorityError::InvalidAuthorityCount(n));
        }
        if !(MIN_THRESHOLD..=n).contains(&threshold) {
            return Err(AuthorityError::InvalidThreshold(threshold, n));
        }
        // Validate stakes if provided
        if let Some(ref stakes) = stakes {
            if stakes.len() != n {
                return Err(AuthorityError::InvalidAuthorityCount(n));
            }
            if stakes.contains(&0) {
                return Err(AuthorityError::InvalidAuthorityCount(n)); // Reuse for zero stake
            }
        }
        // Canonical order: ascending 32-byte encoding.
        // We need to sort authorities and stakes together
        let mut pairs: Vec<(AuthorityPublicKey, Option<u64>)> = if let Some(stakes) = stakes {
            authorities.into_iter().zip(stakes.into_iter().map(Some)).collect()
        } else {
            authorities.into_iter().map(|pk| (pk, None)).collect()
        };
        pairs.sort_unstable_by_key(|(pk, _)| pk.to_bytes());
        // Distinct keys: equal encodings are now necessarily neighbours.
        if pairs.windows(2).any(|w| w[0].0 == w[1].0) {
            return Err(AuthorityError::DuplicateAuthority);
        }
        let (authorities, stakes): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
        let stakes_opt = if stakes.iter().all(|s| s.is_some()) {
            Some(stakes.into_iter().map(|s| s.unwrap()).collect())
        } else {
            None
        };
        let hash = blake3::hash(&Self::canonical_bytes_of(&authorities, threshold, stakes_opt.as_deref()));
        let schedule = match stakes_opt.as_deref() {
            Some(stakes) => Some(build_schedule(stakes)?),
            None => None,
        };
        Ok(Self {
            authorities,
            threshold,
            hash: *hash.as_bytes(),
            stakes: stakes_opt,
            schedule,
        })
    }

    /// The authority public keys, in **canonical order** (ascending 32-byte
    /// encoding) — this is the slot round-robin order. Independent of the
    /// order the keys were supplied in.
    pub fn authorities(&self) -> &[AuthorityPublicKey] {
        &self.authorities
    }

    /// The number of authorities in the set.
    pub fn len(&self) -> usize {
        self.authorities.len()
    }

    /// Whether the set is empty (never true for a valid set).
    pub fn is_empty(&self) -> bool {
        self.authorities.is_empty()
    }

    /// The update threshold `t` (2 ≤ t ≤ n).
    pub fn threshold(&self) -> usize {
        self.threshold
    }

    /// Get stake weights for SW-PoA (None = classic PoA equal weight).
    pub fn stakes(&self) -> Option<&[u64]> {
        self.stakes.as_deref()
    }

    /// Total stake across all authorities.
    ///
    /// Saturating rather than `sum()`: construction rejects a stake vector
    /// whose total overflows ([`AuthorityError::StakeOverflow`]), so this can
    /// only saturate on a set built before that check existed, and a saturated
    /// total is still the right answer for a share denominator.
    pub fn total_stake(&self) -> u64 {
        self.stakes.as_ref().map_or(self.authorities.len() as u64, |s| {
            s.iter().fold(0u64, |acc, &v| acc.saturating_add(v))
        })
    }

    /// Length in slots of one full SW-PoA weighting period, or `None` for
    /// classic PoA.
    ///
    /// The schedule repeats every `period` slots, so `period` is also the
    /// granularity at which a stake rebalance takes effect: with a 10-slot
    /// period a weight change moves the next slot's owner.
    pub fn schedule_period(&self) -> Option<u64> {
        self.schedule.as_ref().map(|s| s.len() as u64)
    }

    /// The stake of `pk`, or `None` for classic PoA / unknown key.
    pub fn stake_of(&self, pk: &AuthorityPublicKey) -> Option<u64> {
        let idx = self.authorities.iter().position(|k| k == pk)?;
        self.stakes.as_ref().map(|s| s[idx])
    }

    /// The set's identity: BLAKE3 of the canonical encoding. This is what the
    /// on-chain `KVA1` Authority UTXO commits to.
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }

    /// The authority scheduled to produce the block for `slot`.
    ///
    /// - Classic PoA (no stakes): `authorities[slot % len]` — deterministic round-robin.
    /// - SW-PoA (with stakes): the precomputed smooth-weighted schedule,
    ///   `authorities[schedule[slot % period]]`, which is proportional to
    ///   stake over each full period and never holds one authority for two
    ///   consecutive slots.
    pub fn active_authority(&self, slot: u64) -> &AuthorityPublicKey {
        if let Some(schedule) = &self.schedule {
            &self.authorities[usize::from(schedule[slot as usize % schedule.len()])]
        } else {
            // Classic PoA: simple round-robin
            &self.authorities[slot as usize % self.authorities.len()]
        }
    }

    /// Verify a 64-byte Ed25519 `sig` over `message` against the authority
    /// scheduled for `slot`.
    pub fn verify_slot_signature(
        &self,
        slot: u64,
        message: &[u8],
        sig: &[u8; 64],
    ) -> Result<(), AuthorityError> {
        let pk = self.active_authority(slot);
        let signature = Signature::from_bytes(sig);
        pk.verify(message, &signature)
            .map_err(|_| AuthorityError::InvalidSignature)
    }

    /// Verify a threshold of distinct authority signatures over `message`.
    ///
    /// `sigs` is a list of `(public_key, signature)` pairs. Validation:
    /// 1. every signer is a member of this set (no unknown keys),
    /// 2. no duplicate signers,
    /// 3. at least `threshold` signatures verify over `message`.
    pub fn verify_threshold_signatures(
        &self,
        message: &[u8],
        sigs: &[(AuthorityPublicKey, [u8; 64])],
    ) -> Result<(), AuthorityError> {
        if sigs.len() < self.threshold {
            return Err(AuthorityError::InsufficientSignatures(
                self.threshold,
                sigs.len(),
            ));
        }
        let mut seen = std::collections::HashSet::new();
        let mut valid = 0usize;
        for (pk, sig) in sigs {
            if !self.authorities.contains(pk) {
                return Err(AuthorityError::UnknownSigner);
            }
            if !seen.insert(pk.to_bytes()) {
                return Err(AuthorityError::DuplicateSigner);
            }
            let signature = Signature::from_bytes(sig);
            if pk.verify(message, &signature).is_ok() {
                valid += 1;
            }
        }
        if valid < self.threshold {
            return Err(AuthorityError::InsufficientSignatures(
                self.threshold,
                valid,
            ));
        }
        Ok(())
    }

    /// Canonical byte encoding: `threshold (u64 LE) || count (u64 LE) ||
    /// pk_1 (32) || … || pk_n (32) || [stake_1 (u64 LE) || … || stake_n (u64 LE)]`.
    /// The hash is BLAKE3 of exactly this.
    pub fn to_bytes(&self) -> Vec<u8> {
        Self::canonical_bytes_of(&self.authorities, self.threshold, self.stakes.as_deref())
    }

    /// Decode a set from its canonical byte encoding.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AuthorityError> {
        if bytes.len() < 16 {
            return Err(AuthorityError::MalformedEncoding);
        }
        let threshold = u64::from_le_bytes(bytes[..8].try_into().unwrap()) as usize;
        let count = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
        let expected_len = 16 + count * 32;
        let has_stakes = bytes.len() > expected_len;
        if has_stakes && bytes.len() != expected_len + count * 8 {
            return Err(AuthorityError::MalformedEncoding);
        }
        if !has_stakes && bytes.len() != expected_len {
            return Err(AuthorityError::MalformedEncoding);
        }
        let mut authorities = Vec::with_capacity(count);
        for i in 0..count {
            let pk_bytes: [u8; 32] = bytes[16 + i * 32..16 + (i + 1) * 32]
                .try_into()
                .map_err(|_| AuthorityError::MalformedEncoding)?;
            let pk = VerifyingKey::from_bytes(&pk_bytes)
                .map_err(|_| AuthorityError::MalformedEncoding)?;
            authorities.push(pk);
        }
        let stakes = if has_stakes {
            let mut stakes = Vec::with_capacity(count);
            let stake_start = expected_len;
            for i in 0..count {
                let stake = u64::from_le_bytes(
                    bytes[stake_start + i * 8..stake_start + (i + 1) * 8]
                        .try_into()
                        .map_err(|_| AuthorityError::MalformedEncoding)?
                );
                stakes.push(stake);
            }
            Some(stakes)
        } else {
            None
        };
        Self::new_with_stakes(authorities, threshold, stakes)
    }

    fn canonical_bytes_of(
        authorities: &[AuthorityPublicKey],
        threshold: usize,
        stakes: Option<&[u64]>,
    ) -> Vec<u8> {
        let mut buf = Vec::with_capacity(16 + 32 * authorities.len() + stakes.map(|s| s.len() * 8).unwrap_or(0));
        buf.extend_from_slice(&(threshold as u64).to_le_bytes());
        buf.extend_from_slice(&(authorities.len() as u64).to_le_bytes());
        for pk in authorities {
            buf.extend_from_slice(pk.as_bytes());
        }
        if let Some(stakes) = stakes {
            for stake in stakes {
                buf.extend_from_slice(&stake.to_le_bytes());
            }
        }
        buf
    }
    
    /// Build a merkle tree of (pubkey -> stake) for SPV stake proofs.
    /// Returns the merkle root (32 bytes).
    pub fn stake_merkle_root(&self) -> Option<[u8; 32]> {
        if let Some(stakes) = &self.stakes {
            let leaves: Vec<[u8; 32]> = self
                .authorities
                .iter()
                .zip(stakes.iter())
                .map(|(pk, stake)| {
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(pk.as_bytes());
                    hasher.update(&stake.to_le_bytes());
                    *hasher.finalize().as_bytes()
                })
                .collect();
            Some(Self::merkle_root(&leaves))
        } else {
            None
        }
    }
    
    /// Generate a merkle proof for a specific authority's stake.
    /// Returns None if no stakes or authority not found.
    pub fn stake_merkle_proof(&self, authority_pubkey: &AuthorityPublicKey) -> Option<StakeMerkleProof> {
        let stakes = self.stakes.as_ref()?;
        let idx = self.authorities.iter().position(|pk| pk == authority_pubkey)?;
        
        let leaves: Vec<[u8; 32]> = self
            .authorities
            .iter()
            .zip(stakes.iter())
            .map(|(pk, stake)| {
                let mut hasher = blake3::Hasher::new();
                hasher.update(pk.as_bytes());
                hasher.update(&stake.to_le_bytes());
                *hasher.finalize().as_bytes()
            })
            .collect();
        
        let path = Self::merkle_path(&leaves, idx);
        Some(StakeMerkleProof {
            leaf: StakeLeaf {
                authority_pubkey: authority_pubkey.to_bytes(),
                stake: stakes[idx],
                vault_id: 0, // TODO: link to KVP-105 vault
            },
            path,
            index: idx,
        })
    }
    
    /// Compute merkle root from leaves.
    fn merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
        if leaves.is_empty() {
            return [0u8; 32];
        }
        let mut current = leaves.to_vec();
        while current.len() > 1 {
            let mut next = Vec::with_capacity(current.len().div_ceil(2));
            for i in (0..current.len()).step_by(2) {
                let left = current[i];
                let right = if i + 1 < current.len() { current[i + 1] } else { left };
                let mut hasher = blake3::Hasher::new();
                hasher.update(&left);
                hasher.update(&right);
                next.push(*hasher.finalize().as_bytes());
            }
            current = next;
        }
        current[0]
    }
    
    /// Compute merkle path for a leaf at index.
    fn merkle_path(leaves: &[[u8; 32]], index: usize) -> Vec<[u8; 32]> {
        let mut path = Vec::new();
        let mut current = leaves.to_vec();
        let mut idx = index;
        while current.len() > 1 {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            let sibling = if sibling_idx < current.len() {
                current[sibling_idx]
            } else {
                current[idx] // last odd leaf paired with itself
            };
            path.push(sibling);
            let mut next = Vec::with_capacity(current.len().div_ceil(2));
            for i in (0..current.len()).step_by(2) {
                let left = current[i];
                let right = if i + 1 < current.len() { current[i + 1] } else { left };
                let mut hasher = blake3::Hasher::new();
                hasher.update(&left);
                hasher.update(&right);
                next.push(*hasher.finalize().as_bytes());
            }
            current = next;
            idx /= 2;
        }
        path
    }
}

impl fmt::Display for AuthoritySet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AuthoritySet({} keys, t={}, hash={}…)",
            self.len(),
            self.threshold,
            hex::encode(&self.hash[..8])
        )
    }
}

/// Merkle proof for an authority's stake weight (SW-PoA SPV).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StakeMerkleProof {
    pub leaf: StakeLeaf,
    pub path: Vec<[u8; 32]>,
    pub index: usize,
}

/// A leaf in the stake merkle tree.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StakeLeaf {
    pub authority_pubkey: [u8; 32],
    pub stake: u64,
    pub vault_id: u64, // KVP-105 vault ID for slashing
}

impl StakeMerkleProof {
    /// Verify the merkle proof against a given root.
    pub fn verify(&self, root: [u8; 32]) -> bool {
        let mut hash = {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&self.leaf.authority_pubkey);
            hasher.update(&self.leaf.stake.to_le_bytes());
            *hasher.finalize().as_bytes()
        };
        for sibling in &self.path {
            let (left, right) = if self.index % 2 == 0 { (hash, *sibling) } else { (*sibling, hash) };
            let mut hasher = blake3::Hasher::new();
            hasher.update(&left);
            hasher.update(&right);
            hash = *hasher.finalize().as_bytes();
        }
        hash == root
    }
}

/// The canonical message authorities sign for an update:
/// `old_set_hash (32) || new_set canonical bytes`.
pub fn update_payload(old_set_hash: &[u8; 32], new_set: &AuthoritySet) -> Vec<u8> {
    let mut buf = Vec::with_capacity(32 + new_set.to_bytes().len());
    buf.extend_from_slice(old_set_hash);
    buf.extend_from_slice(&new_set.to_bytes());
    buf
}

/// A threshold-signed authority set update (RFC-POA §1).
///
/// Spends the current `KVA1` Authority UTXO (identified by `old_set_hash`)
/// and creates the next one (`new_set`). Valid only when it carries
/// **≥ t distinct** valid Ed25519 signatures from members of the *current*
/// set over [`update_payload`] — see [`AuthorityUpdateTx::validate`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorityUpdateTx {
    old_set_hash: [u8; 32],
    new_set: AuthoritySet,
    signatures: Vec<(AuthorityPublicKey, [u8; 64])>,
}

impl AuthorityUpdateTx {
    /// Build an update tx. Structural validation only (new set is already
    /// valid by construction; signers must be distinct); the full consensus
    /// check — membership in the old set, count ≥ old threshold, signature
    /// validity — happens in [`AuthorityUpdateTx::validate`].
    pub fn new(
        old_set_hash: [u8; 32],
        new_set: AuthoritySet,
        signatures: Vec<(AuthorityPublicKey, [u8; 64])>,
    ) -> Result<Self, AuthorityError> {
        let mut seen = std::collections::HashSet::new();
        for (pk, _) in &signatures {
            if !seen.insert(pk.to_bytes()) {
                return Err(AuthorityError::DuplicateSigner);
            }
        }
        Ok(Self {
            old_set_hash,
            new_set,
            signatures,
        })
    }

    /// Full consensus validation against the current authority set:
    /// 1. `old_set.hash()` must match this tx's `old_set_hash`,
    /// 2. every signer must be a member of `old_set` (distinct),
    /// 3. at least `old_set.threshold()` signatures must verify over the
    ///    canonical update payload.
    pub fn validate(&self, old_set: &AuthoritySet) -> Result<(), AuthorityError> {
        if old_set.hash() != self.old_set_hash {
            return Err(AuthorityError::UnknownAuthoritySet);
        }
        let payload = update_payload(&self.old_set_hash, &self.new_set);
        old_set.verify_threshold_signatures(&payload, &self.signatures)
    }

    /// The hash of the authority set this update spends.
    pub fn old_set_hash(&self) -> &[u8; 32] {
        &self.old_set_hash
    }

    /// The new authority set this update creates.
    pub fn new_set(&self) -> &AuthoritySet {
        &self.new_set
    }

    /// The `(public_key, signature)` pairs carried by the update.
    pub fn signatures(&self) -> &[(AuthorityPublicKey, [u8; 64])] {
        &self.signatures
    }
}

/// Sign an authority-set update with one authority's signing key.
/// Returns the 64-byte Ed25519 signature over [`update_payload`].
pub fn sign_update(sk: &SigningKey, old_set_hash: &[u8; 32], new_set: &AuthoritySet) -> [u8; 64] {
    let payload = update_payload(old_set_hash, new_set);
    sk.sign(&payload).to_bytes()
}

/// On-chain tag for an authority-set update transaction (RFC-POA §1).
pub const AUTHORITY_UPDATE_TAG: &[u8; 4] = b"KVA2";

impl AuthorityUpdateTx {
    /// Canonical encoding: `old_set_hash (32) || new_set.to_bytes() || sig_count (u8) || (pk 32 || sig 64) * count`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf =
            Vec::with_capacity(32 + self.new_set.to_bytes().len() + 1 + self.signatures.len() * 96);
        buf.extend_from_slice(&self.old_set_hash);
        buf.extend_from_slice(&self.new_set.to_bytes());
        buf.push(self.signatures.len() as u8);
        for (pk, sig) in &self.signatures {
            buf.extend_from_slice(pk.as_bytes());
            buf.extend_from_slice(sig);
        }
        buf
    }

    /// Decode from the canonical encoding.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AuthorityError> {
        if bytes.len() < 33 {
            return Err(AuthorityError::MalformedEncoding);
        }
        let old_set_hash: [u8; 32] = bytes[..32]
            .try_into()
            .map_err(|_| AuthorityError::MalformedEncoding)?;
        // Parse new_set: first 16 bytes are threshold (u64) + count (u64), then count * 32 bytes of keys.
        if bytes.len() < 32 + 16 {
            return Err(AuthorityError::MalformedEncoding);
        }
        let count = u64::from_le_bytes(
            bytes[40..48]
                .try_into()
                .map_err(|_| AuthorityError::MalformedEncoding)?,
        ) as usize;
        let new_set_len = 16 + count * 32;
        if bytes.len() < 32 + new_set_len + 1 {
            return Err(AuthorityError::MalformedEncoding);
        }
        let new_set = AuthoritySet::from_bytes(&bytes[32..32 + new_set_len])?;
        let sig_start = 32 + new_set_len;
        let sig_count = bytes[sig_start] as usize;
        let expected_len = sig_start + 1 + sig_count * 96;
        if bytes.len() != expected_len {
            return Err(AuthorityError::MalformedEncoding);
        }
        let mut signatures = Vec::with_capacity(sig_count);
        let mut off = sig_start + 1;
        for _ in 0..sig_count {
            let pk_bytes: [u8; 32] = bytes[off..off + 32]
                .try_into()
                .map_err(|_| AuthorityError::MalformedEncoding)?;
            let pk = VerifyingKey::from_bytes(&pk_bytes)
                .map_err(|_| AuthorityError::MalformedEncoding)?;
            let sig: [u8; 64] = bytes[off + 32..off + 96]
                .try_into()
                .map_err(|_| AuthorityError::MalformedEncoding)?;
            signatures.push((pk, sig));
            off += 96;
        }
        Ok(Self {
            old_set_hash,
            new_set,
            signatures,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keypair(seed: u8) -> (SigningKey, VerifyingKey) {
        let sk = SigningKey::from_bytes(&[seed; 32]);
        let pk = sk.verifying_key();
        (sk, pk)
    }

    fn three_authorities() -> (Vec<VerifyingKey>, Vec<SigningKey>) {
        let (sk1, pk1) = keypair(1);
        let (sk2, pk2) = keypair(2);
        let (sk3, pk3) = keypair(3);
        (vec![pk1, pk2, pk3], vec![sk1, sk2, sk3])
    }

    /// A *different* 3-key set (seeds 11, 12, 13) — for update tests that
    /// must distinguish old/new/other sets.
    fn other_three_authorities() -> (Vec<VerifyingKey>, Vec<SigningKey>) {
        let (sk1, pk1) = keypair(11);
        let (sk2, pk2) = keypair(12);
        let (sk3, pk3) = keypair(13);
        (vec![pk1, pk2, pk3], vec![sk1, sk2, sk3])
    }

    /// Re-order `sks` to match a set's canonical authority order, so a test
    /// can sign as `sks[slot % n]` and be the scheduled authority for
    /// `active_authority(slot)` — mirroring what a real authority node does
    /// when it looks its key up by public key (`Node::try_produce_poa`).
    fn sks_canonical(set: &AuthoritySet, mut sks: Vec<SigningKey>) -> Vec<SigningKey> {
        sks.sort_unstable_by_key(|sk| sk.verifying_key().to_bytes());
        assert_eq!(sks.len(), set.len());
        for (i, sk) in sks.iter().enumerate() {
            assert_eq!(sk.verifying_key(), set.authorities()[i]);
        }
        sks
    }

    // ------------------------------------------------------------------
    // SW-PoA slot scheduling
    // ------------------------------------------------------------------

    /// A stake-weighted set over the three test authorities.
    fn staked_set(stakes: [u64; 3]) -> AuthoritySet {
        let (pks, _) = three_authorities();
        AuthoritySet::new_with_stakes(pks, 2, Some(stakes.to_vec()))
            .expect("stake set should be valid")
    }

    /// The slot counts a set must produce over one period.
    ///
    /// `new_with_stakes` binds each stake to the key it was supplied with and
    /// then sorts keys and stakes **together** into canonical order, so the
    /// stake at canonical index `i` is a permutation of the input vector.
    /// Expectations are therefore derived from [`AuthoritySet::stakes`], which
    /// is already in canonical order, rather than from the input literal.
    fn expected_counts(set: &AuthoritySet) -> Vec<usize> {
        let stakes = set.stakes().expect("staked set");
        let gcd = stakes.iter().fold(0u64, |acc, &s| gcd_u64(acc, s));
        stakes.iter().map(|&s| (s / gcd) as usize).collect()
    }

    /// Count how many of the first `slots` slots each authority index owns.
    fn slot_counts(set: &AuthoritySet, slots: u64) -> Vec<usize> {
        let mut counts = vec![0usize; set.len()];
        for slot in 0..slots {
            let owner = set.active_authority(slot);
            let idx = set.authorities().iter().position(|k| k == owner).unwrap();
            counts[idx] += 1;
        }
        counts
    }

    /// Regression test for the defect where `active_authority` computed
    /// `slot * total_stake % total_stake`, which is `0` for every slot, so the
    /// first authority took *every* slot and the other N-1 were permanently
    /// starved (their blocks failed `verify_slot_signature`, so at threshold
    /// 2-of-3 the chain could not progress at all).
    #[test]
    fn sw_poa_every_authority_gets_slots() {
        let set = staked_set([70, 20, 10]);
        let counts = slot_counts(&set, 30);
        assert!(
            counts.iter().all(|&c| c > 0),
            "every authority must be scheduled, got {counts:?}"
        );
    }

    /// The core SW-PoA invariant: over one full period an authority owns
    /// exactly as many slots as its weight, so slot share == stake share.
    #[test]
    fn sw_poa_schedule_is_exactly_proportional() {
        let set = staked_set([70, 20, 10]);
        let expected = expected_counts(&set);
        let period = set.schedule_period().expect("staked set has a period");
        // 70/20/10 shares a factor of 10, so the period is 7 + 2 + 1.
        let mut sorted = expected.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![1, 2, 7], "weights should normalise to 7/2/1");
        assert_eq!(period, 10);
        assert_eq!(slot_counts(&set, period), expected);
    }

    /// Proportionality must survive stake vectors that are not already
    /// normalised, including atom-denominated stakes (1 KVNC = 10^8 atoms).
    #[test]
    fn sw_poa_schedule_normalises_stake_units() {
        let atoms = 100_000_000u64;
        let set = staked_set([70 * atoms, 20 * atoms, 10 * atoms]);
        assert_eq!(
            set.schedule_period(),
            Some(10),
            "a common stake unit must not lengthen the schedule"
        );
        assert_eq!(slot_counts(&set, 10), expected_counts(&set));
    }

    /// Smooth weighted round-robin keeps the wait between two selections of
    /// the same authority within 2 slots of the ideal `period / weight`. That
    /// bound is the fault-tolerance property: no authority is idle for long,
    /// so the chain keeps making progress even if the heaviest one goes down.
    ///
    /// (The bound is deliberately *not* "never two consecutive slots" — a
    /// 70%-weight authority in a 10-slot period owns 7 of them, so adjacency
    /// is unavoidable. A cumulative-interval walk would instead put all 7 in
    /// one contiguous run.)
    #[test]
    fn sw_poa_schedule_keeps_gaps_near_uniform() {
        let set = staked_set([70, 20, 10]);
        let period = set.schedule_period().unwrap();
        let expected = expected_counts(&set);
        // Two full periods so each authority's wrap-around gap is covered.
        let horizon = period * 2;
        for (idx, &weight) in expected.iter().enumerate() {
            let weight = weight as u64;
            let slots: Vec<u64> = (0..horizon)
                .filter(|&slot| set.active_authority(slot) == &set.authorities()[idx])
                .collect();
            assert_eq!(
                slots.len(),
                weight as usize * 2,
                "authority {idx} owned {} slots in {horizon}, want {}",
                slots.len(),
                weight as usize * 2
            );
            for pair in slots.windows(2) {
                let gap = pair[1] - pair[0];
                // `gap` is within 2 of the ideal period/weight, written
                // without floats so the bound is exact.
                assert!(
                    weight * gap <= period + 2 * weight && weight * gap + 2 * weight >= period,
                    "authority {idx} waited {gap} slots; ideal is {}/{weight}",
                    period
                );
            }
        }
    }

    /// The schedule is a pure function of the stake vector, so a set that
    /// arrives from chain data (`from_bytes`) schedules exactly like one built
    /// from local config. If these diverged, nodes would disagree about who
    /// owns a slot and reject each other's blocks.
    #[test]
    fn sw_poa_schedule_survives_serialisation_round_trip() {
        let set = staked_set([70, 20, 10]);
        let decoded = AuthoritySet::from_bytes(&set.to_bytes()).expect("round trip");
        assert_eq!(set.schedule_period(), decoded.schedule_period());
        for slot in 0..50u64 {
            assert_eq!(
                set.active_authority(slot),
                decoded.active_authority(slot),
                "slot {slot} scheduled differently after a round trip"
            );
        }
    }

    /// The schedule repeats with its period, so a rebalance only takes effect
    /// on a period boundary and the mapping stays bounded.
    #[test]
    fn sw_poa_schedule_repeats_every_period() {
        let set = staked_set([70, 20, 10]);
        let period = set.schedule_period().unwrap();
        for slot in 0..30u64 {
            assert_eq!(
                set.active_authority(slot),
                set.active_authority(slot + period),
                "slot {slot} does not repeat at +{period}"
            );
        }
    }

    /// The schedule is a **cache of the slot rule, not part of the set's
    /// identity**. It is not serialised, so tuning it (as this fix did) can
    /// never change the `KVA1` Authority UTXO a chain has already committed to
    /// and can never force a re-commitment. The stake vector, by contrast, *is*
    /// consensus data and does belong in the hash.
    #[test]
    fn sw_poa_schedule_is_not_part_of_the_serialised_identity() {
        let (pks, _) = three_authorities();
        let a = AuthoritySet::new_with_stakes(pks.clone(), 2, Some(vec![70, 20, 10])).unwrap();
        let b = AuthoritySet::new_with_stakes(pks.clone(), 2, Some(vec![70, 20, 10])).unwrap();
        assert_eq!(a.hash(), b.hash(), "identical sets share a hash");
        // Stakes are consensus data and are committed to.
        let c = AuthoritySet::new_with_stakes(pks, 2, Some(vec![70, 20, 11])).unwrap();
        assert_ne!(a.hash(), c.hash(), "stake changes must change the hash");
        // The schedule is not serialised: the encoding is exactly
        // threshold||count||keys||stakes.
        let expected_len = 16 + a.len() * 32 + a.len() * 8;
        assert_eq!(a.to_bytes().len(), expected_len);
    }

    /// Classic PoA keeps plain round-robin and gains no schedule, so its
    /// behaviour is byte-identical to before SW-PoA existed.
    #[test]
    fn classic_poa_keeps_plain_round_robin() {
        let set = AuthoritySet::new(three_authorities().0, 2).unwrap();
        assert_eq!(set.schedule_period(), None, "classic PoA has no schedule");
        for slot in 0..30u64 {
            assert_eq!(
                set.active_authority(slot),
                &set.authorities()[slot as usize % set.len()]
            );
        }
    }

    /// A weight vector with a huge minimal period (co-prime large stakes) must
    /// be rejected at construction rather than forcing an unbounded table.
    /// Truncating instead would silently change the slot->authority mapping,
    /// which is itself a consensus rule.
    #[test]
    fn sw_poa_rejects_an_oversized_schedule_period() {
        let (pks, _) = three_authorities();
        let err = AuthoritySet::new_with_stakes(pks, 2, Some(vec![65_537, 65_539, 65_533]))
            .expect_err("period above MAX_SCHEDULE_PERIOD must be rejected");
        assert!(
            matches!(err, AuthorityError::SchedulePeriodTooLarge { .. }),
            "got {err:?}"
        );
    }

    /// A small legal set still schedules, so the guard rejects only genuinely
    /// pathological ratios.
    #[test]
    fn sw_poa_accepts_an_ordinary_weight_vector() {
        let (pks, _) = three_authorities();
        // gcd(1, 1, 2) == 1, so the period is 1 + 1 + 2 == 4.
        let set = AuthoritySet::new_with_stakes(pks, 2, Some(vec![1, 1, 2])).unwrap();
        assert_eq!(set.schedule_period(), Some(4));
        assert_eq!(slot_counts(&set, 4), expected_counts(&set));
    }

    /// A stake sum that overflows `u64` is rejected at construction instead of
    /// wrapping (which would corrupt every share) or panicking in debug.
    #[test]
    fn sw_poa_rejects_a_stake_sum_overflow() {
        let (pks, _) = three_authorities();
        let err = AuthoritySet::new_with_stakes(pks, 2, Some(vec![u64::MAX, 1, 1]))
            .expect_err("overflowing stake sum must be rejected");
        assert!(matches!(err, AuthorityError::StakeOverflow), "got {err:?}");
    }

    /// A dominant authority is a valid ratio: it takes almost every slot, but
    /// the small ones still get theirs.
    #[test]
    fn sw_poa_handles_a_dominant_authority() {
        let set = staked_set([1000, 1, 1]);
        let mut expected = expected_counts(&set);
        expected.sort_unstable();
        assert_eq!(expected, vec![1, 1, 1000]);
        assert_eq!(slot_counts(&set, 1002), expected_counts(&set));
    }

    /// The scheduled owner must be able to verify its own block signature,
    /// otherwise the node cannot produce and peers reject the block. This is
    /// the end-to-end shape of the original defect.
    #[test]
    fn sw_poa_scheduled_owner_can_sign_its_slot() {
        let (_, sks) = three_authorities();
        let set = staked_set([70, 20, 10]);
        let sks = sks_canonical(&set, sks);
        for slot in 0..30u64 {
            let owner = set.active_authority(slot);
            let idx = set.authorities().iter().position(|k| k == owner).unwrap();
            let message = slot.to_le_bytes();
            let sig = sks[idx].sign(&message).to_bytes();
            assert!(
                set.verify_slot_signature(slot, &message, &sig).is_ok(),
                "the authority scheduled for slot {slot} could not verify its own signature"
            );
        }
    }

    // ------------------------------------------------------------------
    // Construction invariants
    // ------------------------------------------------------------------

    #[test]
    fn authority_set_rejects_too_few_keys() {
        let (_, pk1) = keypair(1);
        let (_, pk2) = keypair(2);
        let err = AuthoritySet::new(vec![pk1, pk2], 2).unwrap_err();
        assert_eq!(err, AuthorityError::InvalidAuthorityCount(2));
    }

    #[test]
    fn authority_set_rejects_too_many_keys() {
        let keys: Vec<_> = (1..=17).map(|i| keypair(i).1).collect();
        let err = AuthoritySet::new(keys, 2).unwrap_err();
        assert_eq!(err, AuthorityError::InvalidAuthorityCount(17));
    }

    #[test]
    fn authority_set_rejects_threshold_below_min() {
        let (keys, _) = three_authorities();
        let err = AuthoritySet::new(keys, 1).unwrap_err();
        assert_eq!(err, AuthorityError::InvalidThreshold(1, 3));
    }

    #[test]
    fn authority_set_rejects_threshold_above_count() {
        let (keys, _) = three_authorities();
        let err = AuthoritySet::new(keys, 4).unwrap_err();
        assert_eq!(err, AuthorityError::InvalidThreshold(4, 3));
    }

    #[test]
    fn authority_set_rejects_duplicate_keys() {
        let (_, pk1) = keypair(1);
        let (_, pk2) = keypair(2);
        let err = AuthoritySet::new(vec![pk1, pk2, pk1], 2).unwrap_err();
        assert_eq!(err, AuthorityError::DuplicateAuthority);
    }

    #[test]
    fn authority_set_accepts_valid_3_of_3() {
        let (keys, _) = three_authorities();
        let set = AuthoritySet::new(keys, 3).unwrap();
        assert_eq!(set.len(), 3);
        assert_eq!(set.threshold(), 3);
    }

    #[test]
    fn authority_set_accepts_valid_4_of_2() {
        let keys: Vec<_> = (1..=4).map(|i| keypair(i).1).collect();
        let set = AuthoritySet::new(keys, 2).unwrap();
        assert_eq!(set.len(), 4);
        assert_eq!(set.threshold(), 2);
    }

    // ------------------------------------------------------------------
    // Hash determinism
    // ------------------------------------------------------------------

    #[test]
    fn authority_set_hash_is_deterministic() {
        let (keys, _) = three_authorities();
        let a = AuthoritySet::new(keys.clone(), 2).unwrap();
        let b = AuthoritySet::new(keys, 2).unwrap();
        assert_eq!(a.hash(), b.hash());
    }

    #[test]
    fn authority_set_hash_differs_for_different_sets() {
        let (keys, _) = three_authorities();
        let a = AuthoritySet::new(keys.clone(), 2).unwrap();
        let b = AuthoritySet::new(keys, 3).unwrap();
        assert_ne!(a.hash(), b.hash());

        // A genuinely different membership must hash differently.
        let (keys2, _) = other_three_authorities();
        let c = AuthoritySet::new(keys2, 2).unwrap();
        assert_ne!(a.hash(), c.hash());
    }

    #[test]
    fn authority_set_hash_is_permutation_invariant() {
        // The key list comes from local config (`KOVANICA_AUTHORITIES`), not
        // from the chain, so the set identity — and therefore the on-chain
        // `KVA1` commitment and the genesis block id — must not depend on the
        // order an operator listed the keys in.
        let (keys, _) = three_authorities();
        let mut swapped = keys.clone();
        swapped.swap(0, 1);
        let a = AuthoritySet::new(keys, 2).unwrap();
        let b = AuthoritySet::new(swapped, 2).unwrap();
        assert_eq!(a.hash(), b.hash());
        assert_eq!(a, b);
        assert_eq!(a.to_bytes(), b.to_bytes());
    }

    #[test]
    fn active_authority_is_permutation_invariant() {
        // Two nodes with the same keys listed in a different order must
        // schedule the *same* authority for every slot, or they reject each
        // other's valid blocks (`InvalidAuthoritySignature`).
        let (keys, _) = three_authorities();
        let mut swapped = keys.clone();
        swapped.swap(0, 1);
        let a = AuthoritySet::new(keys, 2).unwrap();
        let b = AuthoritySet::new(swapped, 2).unwrap();
        for slot in 0..9 {
            assert_eq!(a.active_authority(slot), b.active_authority(slot));
        }
    }

    // ------------------------------------------------------------------
    // Slot round-robin
    // ------------------------------------------------------------------

    #[test]
    fn active_authority_round_robins() {
        let (keys, _) = three_authorities();
        let set = AuthoritySet::new(keys, 2).unwrap();
        // Round-robin over the set's *canonical* order.
        for slot in 0..9 {
            let expected = &set.authorities()[slot as usize % 3];
            assert_eq!(set.active_authority(slot), expected);
        }
    }

    // ------------------------------------------------------------------
    // Slot signature verification
    // ------------------------------------------------------------------

    #[test]
    fn verify_slot_signature_accepts_scheduled_authority() {
        let (keys, sks) = three_authorities();
        let set = AuthoritySet::new(keys, 2).unwrap();
        let sks = sks_canonical(&set, sks);
        let message = b"block hash without authority sig";
        // Slot 0 → authority 0, slot 1 → authority 1, slot 2 → authority 2.
        for (slot, sk) in sks.iter().enumerate() {
            let sig = sk.sign(message).to_bytes();
            set.verify_slot_signature(slot as u64, message, &sig)
                .expect("scheduled authority must verify");
        }
    }

    #[test]
    fn verify_slot_signature_rejects_wrong_authority() {
        let (keys, sks) = three_authorities();
        let set = AuthoritySet::new(keys, 2).unwrap();
        let sks = sks_canonical(&set, sks);
        let message = b"block hash";
        // Slot 0 is authority 0's slot; any other authority's signature must
        // fail, whichever key the canonical order puts in slot 1 or 2.
        for sk in sks.iter().skip(1) {
            let sig = sk.sign(message).to_bytes();
            assert_eq!(
                set.verify_slot_signature(0, message, &sig),
                Err(AuthorityError::InvalidSignature)
            );
        }
    }

    #[test]
    fn verify_slot_signature_rejects_tampered_message() {
        let (keys, sks) = three_authorities();
        let set = AuthoritySet::new(keys, 2).unwrap();
        let sks = sks_canonical(&set, sks);
        let sig = sks[0].sign(b"original").to_bytes();
        assert_eq!(
            set.verify_slot_signature(0, b"tampered", &sig),
            Err(AuthorityError::InvalidSignature)
        );
    }

    // ------------------------------------------------------------------
    // Canonical encoding roundtrip
    // ------------------------------------------------------------------

    #[test]
    fn authority_set_roundtrip() {
        let (keys, _) = three_authorities();
        let set = AuthoritySet::new(keys, 2).unwrap();
        let bytes = set.to_bytes();
        let decoded = AuthoritySet::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, set);
        assert_eq!(decoded.hash(), set.hash());
    }

    #[test]
    fn authority_set_from_bytes_rejects_malformed() {
        assert_eq!(
            AuthoritySet::from_bytes(&[0u8; 8]),
            Err(AuthorityError::MalformedEncoding)
        );
        // Truncated key list.
        let (keys, _) = three_authorities();
        let set = AuthoritySet::new(keys, 2).unwrap();
        let bytes = set.to_bytes();
        assert_eq!(
            AuthoritySet::from_bytes(&bytes[..bytes.len() - 1]),
            Err(AuthorityError::MalformedEncoding)
        );
        // Invalid Ed25519 point. About half of all 32-byte values fail to
        // decompress, so find the first candidate the point decoder actually
        // rejects instead of hardcoding a magic constant that could rot with
        // an ed25519-dalek bump. The scan is deterministic, and we do not
        // assume *where* the key lands in the canonical order.
        let mut bad_point = None;
        for i in 0u16..=u16::MAX {
            let mut candidate = [0u8; 32];
            candidate[0] = i as u8;
            candidate[1] = (i >> 8) as u8;
            candidate[31] = 0x40;
            if VerifyingKey::from_bytes(&candidate).is_err() {
                bad_point = Some(candidate);
                break;
            }
        }
        let bad_point = bad_point.expect("a non-decompressible point must exist");
        let mut bad = bytes.clone();
        bad[16..48].copy_from_slice(&bad_point);
        assert_eq!(
            AuthoritySet::from_bytes(&bad),
            Err(AuthorityError::MalformedEncoding)
        );
    }

    // ------------------------------------------------------------------
    // AuthorityUpdateTx
    // ------------------------------------------------------------------

    #[test]
    fn update_tx_validates_with_threshold() {
        let (keys, sks) = three_authorities();
        let old_set = AuthoritySet::new(keys, 2).unwrap();
        let (new_keys, _) = three_authorities();
        let new_set = AuthoritySet::new(new_keys, 2).unwrap();

        // 2-of-3 signatures from the current set.
        let sigs: Vec<_> = sks[..2]
            .iter()
            .map(|sk| {
                (
                    sk.verifying_key(),
                    sign_update(sk, &old_set.hash(), &new_set),
                )
            })
            .collect();
        let tx = AuthorityUpdateTx::new(old_set.hash(), new_set, sigs).unwrap();
        tx.validate(&old_set).expect("2-of-3 update must validate");
    }

    #[test]
    fn update_tx_rejects_insufficient_signatures() {
        let (keys, sks) = three_authorities();
        let old_set = AuthoritySet::new(keys, 2).unwrap();
        let (new_keys, _) = three_authorities();
        let new_set = AuthoritySet::new(new_keys, 2).unwrap();

        // Only 1-of-3.
        let sigs = vec![(
            sks[0].verifying_key(),
            sign_update(&sks[0], &old_set.hash(), &new_set),
        )];
        let tx = AuthorityUpdateTx::new(old_set.hash(), new_set, sigs).unwrap();
        assert_eq!(
            tx.validate(&old_set),
            Err(AuthorityError::InsufficientSignatures(2, 1))
        );
    }

    #[test]
    fn update_tx_rejects_unknown_signer() {
        let (keys, sks) = three_authorities();
        let old_set = AuthoritySet::new(keys, 2).unwrap();
        let (new_keys, _) = three_authorities();
        let new_set = AuthoritySet::new(new_keys, 2).unwrap();

        // One valid member + one outsider.
        let (sk_out, pk_out) = keypair(99);
        let sigs = vec![
            (
                sks[0].verifying_key(),
                sign_update(&sks[0], &old_set.hash(), &new_set),
            ),
            (pk_out, sign_update(&sk_out, &old_set.hash(), &new_set)),
        ];
        let tx = AuthorityUpdateTx::new(old_set.hash(), new_set, sigs).unwrap();
        assert_eq!(tx.validate(&old_set), Err(AuthorityError::UnknownSigner));
    }

    #[test]
    fn update_tx_rejects_duplicate_signers() {
        let (keys, sks) = three_authorities();
        let old_set = AuthoritySet::new(keys, 2).unwrap();
        let (new_keys, _) = three_authorities();
        let new_set = AuthoritySet::new(new_keys, 2).unwrap();

        // Same signer listed twice (even with a valid signature each).
        let sigs = vec![
            (
                sks[0].verifying_key(),
                sign_update(&sks[0], &old_set.hash(), &new_set),
            ),
            (
                sks[0].verifying_key(),
                sign_update(&sks[0], &old_set.hash(), &new_set),
            ),
        ];
        assert_eq!(
            AuthorityUpdateTx::new(old_set.hash(), new_set, sigs),
            Err(AuthorityError::DuplicateSigner)
        );
    }

    #[test]
    fn update_tx_rejects_wrong_old_set() {
        let (keys, sks) = three_authorities();
        let old_set = AuthoritySet::new(keys, 2).unwrap();
        let (other_keys, _) = other_three_authorities();
        let other_set = AuthoritySet::new(other_keys, 2).unwrap();
        let (new_keys, _) = other_three_authorities();
        let new_set = AuthoritySet::new(new_keys, 2).unwrap();

        // Signatures over the *real* old set, but the tx claims a different
        // old_set_hash (the other set's).
        let sigs: Vec<_> = sks[..2]
            .iter()
            .map(|sk| {
                (
                    sk.verifying_key(),
                    sign_update(sk, &old_set.hash(), &new_set),
                )
            })
            .collect();
        let tx = AuthorityUpdateTx::new(other_set.hash(), new_set, sigs).unwrap();
        assert_eq!(
            tx.validate(&old_set),
            Err(AuthorityError::UnknownAuthoritySet)
        );
    }

    #[test]
    fn update_tx_rejects_tampered_new_set() {
        let (keys, sks) = three_authorities();
        let old_set = AuthoritySet::new(keys, 2).unwrap();
        let (new_keys, _) = other_three_authorities();
        let new_set = AuthoritySet::new(new_keys, 2).unwrap();

        // Signatures over the *intended* new set…
        let sigs: Vec<_> = sks[..2]
            .iter()
            .map(|sk| {
                (
                    sk.verifying_key(),
                    sign_update(sk, &old_set.hash(), &new_set),
                )
            })
            .collect();
        // …but the tx carries a *different* new set (tampered in transit).
        let (tampered_keys, _) = three_authorities();
        let tampered = AuthoritySet::new(tampered_keys, 2).unwrap();
        let tx = AuthorityUpdateTx::new(old_set.hash(), tampered, sigs).unwrap();
        assert_eq!(
            tx.validate(&old_set),
            Err(AuthorityError::InsufficientSignatures(2, 0))
        );
    }

    #[test]
    fn update_tx_roundtrip_encoding() {
        let (keys, sks) = three_authorities();
        let old_set = AuthoritySet::new(keys, 2).unwrap();
        let (new_keys, _) = other_three_authorities();
        let new_set = AuthoritySet::new(new_keys, 2).unwrap();

        let sigs: Vec<_> = sks[..2]
            .iter()
            .map(|sk| {
                (
                    sk.verifying_key(),
                    sign_update(sk, &old_set.hash(), &new_set),
                )
            })
            .collect();
        let tx = AuthorityUpdateTx::new(old_set.hash(), new_set.clone(), sigs).unwrap();

        let bytes = tx.to_bytes();
        let decoded = AuthorityUpdateTx::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, tx);
        assert_eq!(decoded.old_set_hash(), tx.old_set_hash());
        assert_eq!(decoded.new_set(), tx.new_set());
        assert_eq!(decoded.signatures(), tx.signatures());
    }

    #[test]
    fn update_tx_from_bytes_rejects_malformed() {
        let (keys, sks) = three_authorities();
        let old_set = AuthoritySet::new(keys, 2).unwrap();
        let (new_keys, _) = other_three_authorities();
        let new_set = AuthoritySet::new(new_keys, 2).unwrap();
        let sigs: Vec<_> = sks[..2]
            .iter()
            .map(|sk| {
                (
                    sk.verifying_key(),
                    sign_update(sk, &old_set.hash(), &new_set),
                )
            })
            .collect();
        let tx = AuthorityUpdateTx::new(old_set.hash(), new_set.clone(), sigs).unwrap();
        let bytes = tx.to_bytes();
        let _new_set_bytes_len = new_set.to_bytes().len();

        // Truncated
        let mut truncated = bytes.clone();
        truncated.pop();
        assert_eq!(
            AuthorityUpdateTx::from_bytes(&truncated),
            Err(AuthorityError::MalformedEncoding)
        );
        // Wrong sig count (corrupt the sig_count byte at sig_start)
        let mut bad = bytes.clone();
        let _new_set_bytes_len = new_set.to_bytes().len();
        let count = u64::from_le_bytes(bytes[40..48].try_into().unwrap()) as usize;
        let new_set_len = 16 + count * 32;
        let sig_start = 32 + new_set_len;
        bad[sig_start] = 99; // corrupt sig_count
        assert_eq!(
            AuthorityUpdateTx::from_bytes(&bad),
            Err(AuthorityError::MalformedEncoding)
        );
    }
}
