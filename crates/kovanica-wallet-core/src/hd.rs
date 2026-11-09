#![deny(unsafe_code)]

//! Pure-Rust SLIP-0010 Ed25519 Hierarchical Deterministic Key Derivation (BIP-32 / BIP-44).
//!
//! Implements SLIP-0010 for the `ed25519` curve:
//! - Master node derivation: HMAC-SHA512 with key `b"ed25519 seed"`
//! - Hardened child key derivation: `0x00 || ser256(k_par) || ser32(i)` with `i >= 0x8000_0000`
//! - Non-hardened derivation is mathematically undefined for Ed25519 and strictly rejected
//! - Canonical Kovanica BIP-44 path: `m/44'/3007'/<account>'/<change>'/<index>'`

use hmac::{Hmac, Mac};
use sha2::Sha512;
use zeroize::{Zeroize, Zeroizing};

use crate::error::WalletError;

type HmacSha512 = Hmac<Sha512>;

/// Standard BIP-44 purpose constant for HD derivation (44).
pub const BIP44_PURPOSE: u32 = 44;

/// Registered coin type for Kovanica (3007).
pub const KOVANICA_COIN_TYPE: u32 = 3007;

/// Hardened derivation offset (2^31 = 0x8000_0000).
pub const HARDENED_OFFSET: u32 = 0x8000_0000;

/// A SLIP-0010 extended key node containing a 32-byte Ed25519 secret seed and 32-byte chain code.
#[derive(Clone)]
pub struct Slip10Node {
    pub secret_key: Zeroizing<[u8; 32]>,
    pub chain_code: [u8; 32],
}

impl std::fmt::Debug for Slip10Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Slip10Node")
            .field("chain_code", &hex::encode(self.chain_code))
            .field("secret_key", &"[REDACTED]")
            .finish()
    }
}

impl Drop for Slip10Node {
    fn drop(&mut self) {
        self.chain_code.zeroize();
    }
}

impl Slip10Node {
    /// Derive the master node from a 128..=512 bit binary seed (typically 64-byte BIP-39 seed).
    pub fn master(seed: &[u8]) -> Result<Self, WalletError> {
        if seed.len() < 16 || seed.len() > 64 {
            return Err(WalletError::Derivation(format!(
                "invalid seed length {}: SLIP-0010 seed must be between 16 and 64 bytes",
                seed.len()
            )));
        }

        let mut mac = HmacSha512::new_from_slice(b"ed25519 seed")
            .map_err(|e| WalletError::Derivation(format!("HMAC-SHA512 init failed: {e}")))?;
        mac.update(seed);
        let i = mac.finalize().into_bytes();

        let mut secret = [0u8; 32];
        let mut chain_code = [0u8; 32];
        secret.copy_from_slice(&i[0..32]);
        chain_code.copy_from_slice(&i[32..64]);

        Ok(Self {
            secret_key: Zeroizing::new(secret),
            chain_code,
        })
    }

    /// Derive a hardened child key at index `i >= 0x8000_0000`.
    ///
    /// Non-hardened derivation (`i < 0x8000_0000`) is not defined for Ed25519 in SLIP-0010
    /// and is strictly rejected with [`WalletError::Derivation`].
    pub fn derive_child(&self, index: u32) -> Result<Self, WalletError> {
        if index < HARDENED_OFFSET {
            return Err(WalletError::Derivation(format!(
                "SLIP-0010 Ed25519 does not support non-hardened child derivation: index {index:#010x} < {HARDENED_OFFSET:#010x}"
            )));
        }

        let mut mac = HmacSha512::new_from_slice(&self.chain_code)
            .map_err(|e| WalletError::Derivation(format!("HMAC-SHA512 init failed: {e}")))?;

        // Data: 0x00 || ser256(k_par) || ser32(i)
        mac.update(&[0x00]);
        mac.update(&self.secret_key[..]);
        mac.update(&index.to_be_bytes());

        let i = mac.finalize().into_bytes();
        let mut secret = [0u8; 32];
        let mut child_chain_code = [0u8; 32];
        secret.copy_from_slice(&i[0..32]);
        child_chain_code.copy_from_slice(&i[32..64]);

        Ok(Self {
            secret_key: Zeroizing::new(secret),
            chain_code: child_chain_code,
        })
    }

