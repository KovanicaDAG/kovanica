use kovanica_crypto::generate_keypair;
use kovanica_sdk::tx::build;
use kovanica_types::crypto::SigningKey;
use kovanica_types::signing::SigningContext;
use kovanica_types::transaction::{Transaction, TransactionKind};
use kovanica_types::{Address, PublicKey};
use std::collections::HashMap;

use crate::error::WalletError;

#[derive(Clone)]
pub struct WalletAccount {
    pub address: Address,
    pub public_key: PublicKey,
    pub signing_key: SigningKey,
}

impl Default for WalletAccount {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletAccount {
    pub fn new() -> Self {
        let (sk, pk) = generate_keypair();
        Self {
            address: Address(pk.0),
            public_key: pk,
            signing_key: sk,
        }
    }

    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let sk = SigningKey::from_bytes(seed);
        let pk = PublicKey::from_bytes(sk.verifying_key().to_bytes());
        Self {
            address: Address(pk.0),
            public_key: pk,
            signing_key: sk,
        }
    }

    /// Derive a child account from a 64-byte BIP-39 binary seed using SLIP-0010 Ed25519 HD derivation
    /// at standard path `m/44'/3007'/<account>'/<change>'/<index>'`.
    pub fn from_bip39_seed_hd(
        seed_64: &[u8; 64],
        account: u32,
        change: u32,
        index: u32,
    ) -> Result<Self, WalletError> {
        let master = crate::hd::Slip10Node::master(seed_64)?;
        let child = master.derive_bip44(account, change, index)?;
        Ok(Self::from_seed(&child.secret_key))
    }

    /// Derive a child account from a BIP-39 mnemonic phrase and optional passphrase using
    /// SLIP-0010 Ed25519 HD derivation at standard path `m/44'/3007'/<account>'/<change>'/<index>'`.
    pub fn from_mnemonic_hd(
        phrase: &str,
        passphrase: &str,
        account: u32,
        change: u32,
        index: u32,
    ) -> Result<Self, WalletError> {
        let seed_64 = kovanica_keystore::bip39_seed_from_mnemonic(phrase, passphrase)
            .map_err(|e| WalletError::Keystore(e.to_string()))?;
        Self::from_bip39_seed_hd(&seed_64, account, change, index)
    }

    pub fn sign_transfer(
        &self,
        ctx: &SigningContext,
        to: Address,
        amount: u64,
        nonce: u64,
        fee: u64,
    ) -> Transaction {
        build(
            &self.signing_key,
            ctx,
            TransactionKind::Transfer { to, amount },
            nonce,
            fee,
        )
    }
}

pub struct Wallet {
    pub accounts: Vec<WalletAccount>,
    pub balances: HashMap<Address, u64>,
    pub nonces: HashMap<Address, u64>,
    pub token_balances: HashMap<Address, HashMap<String, u64>>,
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

impl Wallet {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            balances: HashMap::new(),
            nonces: HashMap::new(),
            token_balances: HashMap::new(),
        }
    }

    pub fn create_account(&mut self) -> Address {
        let acc = WalletAccount::new();
        let addr = acc.address;
        self.accounts.push(acc);
        self.balances.insert(addr, 0);
        self.nonces.insert(addr, 0);
        addr
    }

    pub fn import_seed(&mut self, seed: &[u8; 32]) -> Address {
        let acc = WalletAccount::from_seed(seed);
        let addr = acc.address;
        self.accounts.push(acc);
        self.balances.insert(addr, 0);
        self.nonces.insert(addr, 0);
        addr
    }

    /// Derive and register an HD account from a BIP-39 mnemonic phrase and optional passphrase
    /// at standard path `m/44'/3007'/<account>'/<change>'/<index>'`.
    pub fn derive_hd_account(
        &mut self,
        phrase: &str,
        passphrase: &str,
        account: u32,
        change: u32,
        index: u32,
    ) -> Result<Address, WalletError> {
        let acc = WalletAccount::from_mnemonic_hd(phrase, passphrase, account, change, index)?;
        let addr = acc.address;
        self.accounts.push(acc);
        self.balances.entry(addr).or_insert(0);
        self.nonces.entry(addr).or_insert(0);
        Ok(addr)
    }

    pub fn list_accounts(&self) -> Vec<Address> {
        self.accounts.iter().map(|a| a.address).collect()
    }

    pub fn get_account(&self, address: &Address) -> Option<&WalletAccount> {
        self.accounts.iter().find(|a| &a.address == address)
    }

    pub fn balance(&self, address: &Address) -> u64 {
        self.balances.get(address).copied().unwrap_or(0)
    }

    pub fn nonce(&self, address: &Address) -> u64 {
        self.nonces.get(address).copied().unwrap_or(0)
    }

    pub fn set_balance(&mut self, address: &Address, amount: u64) {
        self.balances.insert(*address, amount);
    }

    pub fn set_nonce(&mut self, address: &Address, nonce: u64) {
        self.nonces.insert(*address, nonce);
    }

    pub fn update_from_provider_result(&mut self, address: &Address, balance: u64, nonce: u64) {
        self.set_balance(address, balance);
        self.set_nonce(address, nonce);
    }

    pub fn apply_balance_update(&mut self, address: &Address, delta: i64) {
        let cur = self.balance(address) as i64;
        self.set_balance(address, (cur + delta).max(0) as u64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_types::signing::SigningContext;

    const CTX: SigningContext = SigningContext::new(4);

    #[test]
    fn key_lifecycle_and_sign() {
        let mut w = Wallet::new();
        let addr = w.create_account();
        let acc = w.get_account(&addr).unwrap();
        let tx = acc.sign_transfer(&CTX, kovanica_types::Address([2u8; 32]), 10, 0, 1);
        assert!(tx.verify_signature(&CTX));
        assert_eq!(w.balance(&addr), 0);
        w.set_balance(&addr, 1000);
        assert_eq!(w.balance(&addr), 1000);
        w.apply_balance_update(&addr, -100);
        assert_eq!(w.balance(&addr), 900);
    }

    #[test]
    fn import_seed_deterministic() {
        let seed = [7u8; 32];
        let mut w = Wallet::new();
        let a = w.import_seed(&seed);
        let mut w2 = Wallet::new();
        let b = w2.import_seed(&seed);
        assert_eq!(a, b);
    }

    #[test]
    fn hd_account_derivation_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
        let mut w1 = Wallet::new();
        let addr1 = w1.derive_hd_account(phrase, "", 0, 0, 0).unwrap();
        let addr2 = w1.derive_hd_account(phrase, "", 0, 0, 1).unwrap();
        assert_ne!(addr1, addr2);

        let mut w2 = Wallet::new();
        let addr1_repeat = w2.derive_hd_account(phrase, "", 0, 0, 0).unwrap();
        assert_eq!(addr1, addr1_repeat);

        let acc = WalletAccount::from_mnemonic_hd(phrase, "", 0, 0, 0).unwrap();
        assert_eq!(acc.address, addr1);
    }
}
