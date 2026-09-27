// ============================================================
// SW-PoA SPV/FFI Changes — Draft
// ============================================================
// File: protocol/crates/kovanica-dag/src/block.rs
// ============================================================

use serde::{Deserialize, Serialize};
use kovanica_state::BlockId;

/// SW-PoA block header (extends existing PoA header)
/// Total overhead: ~80 bytes vs current PoA header
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwPoAHeader {
    // --- Existing PoA fields ---
    pub version: u16,
    pub parents: Vec<BlockId>,
    pub timestamp_ms: u64,
    pub payload_hash: [u8; 32],
    pub authority_sig: [u8; 64],        // Ed25519 signature
    pub authority_pubkey: [u8; 32],     // Signer's pubkey
    
    // --- NEW SW-PoA fields ---
    /// Merkle root of (authority_pubkey -> stake) map
    /// Allows SPV verification of stake weights
    pub authority_stake_root: [u8; 32],
    
    /// Total stake across all authorities (atoms)
    /// Used for weighted slot selection
    pub total_stake: u64,
    
    /// Slot number (timestamp_ms / SLOT_DURATION_MS)
    /// Derived but included for SPV convenience
    pub slot: u64,
    
    /// Epoch number for stake root updates
    /// Stake root only changes at epoch boundaries
    pub epoch: u64,
}

impl SwPoAHeader {
    /// Size on wire: ~280 bytes (vs ~200 for current PoA)
    pub const SERIALIZED_SIZE: usize = 280;
    
    /// Verify this header's stake proof against the committed stake root
    pub fn verify_stake_proof(&self, proof: &StakeMerkleProof) -> bool {
        proof.verify(self.authority_stake_root)
    }
    
    /// Get the expected authority for this slot given stake weights
    pub fn expected_authority(&self, authorities: &[(AuthorityPublicKey, u64)]) -> AuthorityPublicKey {
        let total: u64 = authorities.iter().map(|(_, s)| *s).sum();
        let target = (self.slot * total) % total;
        let mut acc = 0;
        for (pk, stake) in authorities {
            acc += stake;
            if target < acc { return *pk; }
        }
        authorities.last().unwrap().0
    }
}

