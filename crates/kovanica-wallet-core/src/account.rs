use kovanica_crypto::generate_keypair;
use kovanica_sdk::tx::build;
use kovanica_types::crypto::SigningKey;
use kovanica_types::signing::SigningContext;
use kovanica_types::transaction::{Transaction, TransactionKind};
use kovanica_types::{Address, PublicKey};
use std::collections::HashMap;

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

    /// Apply a mock receipt to update balance/nonce locally.
    pub fn apply_receipt(
        &mut self,
        receipt: &kovanica_execution::TransactionReceipt,
        _ctx: &SigningContext,
    ) {
        // naive: increment nonce for sender? We don't have sender in receipt. Skip.
        let _ = receipt;
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

    const CTX: SigningContext = SigningContext {
        chain_id: 4,
        epoch: 0,
    };

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
}
