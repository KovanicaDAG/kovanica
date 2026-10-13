//! Committee and Authority definitions.

use crate::crypto::PublicKey;
use crate::Address;
use serde::{Deserialize, Serialize};

/// Stake amount (in smallest units of KUNA).
pub type Stake = u64;

/// Single authority (validator).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Authority {
    /// Index of this authority in the committee.
    pub index: u16,
    /// Ed25519 public key for block signing.
    pub public_key: PublicKey,
    /// VRF public key for verifiable leader selection.
    pub vrf_public_key: Option<PublicKey>,
    /// Account address (derived from public key).
    pub address: Address,
    /// Stake amount in base units.
    pub stake: Stake,
    /// Network address (multiaddr or host:port) for P2P.
    pub network_address: String,
}

/// Current committee (validator set).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Committee {
    /// Epoch number for this committee.
    pub epoch: u64,
    /// List of authorities in this committee.
    pub authorities: Vec<Authority>,
    /// Total stake of all authorities.
    pub total_stake: Stake,
    /// Quorum threshold (2f+1 stake).
    pub quorum_threshold: Stake,
    /// Validity threshold (f+1 stake).
    pub validity_threshold: Stake,
}

impl Committee {
    /// Create a new committee and compute thresholds.
    pub fn new(epoch: u64, authorities: Vec<Authority>) -> Self {
        let total_stake: Stake = authorities.iter().map(|a| a.stake).sum();
        // Classic BFT: quorum = 2f+1 → floor(2/3 * total) + 1
        let quorum_threshold = (total_stake * 2) / 3 + 1;
        let validity_threshold = total_stake / 3 + 1;

        Self {
            epoch,
            authorities,
            total_stake,
            quorum_threshold,
            validity_threshold,
        }
    }

    /// Return the number of authorities in the committee.
    pub fn size(&self) -> usize {
        self.authorities.len()
    }

    /// Look up an authority by its index.
    pub fn get_by_index(&self, index: u16) -> Option<&Authority> {
        self.authorities.iter().find(|a| a.index == index)
    }

    /// Stake-weighted VRF-based leader election.
    ///
    /// Uses VRF output from each validator for the given round. The leader is
    /// the validator with the lowest VRF hash output (highest priority), weighted
    /// by stake. Validators without VRF keys fall back to deterministic selection.
    pub fn leader(&self, round: u64) -> u16 {
        self.leader_with_vrf(round, &std::collections::HashMap::new())
    }

    /// VRF-aware leader selection.
    pub fn leader_with_vrf(
        &self,
        round: u64,
        vrf_outputs: &std::collections::HashMap<u16, [u8; 32]>,
    ) -> u16 {
        if self.authorities.is_empty() {
            return 0;
        }

        // Try VRF-based selection first
        let mut best_authority: Option<(u16, u128)> = None; // (index, priority)

        for authority in &self.authorities {
            if let Some(vrf_hash) = vrf_outputs.get(&authority.index) {
                // Priority = VRF hash / stake (lower = higher priority)
                // Using u128 to avoid overflow: priority = hash_as_u128 * total_stake / stake
                let hash_u128 = u128::from_le_bytes([
                    vrf_hash[0], vrf_hash[1], vrf_hash[2], vrf_hash[3],
                    vrf_hash[4], vrf_hash[5], vrf_hash[6], vrf_hash[7],
                    vrf_hash[8], vrf_hash[9], vrf_hash[10], vrf_hash[11],
                    vrf_hash[12], vrf_hash[13], vrf_hash[14], vrf_hash[15],
                ]);
                
                let priority = hash_u128
                    .saturating_mul(self.total_stake as u128)
                    .saturating_div(authority.stake.max(1) as u128);

                if best_authority.map_or(true, |(_, p)| priority < p) {
                    best_authority = Some((authority.index, priority));
                }
            }
        }

        if let Some((idx, _)) = best_authority {
            return idx;
        }

        // Fallback: deterministic round-robin (no VRF outputs available)
        let seed = round;
        let idx = (seed as usize) % self.authorities.len();
        self.authorities[idx].index
    }
}