    /// Derive a child along a string path such as `m/44'/3007'/0'/0'/0'`.
    ///
    /// Every segment must be hardened (ending with `'` or `h` or `H`).
    pub fn derive_path(&self, path: &str) -> Result<Self, WalletError> {
        let trimmed = path.trim();
        let segments: Vec<&str> = trimmed.split('/').collect();
        if segments.is_empty() {
            return Err(WalletError::Derivation("empty derivation path".into()));
        }

        let mut start_idx = 0;
        if segments[0] == "m" || segments[0] == "M" {
            start_idx = 1;
        }

        let mut current = self.clone();
        for &seg in &segments[start_idx..] {
            if seg.is_empty() {
                continue;
            }
            let is_hardened = seg.ends_with('\'') || seg.ends_with('h') || seg.ends_with('H');
            if !is_hardened {
                return Err(WalletError::Derivation(format!(
                    "SLIP-0010 Ed25519 strictly requires hardened path segments: '{seg}' is not hardened"
                )));
            }
            let num_str = &seg[..seg.len() - 1];
            let raw_idx: u32 = num_str.parse().map_err(|e| {
                WalletError::Derivation(format!("invalid segment index '{num_str}': {e}"))
            })?;
            if raw_idx >= HARDENED_OFFSET {
                return Err(WalletError::Derivation(format!(
                    "segment index '{raw_idx}' already exceeds hardened offset"
                )));
            }
            let hardened_index = raw_idx
                .checked_add(HARDENED_OFFSET)
                .ok_or_else(|| WalletError::Derivation("index overflow".into()))?;
            current = current.derive_child(hardened_index)?;
        }

        Ok(current)
    }

    /// Derive standard Kovanica BIP-44 path `m/44'/3007'/<account>'/<change>'/<index>'`.
    pub fn derive_bip44(&self, account: u32, change: u32, index: u32) -> Result<Self, WalletError> {
        if account >= HARDENED_OFFSET || change >= HARDENED_OFFSET || index >= HARDENED_OFFSET {
            return Err(WalletError::Derivation(
                "account, change, and index must be unhardened indices (< 0x8000_0000)".into(),
            ));
        }

        let purpose_idx = BIP44_PURPOSE + HARDENED_OFFSET;
        let coin_idx = KOVANICA_COIN_TYPE + HARDENED_OFFSET;
        let account_idx = account + HARDENED_OFFSET;
        let change_idx = change + HARDENED_OFFSET;
        let addr_idx = index + HARDENED_OFFSET;

        self.derive_child(purpose_idx)?
            .derive_child(coin_idx)?
            .derive_child(account_idx)?
            .derive_child(change_idx)?
            .derive_child(addr_idx)
    }
}

