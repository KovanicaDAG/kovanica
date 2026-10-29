//! PeerId <-> validator authority binding for Kovanica.

#![deny(unsafe_code)]

use kovanica_types::crypto::{PublicKey, Signature};
use kovanica_types::Address;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub const P2P_IDENTITY_DOMAIN: &[u8] = b"KUNA/p2p-identity/v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerIdentityClaim {
    pub peer_id: Vec<u8>,
    pub authority_pubkey: PublicKey,
    pub authority_address: Address,
    pub epoch: u64,
    pub signature: Signature,
}

impl PeerIdentityClaim {
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(P2P_IDENTITY_DOMAIN.len() + 64);
        out.extend_from_slice(P2P_IDENTITY_DOMAIN);
        out.extend_from_slice(self.peer_id.as_slice());
        out.extend_from_slice(&self.authority_pubkey.0);
        out.extend_from_slice(&self.authority_address.0);
        out.extend_from_slice(&self.epoch.to_le_bytes());
        out
    }

    pub fn verify(&self, expected_peer: &PeerId) -> Result<(), IdentityError> {
        if self.peer_id != expected_peer.to_bytes() {
            return Err(IdentityError::PeerMismatch);
        }
        kovanica_crypto::verify(
            &self.authority_pubkey,
            &self.signing_bytes(),
            &self.signature,
        )
        .map_err(|_| IdentityError::BadSignature)?;
        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum IdentityError {
    #[error("bad signature")]
    BadSignature,
    #[error("peer id mismatch")]
    PeerMismatch,
    #[error("stale epoch")]
    StaleEpoch,
    #[error("authority not in committee")]
    UnknownAuthority,
}

#[derive(Default)]
pub struct PeerIdentityRegistry {
    by_peer: HashMap<PeerId, PeerIdentityClaim>,
    by_authority: HashMap<Vec<u8>, PeerId>,
    current_epoch: u64,
    committee: Option<Vec<PublicKey>>,
    check_committee: bool,
}

impl PeerIdentityRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_epoch(&mut self, epoch: u64) {
        self.current_epoch = epoch;
    }
    pub fn set_committee(&mut self, committee: Vec<PublicKey>, check: bool) {
        self.committee = Some(committee);
        self.check_committee = check;
    }
    pub fn insert(&mut self, claim: PeerIdentityClaim, peer: PeerId) -> Result<(), IdentityError> {
        claim.verify(&peer)?;
        if claim.epoch < self.current_epoch {
            return Err(IdentityError::StaleEpoch);
        }
        if self.check_committee {
            if let Some(c) = &self.committee {
                if !c.iter().any(|k| k.0 == claim.authority_pubkey.0) {
                    return Err(IdentityError::UnknownAuthority);
                }
            }
        }
        if let Some(existing) = self.by_peer.get(&peer) {
            if existing.epoch > claim.epoch {
                return Ok(());
            }
        }
        let key = claim.authority_pubkey.0.to_vec();
        if let Some(old_peer) = self.by_authority.get(&key).copied() {
            if old_peer != peer {
                self.by_peer.remove(&old_peer);
            }
        }
        self.by_authority.insert(key, peer);
        self.by_peer.insert(peer, claim);
        Ok(())
    }
    pub fn remove_peer(&mut self, peer: &PeerId) {
        if let Some(c) = self.by_peer.remove(peer) {
            let key = c.authority_pubkey.0.to_vec();
            if self.by_authority.get(&key) == Some(peer) {
                self.by_authority.remove(&key);
            }
        }
    }
    pub fn authority_for(&self, peer: &PeerId) -> Option<&PeerIdentityClaim> {
        self.by_peer.get(peer)
    }
    pub fn verified_count_in_committee(&self, connected: &[PeerId]) -> usize {
        let Some(c) = &self.committee else {
            return 0;
        };
        connected
            .iter()
            .filter_map(|p| self.by_peer.get(p))
            .filter(|cl| c.iter().any(|k| k.0 == cl.authority_pubkey.0))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_crypto::generate_keypair;

    fn make_claim(epoch: u64) -> (PeerIdentityClaim, PeerId) {
        let peer = PeerId::random();
        let (sk, pk) = generate_keypair();
        let addr = Address([7u8; 32]);
        let mut claim = PeerIdentityClaim {
            peer_id: peer.to_bytes(),
            authority_pubkey: pk,
            authority_address: addr,
            epoch,
            signature: Signature([0u8; 64]),
        };
        claim.signature = kovanica_crypto::sign(&sk, &claim.signing_bytes());
        (claim, peer)
    }

    #[test]
    fn valid_claim_inserts() {
        let mut reg = PeerIdentityRegistry::new();
        reg.set_epoch(0);
        let (claim, peer) = make_claim(1);
        reg.insert(claim, peer).unwrap();
        assert!(reg.authority_for(&peer).is_some());
    }

    #[test]
    fn forged_signature_rejected() {
        let mut reg = PeerIdentityRegistry::new();
        reg.set_epoch(0);
        let (mut claim, peer) = make_claim(1);
        claim.signature.0[0] ^= 1;
        assert!(reg.insert(claim, peer).is_err());
    }

    #[test]
    fn peer_mismatch_rejected() {
        let mut reg = PeerIdentityRegistry::new();
        reg.set_epoch(0);
        let (claim, _peer) = make_claim(1);
        let other = PeerId::random();
        assert!(reg.insert(claim, other).is_err());
    }

    #[test]
    fn stale_epoch_rejected() {
        let mut reg = PeerIdentityRegistry::new();
        reg.set_epoch(5);
        let (claim, peer) = make_claim(3);
        assert_eq!(reg.insert(claim, peer), Err(IdentityError::StaleEpoch));
    }

    #[test]
    fn unknown_authority_rejected_when_check_enabled() {
        let mut reg = PeerIdentityRegistry::new();
        reg.set_epoch(0);
        reg.set_committee(vec![], true);
        let (claim, peer) = make_claim(1);
        assert_eq!(
            reg.insert(claim, peer),
            Err(IdentityError::UnknownAuthority)
        );
    }

    #[test]
    fn authority_accepted_when_in_committee() {
        let mut reg = PeerIdentityRegistry::new();
        reg.set_epoch(0);
        let (sk, pk) = generate_keypair();
        let addr = Address([7u8; 32]);
        let peer = PeerId::random();
        let mut claim = PeerIdentityClaim {
            peer_id: peer.to_bytes(),
            authority_pubkey: pk,
            authority_address: addr,
            epoch: 1,
            signature: Signature([0u8; 64]),
        };
        claim.signature = kovanica_crypto::sign(&sk, &claim.signing_bytes());
        reg.set_committee(vec![pk], true);
        reg.insert(claim, peer).unwrap();
        assert!(reg.authority_for(&peer).is_some());
    }

    #[test]
    fn remove_peer_clears() {
        let mut reg = PeerIdentityRegistry::new();
        reg.set_epoch(0);
        let (claim, peer) = make_claim(1);
        reg.insert(claim, peer).unwrap();
        reg.remove_peer(&peer);
        assert!(reg.authority_for(&peer).is_none());
    }

    #[test]
    fn verified_count_in_committee_counts_only_verified() {
        let mut reg = PeerIdentityRegistry::new();
        reg.set_epoch(0);
        let (sk_a, pk_a) = generate_keypair();
        let (sk_b, pk_b) = generate_keypair();
        let addr = Address([7u8; 32]);
        let mut claim_a = PeerIdentityClaim {
            peer_id: PeerId::random().to_bytes(),
            authority_pubkey: pk_a,
            authority_address: addr,
            epoch: 1,
            signature: Signature([0u8; 64]),
        };
        let peer_a = PeerId::from_bytes(&claim_a.peer_id).expect("decodes");
        claim_a.signature = kovanica_crypto::sign(&sk_a, &claim_a.signing_bytes());
        let mut claim_b = PeerIdentityClaim {
            peer_id: PeerId::random().to_bytes(),
            authority_pubkey: pk_b,
            authority_address: addr,
            epoch: 1,
            signature: Signature([0u8; 64]),
        };
        let peer_b = PeerId::from_bytes(&claim_b.peer_id).expect("decodes");
        claim_b.signature = kovanica_crypto::sign(&sk_b, &claim_b.signing_bytes());
        reg.insert(claim_a, peer_a).unwrap();
        reg.insert(claim_b, peer_b).unwrap();
        reg.set_committee(vec![pk_a, pk_b], false);
        assert_eq!(reg.verified_count_in_committee(&[peer_a, peer_b]), 2);
        assert_eq!(
            reg.verified_count_in_committee(&[peer_a, peer_b, PeerId::random()]),
            2
        );
    }
}