/// Merkle proof for authority stake weight
/// Size: ~32 * log2(N) bytes (N=validators, typically 7-100)
/// For N=7: ~96 bytes; N=100: ~224 bytes
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StakeMerkleProof {
    pub leaf: StakeLeaf,
    pub path: Vec<[u8; 32]>,  // Merkle path to root
    pub index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StakeLeaf {
    pub authority_pubkey: [u8; 32],
    pub stake: u64,
    pub vault_id: u64,  // KVP-105 vault ID for slashing
}

impl StakeMerkleProof {
    pub fn verify(&self, root: [u8; 32]) -> bool {
        let mut hash = blake3::hash(&self.leaf.to_bytes()).as_bytes().clone();
        for (i, sibling) in self.path.iter().enumerate() {
            let (left, right) = if (self.index >> i) & 1 == 0 {
                (hash, *sibling)
            } else {
                (*sibling, hash)
            };
            hash = blake3::hash(&[&left, &right].concat()).as_bytes().clone();
        }
        hash == root
    }
}

// ============================================================
// File: protocol/crates/kovanica-state/src/spv.rs
// ============================================================

/// Extended SPV verification for SW-PoA
impl SpvClient {
    /// Verify SW-PoA block header with stake proof
    pub fn verify_sw_poa_header(
        &self,
        header: &SwPoAHeader,
        stake_proof: &StakeMerkleProof,
    ) -> Result<(), SpvError> {
        // 1. Verify stake proof matches header's stake root
        if !stake_proof.verify(header.authority_stake_root) {
            return Err(SpvError::InvalidStakeProof);
        }
        
        // 2. Extract authority set from proof (single authority for this slot)
        let authority = stake_proof.leaf;
        
        // 3. Verify slot assignment matches stake weight
        // Note: For full verification, need all authorities' stakes
        // This is simplified - real impl needs full stake set or epoch summary
        let expected = self.compute_expected_authority(header.slot, &authority.stake, header.total_stake)?;
        if authority.authority_pubkey != expected {
            return Err(SpvError::InvalidSlotAuthority);
        }
        
        // 4. Verify authority signature
        if !verify_ed25519(
            &authority.authority_pubkey,
            &header.hash_without_sig(),
            &header.authority_sig,
        ) {
            return Err(SpvError::InvalidSignature);
        }
        
        Ok(())
    }
    
    /// Fetch stake proof for a given slot from full node
    pub async fn fetch_stake_proof(&self, slot: u64) -> Result<StakeMerkleProof, SpvError> {
        // RPC call to full node: getstakeproof <slot>
        self.rpc.get_stake_proof(slot).await
    }
    
    /// Compute expected authority for slot (simplified - needs full set)
    fn compute_expected_authority(&self, slot: u64, stake: u64, total_stake: u64) -> Result<[u8; 32], SpvError> {
        // In practice, light client needs the full authority set for this epoch
        // This can be cached from epoch boundary block
        todo!("Requires epoch authority set cache")
    }
}

// ============================================================
// File: protocol/crates/kovanica-ffi/src/light_node.rs
// ============================================================

/// FFI-exposed SW-PoA verification
#[no_mangle]
pub extern "C" fn kovanica_light_verify_sw_poa_header(
    header_ptr: *const u8,
    header_len: usize,
    proof_ptr: *const u8,
    proof_len: usize,
    out_error: *mut u8,
) -> bool {
    let header = deserialize_sw_poa_header(header_ptr, header_len);
    let proof = deserialize_stake_proof(proof_ptr, proof_len);
    
    let client = LightNode::get_instance();
    match client.verify_sw_poa_header(&header, &proof) {
        Ok(()) => true,
        Err(e) => {
            if !out_error.is_null() {
                write_error(out_error, &e.to_string());
            }
            false
        }
    }
}

/// FFI: Fetch stake proof for slot
#[no_mangle]
pub extern "C" fn kovanica_light_fetch_stake_proof(
    slot: u64,
    out_proof: *mut u8,
    out_len: *mut usize,
) -> bool {
    let client = LightNode::get_instance();
    match client.fetch_stake_proof(slot).await {
        Ok(proof) => {
            let bytes = serialize_stake_proof(&proof);
            if bytes.len() > MAX_PROOF_SIZE { return false; }
            unsafe { out_proof.copy_from_nonoverlapping(bytes.as_ptr(), bytes.len()) };
            unsafe { *out_len = bytes.len() };
            true
        }
        Err(_) => false,
    }
}

// ============================================================
// File: protocol/crates/kovanica-node/src/rpc.rs
// ============================================================

/// New RPC methods for SW-PoA SPV
impl RpcServer {
    /// Get stake merkle proof for a slot
    /// Returns: StakeMerkleProof for the authority assigned to this slot
    pub async fn get_stake_proof(&self, slot: u64) -> Result<StakeMerkleProof, RpcError> {
        let epoch = slot / EPOCH_SLOTS;
        let epoch_block = self.find_epoch_boundary_block(epoch)?;
        let authority_set = epoch_block.authority_stake_set();
        
        let authority = authority_set.active_authority(slot);
        let proof = authority_set.merkle_proof_for(authority.pubkey);
        
        Ok(proof)
    }
    
    /// Get full authority stake set for an epoch (for light client caching)
    /// Called once per epoch (~100k slots = ~13 days at 120s)
    pub async fn get_epoch_authority_set(&self, epoch: u64) -> Result<Vec<(AuthorityPublicKey, u64)>, RpcError> {
        let epoch_block = self.find_epoch_boundary_block(epoch)?;
        Ok(epoch_block.authority_stake_set().into_iter().collect())
    }
}

// ============================================================
// FFI Bindings Regeneration
// ============================================================
// Run: cargo run --example generate_bindings
// Updates: kotlin/src/main/kotlin/com/kovanica/ffi/LightNode.kt
//          swift/Sources/KovanicaFFI/LightNode.swift

// New Kotlin methods:
// fun verifySwPoAHeader(header: ByteArray, proof: ByteArray): Boolean
// fun fetchStakeProof(slot: Long): ByteArray
// fun getEpochAuthoritySet(epoch: Long): List<AuthorityStake>

// New Swift methods:
// func verifySwPoAHeader(header: Data, proof: Data) -> Bool
// func fetchStakeProof(slot: UInt64) async throws -> Data
// func getEpochAuthoritySet(epoch: UInt64) async throws -> [AuthorityStake]