/// Format the canonical Kovanica BIP-44 path for given account, change, and index.
pub fn format_bip44_path(account: u32, change: u32, index: u32) -> String {
    format!("m/{BIP44_PURPOSE}'/{KOVANICA_COIN_TYPE}'/{account}'/{change}'/{index}'")
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_types::signing::SigningContext;
    use kovanica_types::{Address, PublicKey, SigningKey};

    #[test]
    fn slip0010_test_vector_1() {
        // Official SLIP-0010 Test Vector 1 for Ed25519:
        // Seed (hex): 000102030405060708090a0b0c0d0e0f
        let seed = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();
        let master = Slip10Node::master(&seed).unwrap();

        assert_eq!(
            hex::encode(master.secret_key.as_slice()),
            "2b4be7f19ee27bbf30c667b642d5f4aa69fd169872f8fc3059c08ebae2eb19e7"
        );
        assert_eq!(
            hex::encode(master.chain_code),
            "90046a93de5380a72b5e45010748567d5ea02bbf6522f979e05c0d8d8ca9fffb"
        );

        // Path m/0'
        let m_0 = master.derive_child(HARDENED_OFFSET).unwrap();
        assert_eq!(
            hex::encode(m_0.secret_key.as_slice()),
            "68e0fe46dfb67e368c75379acec591dad19df3cde26e63b93a8e704f1dade7a3"
        );
        assert_eq!(
            hex::encode(m_0.chain_code),
            "8b59aa11380b624e81507a27fedda59fea6d0b779a778918a2fd3590e16e9c69"
        );

        // Path m/0'/1'
        let m_0_1 = m_0.derive_child(HARDENED_OFFSET + 1).unwrap();
        assert_eq!(
            hex::encode(m_0_1.secret_key.as_slice()),
            "b1d0bad404bf35da785a64ca1ac54b2617211d2777696fbffaf208f746ae84f2"
        );
        assert_eq!(
            hex::encode(m_0_1.chain_code),
            "a320425f77d1b5c2505a6b1b27382b37368ee640e3557c315416801243552f14"
        );

        // Verify string path derivation produces exact same keys
        let derived_from_path = master.derive_path("m/0'/1'").unwrap();
        assert_eq!(
            derived_from_path.secret_key.as_slice(),
            m_0_1.secret_key.as_slice()
        );
        assert_eq!(derived_from_path.chain_code, m_0_1.chain_code);
    }

    #[test]
    fn slip0010_test_vector_2() {
        // Official SLIP-0010 Test Vector 2 for Ed25519:
        let seed = hex::decode(
            "fffcf9f6f3f0edeae7e4e1dedbd8d5d2cfccc9c6c3c0bdbab7b4b1aeaba8a5a29f9c999693908d8a8784817e7b7875726f6c696663605d5a5754514e4b484542",
        )
        .unwrap();
        let master = Slip10Node::master(&seed).unwrap();

        assert_eq!(
            hex::encode(master.secret_key.as_slice()),
            "171cb88b1b3c1db25add599712e36245d75bc65a1a5c9e18d76f9f2b1eab4012"
        );
        assert_eq!(
            hex::encode(master.chain_code),
            "ef70a74db9c3a5af931b5fe73ed8e1a53464133654fd55e7a66f8570b8e33c3b"
        );

        // Path m/0'
        let m_0 = master.derive_child(HARDENED_OFFSET).unwrap();
        assert_eq!(
            hex::encode(m_0.secret_key.as_slice()),
            "1559eb2bbec5790b0c65d8693e4d0875b1747f4970ae8b650486ed7470845635"
        );
        assert_eq!(
            hex::encode(m_0.chain_code),
            "0b78a3226f915c082bf118f83618a618ab6dec793752624cbeb622acb562862d"
        );
    }

    #[test]
    fn rejects_non_hardened_derivation() {
        let seed = [42u8; 32];
        let master = Slip10Node::master(&seed).unwrap();

        // Index 0 (< 0x8000_0000) must be rejected
        let err = master.derive_child(0).unwrap_err();
        assert!(matches!(err, WalletError::Derivation(_)));
        assert!(err.to_string().contains("non-hardened"));

        // Path with non-hardened segment must be rejected
        let err = master.derive_path("m/44'/3007'/0'/0/0").unwrap_err();
        assert!(matches!(err, WalletError::Derivation(_)));
        assert!(err.to_string().contains("hardened"));
    }

    #[test]
    fn multi_account_derivation_and_transaction_signing() {
        let seed = [7u8; 64];
        let master = Slip10Node::master(&seed).unwrap();
        let ctx = SigningContext::new(5);

        // Derive accounts 0, 1, and 2 at standard path m/44'/3007'/<acc>'/0'/0'
        let mut addrs = Vec::new();
        let mut keys = Vec::new();

        for acc in 0..3 {
            let node = master.derive_bip44(acc, 0, 0).unwrap();
            let sk = SigningKey::from_bytes(&node.secret_key);
            let pk = PublicKey::from_bytes(sk.verifying_key().to_bytes());
            let addr = Address(pk.0);

            // Verify unique address per account
            assert!(!addrs.contains(&addr));
            addrs.push(addr);
            keys.push(sk);
        }

        // Verify each derived account key signs valid transactions
        for (i, sk) in keys.iter().enumerate() {
            let acc = crate::WalletAccount::from_seed(&sk.to_bytes());
            let tx = acc.sign_transfer(
                &ctx,
                Address([99u8; 32]),
                (i as u64 + 1) * 1_000_000_000,
                0,
                100,
            );
            assert!(tx.verify_signature(&ctx));
        }
    }
}
