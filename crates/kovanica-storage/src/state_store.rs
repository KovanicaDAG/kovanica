//! State storage for Kovanica.
//!
//! Stores account balances, nonces, contract code/storage, and staking state.

use crate::{address_to_bytes, BincodeSerialize, StorageError};
use kovanica_staking::{Delegation, StakingState, TreasuryState, ValidatorInfo};
use kovanica_types::{Address, Stake};
use redb::{ReadTransaction, ReadableTable, WriteTransaction};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;

/// Errors specific to state store operations.
#[derive(Debug, Error)]
pub enum StateStoreError {
    #[error("Account not found: {0}")]
    NotFound(String),
    /// A debit larger than the account balance; nothing was written.
    #[error("Insufficient balance: account {address} has {balance}, debit {amount}")]
    InsufficientBalance {
        /// Account being debited.
        address: String,
        /// Balance before the (rejected) debit.
        balance: u64,
        /// Requested debit.
        amount: u64,
    },
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Database error: {0}")]
    Database(#[from] redb::DatabaseError),
    #[error("Transaction error: {0}")]
    Transaction(#[from] redb::TransactionError),
    #[error("Table error: {0}")]
    Table(#[from] redb::TableError),
    #[error("Commit error: {0}")]
    Commit(#[from] redb::CommitError),
    #[error("Storage error: {0}")]
    Storage(#[from] redb::StorageError),
}

/// Account state in the Kovanica blockchain.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Account {
    /// Account balance in base units.
    pub balance: u64,
    /// Transaction nonce (prevents replay).
    pub nonce: u64,
    /// Contract code hash (empty for EOAs).
    pub code_hash: [u8; 32],
    /// Contract code (only for contract accounts).
    pub code: Vec<u8>,
}

/// Configuration for state pruning.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PruningConfig {
    /// Number of recent committed leader heights to keep full state for.
    pub keep_recent: u64,
    /// Maximum number of state roots to keep in history.
    pub max_state_roots: u64,
}

impl Default for PruningConfig {
    fn default() -> Self {
        Self {
            keep_recent: 1000,     // Keep last 1000 committed leaders
            max_state_roots: 2000, // Keep up to 2000 state roots
        }
    }
}

/// State store for accounts, contracts, and staking.
pub struct StateStore {
    pruning: PruningConfig,
}

impl StateStore {
    /// Create a new state store, initializing tables if needed.
    pub fn new(db: &redb::Database) -> Result<Self, StorageError> {
        Self::with_pruning(db, PruningConfig::default())
    }

    /// Create a new state store with custom pruning config.
    pub fn with_pruning(db: &redb::Database, pruning: PruningConfig) -> Result<Self, StorageError> {
        let write_txn = db.begin_write().map_err(StorageError::Transaction)?;
        {
            let _ = write_txn.open_table(crate::tables::ACCOUNTS)?;
            let _ = write_txn.open_table(crate::tables::CONTRACT_CODE)?;
            let _ = write_txn.open_table(crate::tables::CONTRACT_STORAGE)?;
            let _ = write_txn.open_table(crate::tables::STAKING_STATE)?;
            let _ = write_txn.open_table(crate::tables::STATE_ROOT)?;
            let _ = write_txn.open_table(crate::tables::LEADER_HEADERS)?;
            let _ = write_txn.open_table(crate::tables::COMMIT_SEALS)?;
            let _ = write_txn.open_table(crate::tables::EXECUTED_TX_HASHES)?;
            let _ = write_txn.open_table(crate::tables::LIGHT_SET)?;
        }
        write_txn.commit().map_err(StorageError::Commit)?;
        Ok(Self { pruning })
    }

    // ============================================================
    // Account operations
    /// Get an account by address.
    pub fn get_account(
        &self,
        txn: &ReadTransaction,
        address: &Address,
    ) -> Result<Account, StateStoreError> {
        let key = address_to_bytes(address);
        let table = txn.open_table(crate::tables::ACCOUNTS)?;
        let value = table
            .get(key)?
            .ok_or_else(|| StateStoreError::NotFound(format!("account {}", address)))?;
        Ok(Account::from_bytes(&value.value())?)
    }

    /// Get an account, returning default if not found.
    pub fn get_account_or_default(
        &self,
        txn: &ReadTransaction,
        address: &Address,
    ) -> Result<Account, StateStoreError> {
        let key = address_to_bytes(address);
        let table = txn.open_table(crate::tables::ACCOUNTS)?;
        let value = table.get(key)?;
        Ok(value
            .map(|v| Account::from_bytes(&v.value()).unwrap_or_default())
            .unwrap_or_default())
    }

    /// Get an account, returning default if not found (write transaction variant).
    pub fn get_account_or_default_write(
        &self,
        txn: &WriteTransaction,
        address: &Address,
    ) -> Result<Account, StateStoreError> {
        let key = address_to_bytes(address);
        let table = txn.open_table(crate::tables::ACCOUNTS)?;
        let value = table.get(key)?;
        let account = value
            .map(|v| Account::from_bytes(&v.value()).unwrap_or_default())
            .unwrap_or_default();
        drop(table);
        Ok(account)
    }

    /// Set an account.
    pub fn set_account(
        &self,
        txn: &WriteTransaction,
        address: &Address,
        account: &Account,
    ) -> Result<(), StateStoreError> {
        let key = address_to_bytes(address);
        let mut table = txn.open_table(crate::tables::ACCOUNTS)?;
        table.insert(key, account.to_bytes()?)?;
        Ok(())
    }

    /// Update account balance.
    pub fn add_balance(
        &self,
        txn: &WriteTransaction,
        address: &Address,
        amount: u64,
    ) -> Result<u64, StateStoreError> {
        let key = address_to_bytes(address);
        let table = txn.open_table(crate::tables::ACCOUNTS)?;
        let value = table.get(key)?;
        let mut account = value
            .map(|v| Account::from_bytes(&v.value()).unwrap_or_default())
            .unwrap_or_default();
        drop(table);
        account.balance = account.balance.saturating_add(amount);
        self.set_account(txn, address, &account)?;
        Ok(account.balance)
    }

    /// Subtract `amount` from an existing account's balance and return the
    /// new balance.
    ///
    /// Errors with `NotFound` if the account does not exist and with
    /// `InsufficientBalance` if `amount` exceeds the balance; in both cases
    /// nothing is written. Safe to combine with other table operations in
    /// the same write transaction: the ACCOUNTS table and its read guard
    /// are dropped before the write.
    pub fn sub_balance(
        &self,
        txn: &WriteTransaction,
        address: &Address,
        amount: u64,
    ) -> Result<u64, StateStoreError> {
        let key = address_to_bytes(address);
        let mut account = {
            let table = txn.open_table(crate::tables::ACCOUNTS)?;
            let value = table
                .get(key)?
                .ok_or_else(|| StateStoreError::NotFound(format!("account {}", address)))?;
            Account::from_bytes(&value.value())?
            // `value` (read guard) and `table` are dropped at the end of
            // this block, before `set_account` reopens ACCOUNTS.
        };
        account.balance = account.balance.checked_sub(amount).ok_or_else(|| {
            StateStoreError::InsufficientBalance {
                address: address.to_string(),
                balance: account.balance,
                amount,
            }
        })?;
        self.set_account(txn, address, &account)?;
        Ok(account.balance)
    }

    /// Increment account nonce.
    pub fn increment_nonce(
        &self,
        txn: &WriteTransaction,
        address: &Address,
    ) -> Result<u64, StateStoreError> {
        let key = address_to_bytes(address);
        let table = txn.open_table(crate::tables::ACCOUNTS)?;
        let value = table.get(key)?;
        let mut account = value
            .map(|v| Account::from_bytes(&v.value()).unwrap_or_default())
            .unwrap_or_default();
        drop(table);
        account.nonce = account.nonce.saturating_add(1);
        self.set_account(txn, address, &account)?;
        Ok(account.nonce)
    }

    /// Set account nonce.
    pub fn set_nonce(
        &self,
        txn: &WriteTransaction,
        address: &Address,
        nonce: u64,
    ) -> Result<(), StateStoreError> {
        let key = address_to_bytes(address);
        let table = txn.open_table(crate::tables::ACCOUNTS)?;
        let value = table.get(key)?;
        let mut account = value
            .map(|v| Account::from_bytes(&v.value()).unwrap_or_default())
            .unwrap_or_default();
        drop(table);
        account.nonce = nonce;
        self.set_account(txn, address, &account)?;
        Ok(())
    }

    // ============================================================
    // Contract operations
    // ============================================================

    /// Deploy contract code.
    pub fn deploy_contract(
        &self,
        txn: &WriteTransaction,
        address: &Address,
        code: Vec<u8>,
    ) -> Result<[u8; 32], StateStoreError> {
        // Compute code hash
        let code_hash = blake3::hash(&code);
        let code_hash_bytes: [u8; 32] = code_hash.as_bytes().clone().try_into().unwrap();

        // Store code
        {
            let mut table = txn.open_table(crate::tables::CONTRACT_CODE)?;
            table.insert(code_hash_bytes, code.clone())?;
        }

        // Update account
        let key = address_to_bytes(address);
        let table = txn.open_table(crate::tables::ACCOUNTS)?;
        let value = table.get(key)?;
        let mut account = value
            .map(|v| Account::from_bytes(&v.value()).unwrap_or_default())
            .unwrap_or_default();
        drop(table);
        account.code_hash = code_hash_bytes;
        account.code = code;
        self.set_account(txn, address, &account)?;

        Ok(code_hash_bytes)
    }

    /// Get contract code by hash.
    pub fn get_contract_code(
        &self,
        txn: &ReadTransaction,
        code_hash: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, StateStoreError> {
        let table = txn.open_table(crate::tables::CONTRACT_CODE)?;
        Ok(table.get(*code_hash)?.map(|v| v.value().to_vec()))
    }

    /// Get contract storage value.
    pub fn get_storage(
        &self,
        txn: &ReadTransaction,
        contract: &Address,
        key: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, StateStoreError> {
        let contract_bytes = address_to_bytes(contract);
        let table = txn.open_table(crate::tables::CONTRACT_STORAGE)?;
        Ok(table
            .get((contract_bytes, *key))?
            .map(|v| v.value().to_vec()))
    }

    /// Set contract storage value.
    pub fn set_storage(
        &self,
        txn: &WriteTransaction,
        contract: &Address,
        key: [u8; 32],
        value: Vec<u8>,
    ) -> Result<(), StateStoreError> {
        let contract_bytes = address_to_bytes(contract);
        let mut table = txn.open_table(crate::tables::CONTRACT_STORAGE)?;
        table.insert((contract_bytes, key), value)?;
        Ok(())
    }

    // ============================================================
    // Staking state operations
    // ============================================================

    /// Save the entire staking state.
    pub fn save_staking_state(
        &self,
        txn: &WriteTransaction,
        state: &StakingState,
    ) -> Result<(), StateStoreError> {
        let mut table = txn.open_table(crate::tables::STAKING_STATE)?;
        table.insert("staking", state.to_bytes()?)?;
        Ok(())
    }

    /// Store the light-node set. This is a separate table from staking state.
    pub fn save_light_set(
        &self,
        txn: &WriteTransaction,
        set: &kovanica_staking::LightSet,
    ) -> Result<(), StateStoreError> {
        let mut table = txn.open_table(crate::tables::LIGHT_SET)?;
        table.insert("current", set.to_bytes()?)?;
        Ok(())
    }

    /// Remove the light-node set row, if one was written.
    pub fn clear_light_set(&self, txn: &WriteTransaction) -> Result<(), StateStoreError> {
        let mut table = txn.open_table(crate::tables::LIGHT_SET)?;
        table.remove("current")?;
        Ok(())
    }

    /// Load the light-node set from a write transaction.
    pub fn load_light_set_write(
        &self,
        txn: &WriteTransaction,
    ) -> Result<Option<kovanica_staking::LightSet>, StateStoreError> {
        let table = txn.open_table(crate::tables::LIGHT_SET)?;
        let bytes = table.get("current")?.map(|guard| guard.value().to_vec());
        match bytes {
            Some(bytes) => Ok(Some(kovanica_staking::LightSet::from_bytes(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Load the light-node set from a read transaction.
    pub fn load_light_set(
        &self,
        txn: &ReadTransaction,
    ) -> Result<Option<kovanica_staking::LightSet>, StateStoreError> {
        let table = txn.open_table(crate::tables::LIGHT_SET)?;
        let bytes = table.get("current")?.map(|guard| guard.value().to_vec());
        match bytes {
            Some(bytes) => Ok(Some(kovanica_staking::LightSet::from_bytes(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Load the staking state.
    pub fn load_staking_state(
        &self,
        txn: &ReadTransaction,
    ) -> Result<StakingState, StateStoreError> {
        let table = txn.open_table(crate::tables::STAKING_STATE)?;
        let value = table
            .get("staking")?
            .ok_or_else(|| StateStoreError::NotFound("staking state".to_string()))?;
        Ok(StakingState::from_bytes(&value.value())?)
    }

    // ============================================================
    // State root operations (for snapshots/light clients)
    // ============================================================

    /// Rebuild the account sparse Merkle from the accounts table and store
    /// it as the state root at `committed_leader_height`.
    ///
    /// Only balance and nonce are committed. A row with both at zero is
    /// skipped, so it matches an absent account. Call this at the end of a
    /// committed-leader write, after every account mutation in that
    /// transaction.
    pub fn commit_account_smt(
        &self,
        txn: &WriteTransaction,
        committed_leader_height: u64,
    ) -> Result<kovanica_types::hash::Hash, StateStoreError> {
        let mut tree = kovanica_types::AccountSmt::empty();
        {
            let table = txn.open_table(crate::tables::ACCOUNTS)?;
            for entry in table.iter()? {
                let (key, value) = entry?;
                let account = Account::from_bytes(&value.value())?;
                if account.balance == 0 && account.nonce == 0 {
                    continue;
                }
                let mut address = [0u8; 32];
                address.copy_from_slice(key.value().as_ref());
                tree.insert(kovanica_types::Address(address), account.balance, account.nonce);
            }
        }
        let root = tree.root();
        self.save_state_root(txn, committed_leader_height, &root)?;
        Ok(root)
    }

    /// Prove `address` against the accounts currently stored.
    ///
    /// The proof's root is the tree of those rows. It matches
    /// [`Self::commit_account_smt`] when no account write happened since
    /// that call.
    pub fn prove_account(
        &self,
        txn: &ReadTransaction,
        address: &kovanica_types::Address,
    ) -> Result<kovanica_types::StateProof, StateStoreError> {
        let mut tree = kovanica_types::AccountSmt::empty();
        let mut found: Option<Account> = None;
        {
            let table = txn.open_table(crate::tables::ACCOUNTS)?;
            for entry in table.iter()? {
                let (key, value) = entry?;
                let account = Account::from_bytes(&value.value())?;
                let mut row_address = [0u8; 32];
                row_address.copy_from_slice(key.value().as_ref());
                if row_address == address.0 {
                    found = Some(account.clone());
                }
                if account.balance == 0 && account.nonce == 0 {
                    continue;
                }
                tree.insert(
                    kovanica_types::Address(row_address),
                    account.balance,
                    account.nonce,
                );
            }
        }
        let (present, balance, nonce) = match found {
            Some(account) if account.balance != 0 || account.nonce != 0 => {
                (true, account.balance, account.nonce)
            }
            _ => (false, 0, 0),
        };
        let proof = tree.prove(address, balance, nonce, present);
        Ok(kovanica_types::StateProof {
            state_root: tree.root(),
            address: *address,
            balance,
            nonce,
            present,
            siblings: proof.siblings.to_vec(),
        })
    }

    /// Store the locally executed leader header.
    ///
    /// A header already stored at this height with a different hash is
    /// replaced, and every seal at that height is deleted. The local
    /// execution result is the one later seals must match.
    pub fn save_leader_header(
        &self,
        txn: &WriteTransaction,
        header: &kovanica_types::LeaderHeader,
    ) -> Result<(), StateStoreError> {
        let bytes = bincode::serialize(header)?;
        let height = header.committed_height;
        let new_hash = header.hash();
        let replaced = {
            let table = txn.open_table(crate::tables::LEADER_HEADERS)?;
            let existing = table.get(height)?.map(|guard| guard.value().to_vec());
            match existing {
                Some(bytes) => {
                    let old: kovanica_types::LeaderHeader = bincode::deserialize(&bytes)?;
                    old.hash() != new_hash
                }
                None => false,
            }
        };
        if replaced {
            self.delete_seals_at(txn, height)?;
        }
        let mut table = txn.open_table(crate::tables::LEADER_HEADERS)?;
        table.insert(height, bytes)?;
        Ok(())
    }

    /// Leader header stored for `height`, if this node has executed it.
    pub fn get_leader_header(
        &self,
        txn: &ReadTransaction,
        height: u64,
    ) -> Result<Option<kovanica_types::LeaderHeader>, StateStoreError> {
        let table = txn.open_table(crate::tables::LEADER_HEADERS)?;
        let bytes = table.get(height)?.map(|guard| guard.value().to_vec());
        match bytes {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Highest stored leader header. Keys are committed heights, so the
    /// last row is the latest.
    pub fn get_latest_leader_header(
        &self,
        txn: &ReadTransaction,
    ) -> Result<Option<kovanica_types::LeaderHeader>, StateStoreError> {
        let table = txn.open_table(crate::tables::LEADER_HEADERS)?;
        let mut latest = None;
        for entry in table.iter()? {
            let (_height, value) = entry?;
            let bytes = value.value().to_vec();
            latest = Some(bincode::deserialize(&bytes)?);
        }
        Ok(latest)
    }

    /// Store one seal. A different signature already stored for the same
    /// authority is left in place.
    pub fn save_commit_seal(
        &self,
        txn: &WriteTransaction,
        height: u64,
        seal: &kovanica_types::CommitSeal,
    ) -> Result<(), StateStoreError> {
        let mut table = txn.open_table(crate::tables::COMMIT_SEALS)?;
        let key = (height, seal.authority_index);
        if table.get(key)?.is_some() {
            return Ok(());
        }
        table.insert(key, seal.signature.0.to_vec())?;
        Ok(())
    }

    /// Store `seal` only when the header at `height` hashes to `header_hash`.
    pub fn save_seal_if_header(
        &self,
        txn: &WriteTransaction,
        height: u64,
        header_hash: &kovanica_types::hash::Hash,
        seal: &kovanica_types::CommitSeal,
    ) -> Result<bool, StateStoreError> {
        let stored_hash = {
            let table = txn.open_table(crate::tables::LEADER_HEADERS)?;
            let bytes = table.get(height)?.map(|guard| guard.value().to_vec());
            match bytes {
                Some(bytes) => {
                    let header: kovanica_types::LeaderHeader = bincode::deserialize(&bytes)?;
                    Some(header.hash())
                }
                None => None,
            }
        };
        if stored_hash.as_ref() != Some(header_hash) {
            return Ok(false);
        }
        self.save_commit_seal(txn, height, seal)?;
        Ok(true)
    }

    /// Seals stored for `height`, in authority-index order.
    pub fn seals_for(
        &self,
        txn: &ReadTransaction,
        height: u64,
    ) -> Result<Vec<kovanica_types::CommitSeal>, StateStoreError> {
        let table = txn.open_table(crate::tables::COMMIT_SEALS)?;
        let mut seals = Vec::new();
        for entry in table.range((height, 0u16)..=(height, u16::MAX))? {
            let (key, value) = entry?;
            let (_, index) = key.value();
            let bytes = value.value().to_vec();
            if bytes.len() != 64 {
                return Err(StateStoreError::NotFound(format!(
                    "commit seal at height {height} authority {index} has {} bytes",
                    bytes.len()
                )));
            }
            let mut signature = [0u8; 64];
            signature.copy_from_slice(&bytes);
            seals.push(kovanica_types::CommitSeal {
                authority_index: index,
                signature: kovanica_types::Signature(signature),
            });
        }
        Ok(seals)
    }

    /// Replace the executed-transaction hash list for `height`.
    pub fn save_executed_tx_hashes(
        &self,
        txn: &WriteTransaction,
        height: u64,
        hashes: &[kovanica_types::hash::Hash],
    ) -> Result<(), StateStoreError> {
        let bytes = bincode::serialize(hashes)?;
        let mut table = txn.open_table(crate::tables::EXECUTED_TX_HASHES)?;
        table.insert(height, bytes)?;
        Ok(())
    }

    /// Executed-transaction hashes for `height`, if recorded.
    pub fn get_executed_tx_hashes(
        &self,
        txn: &ReadTransaction,
        height: u64,
    ) -> Result<Option<Vec<kovanica_types::hash::Hash>>, StateStoreError> {
        let table = txn.open_table(crate::tables::EXECUTED_TX_HASHES)?;
        let bytes = table.get(height)?.map(|guard| guard.value().to_vec());
        match bytes {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }

    fn delete_seals_at(&self, txn: &WriteTransaction, height: u64) -> Result<(), StateStoreError> {
        let keys: Vec<(u64, u16)> = {
            let table = txn.open_table(crate::tables::COMMIT_SEALS)?;
            let mut keys = Vec::new();
            for entry in table.range((height, 0u16)..=(height, u16::MAX))? {
                let (key, _) = entry?;
                keys.push(key.value());
            }
            keys
        };
        if keys.is_empty() {
            return Ok(());
        }
        let mut table = txn.open_table(crate::tables::COMMIT_SEALS)?;
        for key in keys {
            table.remove(key)?;
        }
        Ok(())
    }

    /// Save state root for a committed leader height.
    pub fn save_state_root(
        &self,
        txn: &WriteTransaction,
        committed_leader_height: u64,
        state_root: &kovanica_types::hash::Hash,
    ) -> Result<(), StateStoreError> {
        let mut table = txn.open_table(crate::tables::STATE_ROOT)?;
        table.insert(&committed_leader_height, state_root.0)?;
        Ok(())
    }

    /// Get state root for a committed leader height.
    pub fn get_state_root(
        &self,
        txn: &ReadTransaction,
        committed_leader_height: u64,
    ) -> Result<Option<kovanica_types::hash::Hash>, StateStoreError> {
        let table = txn.open_table(crate::tables::STATE_ROOT)?;
        Ok(table
            .get(&committed_leader_height)?
            .map(|v| kovanica_types::hash::Hash(v.value())))
    }

    /// Get latest state root.
    pub fn get_latest_state_root(
        &self,
        txn: &ReadTransaction,
    ) -> Result<Option<(u64, kovanica_types::hash::Hash)>, StateStoreError> {
        let table = txn.open_table(crate::tables::STATE_ROOT)?;
        let mut latest = None;
        for entry in table.iter()? {
            let (height, hash) = entry?;
            latest = Some((height.value(), kovanica_types::hash::Hash(hash.value())));
        }
        Ok(latest)
    }

    // ============================================================
    // Sorted KV Merkle root
    // ============================================================

    /// Compute a deterministic sorted key-value Merkle root over all
    /// state tables (accounts, contract code, contract storage,
    /// staking state). Each leaf = H(key || value) using BLAKE3.
    /// Keys are sorted lexicographically (as byte arrays) before
    /// hashing so the result is deterministic.
    pub fn compute_state_root(
        &self,
        txn: &ReadTransaction,
    ) -> Result<kovanica_types::hash::Hash, StateStoreError> {
        let mut entries: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();

        // Accounts table
        {
            let table = txn.open_table(crate::tables::ACCOUNTS)?;
            for entry in table.iter()? {
                let (key_bytes, value_bytes) = entry?;
                let mut leaf_key = key_bytes.value().to_vec();
                leaf_key.extend_from_slice(value_bytes.value().as_slice());
                entries.insert(key_bytes.value().to_vec(), leaf_key);
            }
        }

        // Contract code table
        {
            let table = txn.open_table(crate::tables::CONTRACT_CODE)?;
            for entry in table.iter()? {
                let (key_bytes, value_bytes) = entry?;
                let mut leaf_key = b"code:".to_vec();
                leaf_key.extend_from_slice(key_bytes.value().as_ref());
                let mut combined = leaf_key.clone();
                combined.extend_from_slice(value_bytes.value().as_slice());
                entries.insert(leaf_key, combined);
            }
        }

        // Contract storage table
        {
            let table = txn.open_table(crate::tables::CONTRACT_STORAGE)?;
            for entry in table.iter()? {
                let (key_pair_bytes, value_bytes) = entry?;
                let key_pair: ([u8; 32], [u8; 32]) = bincode::deserialize(
                    &bincode::serialize(&key_pair_bytes.value()).unwrap_or_default()[..],
                )?;
                let mut leaf_key = b"storage:".to_vec();
                leaf_key.extend_from_slice(&key_pair.0);
                leaf_key.extend_from_slice(&key_pair.1);
                let mut combined = leaf_key.clone();
                combined.extend_from_slice(value_bytes.value().as_slice());
                entries.insert(leaf_key, combined);
            }
        }

        // Staking state table
        {
            let table = txn.open_table(crate::tables::STAKING_STATE)?;
            for entry in table.iter()? {
                let (key_bytes, value_bytes) = entry?;
                let mut leaf_key = b"staking:".to_vec();
                leaf_key.extend_from_slice(key_bytes.value().as_bytes());
                let mut combined = leaf_key.clone();
                combined.extend_from_slice(value_bytes.value().as_slice());
                entries.insert(leaf_key, combined);
            }
        }

        let mut leaves: Vec<[u8; 32]> = Vec::with_capacity(entries.len());
        for (_, combined) in entries {
            let hash = kovanica_types::hash::Hash::new(&combined);
            leaves.push(hash.0);
        }
        Ok(kovanica_types::hash::Hash(merkle_root(leaves)))
    }

    /// Export a snapshot of all state data to a file using bincode.
    pub fn export_snapshot<P: AsRef<std::path::Path>>(
        &self,
        txn: &ReadTransaction,
        path: P,
    ) -> Result<(), StateStoreError> {
        use std::fs::File;
        use std::io::Write;

        #[derive(Serialize, Deserialize, Debug, Default)]
        struct SnapshotData {
            accounts: Vec<(Vec<u8>, Vec<u8>)>,
            contract_code: Vec<(Vec<u8>, Vec<u8>)>,
            contract_storage: Vec<(Vec<u8>, Vec<u8>)>,
            staking_state: Vec<(Vec<u8>, Vec<u8>)>,
        }

        let mut data = SnapshotData::default();
        {
            let table = txn.open_table(crate::tables::ACCOUNTS)?;
            for entry in table.iter()? {
                let (k, v) = entry?;
                data.accounts.push((k.value().to_vec(), v.value().to_vec()));
            }
        }
        {
            let table = txn.open_table(crate::tables::CONTRACT_CODE)?;
            for entry in table.iter()? {
                let (k, v) = entry?;
                data.contract_code
                    .push((k.value().to_vec(), v.value().to_vec()));
            }
        }
        {
            let table = txn.open_table(crate::tables::CONTRACT_STORAGE)?;
            for entry in table.iter()? {
                let (k, v) = entry?;
                let k_bytes = bincode::serialize(&k.value()).unwrap();
                data.contract_storage.push((k_bytes, v.value().to_vec()));
            }
        }
        {
            let table = txn.open_table(crate::tables::STAKING_STATE)?;
            for entry in table.iter()? {
                let (k, v) = entry?;
                data.staking_state
                    .push((k.value().as_bytes().to_vec(), v.value().to_vec()));
            }
        }

        let bytes = bincode::serialize(&data)?;
        let mut file = File::create(path.as_ref())
            .map_err(|e| StateStoreError::NotFound(format!("snapshot file error: {e}")))?;
        file.write_all(&bytes)
            .map_err(|e| StateStoreError::NotFound(format!("snapshot write error: {e}")))?;
        Ok(())
    }

    /// Restore a snapshot from a file. Requires a write transaction.
    pub fn import_snapshot<P: AsRef<std::path::Path>>(
        &self,
        txn: &WriteTransaction,
        path: P,
    ) -> Result<(), StateStoreError> {
        use std::fs;

        #[derive(Serialize, Deserialize, Debug, Default)]
        struct SnapshotData {
            accounts: Vec<(Vec<u8>, Vec<u8>)>,
            contract_code: Vec<(Vec<u8>, Vec<u8>)>,
            contract_storage: Vec<(Vec<u8>, Vec<u8>)>,
            staking_state: Vec<(Vec<u8>, Vec<u8>)>,
        }

        let bytes = fs::read(path.as_ref())
            .map_err(|e| StateStoreError::NotFound(format!("snapshot read error: {e}")))?;
        let data: SnapshotData = bincode::deserialize(&bytes)?;

        {
            let table = txn.open_table(crate::tables::ACCOUNTS)?;
            let keys: Vec<[u8; 32]> = table
                .iter()?
                .filter_map(|e| e.ok())
                .map(|(k, _)| k.value())
                .collect();
            drop(table);
            let mut table = txn.open_table(crate::tables::ACCOUNTS)?;
            for k in keys {
                table.remove(k)?;
            }
            for (k, v) in data.accounts {
                let k32: [u8; 32] = k.try_into().unwrap_or([0; 32]);
                table.insert(k32, v)?;
            }
        }
        {
            let table = txn.open_table(crate::tables::CONTRACT_CODE)?;
            let keys: Vec<[u8; 32]> = table
                .iter()?
                .filter_map(|e| e.ok())
                .map(|(k, _)| k.value())
                .collect();
            drop(table);
            let mut table = txn.open_table(crate::tables::CONTRACT_CODE)?;
            for k in keys {
                table.remove(k)?;
            }
            for (k, v) in data.contract_code {
                let k32: [u8; 32] = k.try_into().unwrap_or([0; 32]);
                table.insert(k32, v)?;
            }
        }
        {
            let table = txn.open_table(crate::tables::CONTRACT_STORAGE)?;
            let keys: Vec<([u8; 32], [u8; 32])> = table
                .iter()?
                .filter_map(|e| e.ok())
                .map(|(k, _)| {
                    let b = k.value();
                    let serialized = bincode::serialize(&b).unwrap_or_default();
                    bincode::deserialize(&serialized[..]).unwrap_or_default()
                })
                .collect();
            drop(table);
            let mut table = txn.open_table(crate::tables::CONTRACT_STORAGE)?;
            for k in keys {
                table.remove(k)?;
            }
            for (k, v) in data.contract_storage {
                let k_tuple: ([u8; 32], [u8; 32]) = bincode::deserialize(&k)?;
                table.insert(k_tuple, v)?;
            }
        }
        {
            let table = txn.open_table(crate::tables::STAKING_STATE)?;
            let keys: Vec<String> = table
                .iter()?
                .filter_map(|e| e.ok())
                .map(|(k, _)| k.value().to_string())
                .collect();
            drop(table);
            let mut table = txn.open_table(crate::tables::STAKING_STATE)?;
            for k in keys {
                table.remove(k.as_str())?;
            }
            for (k, v) in data.staking_state {
                table.insert("staking", v)?;
            }
        }
        Ok(())
    }

    /// Prune old state roots beyond the configured retention.
    /// Keeps the most recent `max_state_roots` entries.
    pub fn prune_state_roots(&self, txn: &WriteTransaction) -> Result<u64, StateStoreError> {
        let mut table = txn.open_table(crate::tables::STATE_ROOT)?;
        let mut roots: Vec<u64> = table
            .iter()?
            .filter_map(|e| e.ok())
            .map(|(k, _)| k.value())
            .collect();

        if roots.len() <= self.pruning.max_state_roots as usize {
            return Ok(0);
        }

        // Sort by height descending (newest first)
        roots.sort_by_key(|h| std::cmp::Reverse(*h));

        // Remove oldest entries beyond max_state_roots
        let to_remove = &roots[self.pruning.max_state_roots as usize..];
        let mut removed = 0u64;
        for height in to_remove {
            if table.remove(*height)?.is_some() {
                removed += 1;
            }
        }

        Ok(removed)
    }

    /// Get the current pruning configuration.
    pub fn pruning_config(&self) -> &PruningConfig {
        &self.pruning
    }
}

/// Compute binary Merkle root from sorted leaf hashes.
fn merkle_root(mut leaves: Vec<[u8; 32]>) -> [u8; 32] {
    if leaves.is_empty() {
        return [0u8; 32];
    }
    while leaves.len() > 1 {
        if leaves.len() % 2 == 1 {
            leaves.push(leaves[leaves.len() - 1]);
        }
        let mut next_level = Vec::with_capacity(leaves.len() / 2);
        for pair in leaves.chunks(2) {
            let mut combined = pair[0].to_vec();
            combined.extend_from_slice(&pair[1]);
            let hash = kovanica_types::hash::Hash::new(&combined);
            next_level.push(hash.0);
        }
        leaves = next_level;
    }
    leaves[0]
}

#[cfg(test)]
mod sub_balance_tests {
    use super::*;
    use crate::Storage;
    use kovanica_types::Address;
    use tempfile::tempdir;

    fn seeded(balance: u64) -> (tempfile::TempDir, Storage, Address) {
        let dir = tempdir().unwrap();
        let storage = Storage::new(dir.path().join("sub.db")).unwrap();
        let addr = Address([9; 32]);
        let txn = storage.begin_write().unwrap();
        storage.state().add_balance(&txn, &addr, balance).unwrap();
        txn.commit().unwrap();
        (dir, storage, addr)
    }

    fn balance(storage: &Storage, addr: &Address) -> u64 {
        let txn = storage.begin_write().unwrap();
        storage
            .state()
            .get_account_or_default_write(&txn, addr)
            .unwrap()
            .balance
    }

    #[test]
    fn sub_balance_then_other_table_op_in_one_txn_commits() {
        let (_dir, storage, addr) = seeded(1_000);
        let other = Address([8; 32]);
        let txn = storage.begin_write().unwrap();
        let left = storage.state().sub_balance(&txn, &addr, 300).unwrap();
        assert_eq!(left, 700);
        // Another ACCOUNTS write and a STAKING_STATE table op in the same txn.
        storage.state().add_balance(&txn, &other, 300).unwrap();
        txn.open_table(crate::tables::STAKING_STATE).unwrap();
        txn.commit().unwrap();
        assert_eq!(balance(&storage, &addr), 700);
        assert_eq!(balance(&storage, &other), 300);
    }

    #[test]
    fn sub_balance_overdraft_is_rejected_and_balance_unchanged() {
        let (_dir, storage, addr) = seeded(100);
        let txn = storage.begin_write().unwrap();
        let err = storage.state().sub_balance(&txn, &addr, 101).unwrap_err();
        assert!(
            matches!(err, StateStoreError::InsufficientBalance { .. }),
            "got {err:?}"
        );
        txn.commit().unwrap();
        assert_eq!(balance(&storage, &addr), 100);
    }

    #[test]
    fn sub_balance_exact_balance_to_zero() {
        let (_dir, storage, addr) = seeded(250);
        let txn = storage.begin_write().unwrap();
        assert_eq!(storage.state().sub_balance(&txn, &addr, 250).unwrap(), 0);
        txn.commit().unwrap();
        assert_eq!(balance(&storage, &addr), 0);
    }
}

#[cfg(test)]
mod account_smt_tests {
    use super::*;
    use crate::Storage;
    use kovanica_types::Address;
    use tempfile::tempdir;

    #[test]
    fn committed_root_matches_an_account_proof() {
        let dir = tempdir().unwrap();
        let storage = Storage::new(dir.path().join("smt.db")).unwrap();
        let addr = Address([4; 32]);
        let other = Address([8; 32]);
        let root = {
            let txn = storage.begin_write().unwrap();
            storage
                .state()
                .set_account(
                    &txn,
                    &addr,
                    &Account {
                        balance: 25,
                        nonce: 2,
                        code_hash: [9; 32],
                        code: vec![1, 2, 3],
                    },
                )
                .unwrap();
            storage
                .state()
                .set_account(
                    &txn,
                    &other,
                    &Account {
                        balance: 0,
                        nonce: 0,
                        code_hash: [1; 32],
                        code: vec![9],
                    },
                )
                .unwrap();
            let root = storage.state().commit_account_smt(&txn, 3).unwrap();
            txn.commit().unwrap();
            root
        };
        let txn = storage.begin_read().unwrap();
        let proof = storage.state().prove_account(&txn, &addr).unwrap();
        assert_eq!(proof.state_root, root);
        assert!(proof.present);
        assert_eq!(proof.balance, 25);
        assert_eq!(proof.nonce, 2);
        assert!(proof.verify());
        let absent = storage.state().prove_account(&txn, &other).unwrap();
        assert!(!absent.present);
        assert_eq!(absent.state_root, root);
        assert!(absent.verify());
        let saved = storage.state().get_state_root(&txn, 3).unwrap();
        assert_eq!(saved, Some(root));
    }
}

#[cfg(test)]
mod leader_header_tests {
    use super::*;
    use crate::Storage;
    use kovanica_types::{CommitSeal, Hash, LeaderHeader, Signature};
    use tempfile::tempdir;

    fn header(height: u64, state: u8) -> LeaderHeader {
        LeaderHeader {
            committed_height: height,
            leader_round: 3,
            leader_digest: Hash::new([height as u8]),
            leader_author: 0,
            parent_leader_digest: Hash::zero(),
            tx_merkle_root: Hash::zero(),
            executed_tx_root: Hash::zero(),
            non_blue_root: Hash::zero(),
            blue_root: Hash::zero(),
            mysticghost: false,
            state_root: Hash::new([state]),
            epoch: 0,
            committee_hash: Hash::zero(),
            timestamp: 0,
        }
    }

    fn seal(index: u16, byte: u8) -> CommitSeal {
        CommitSeal {
            authority_index: index,
            signature: Signature([byte; 64]),
        }
    }

    #[test]
    fn replacing_a_header_drops_seals_for_that_height() {
        let dir = tempdir().unwrap();
        let storage = Storage::new(dir.path().join("seals.db")).unwrap();
        let first = header(1, 1);
        let txn = storage.begin_write().unwrap();
        storage.state().save_leader_header(&txn, &first).unwrap();
        storage
            .state()
            .save_commit_seal(&txn, 1, &seal(0, 9))
            .unwrap();
        storage
            .state()
            .save_executed_tx_hashes(&txn, 1, &[Hash::new([3u8])])
            .unwrap();
        txn.commit().unwrap();

        let second = header(1, 2);
        assert_ne!(first.hash(), second.hash());
        let txn = storage.begin_write().unwrap();
        storage.state().save_leader_header(&txn, &second).unwrap();
        let kept = storage
            .state()
            .save_seal_if_header(&txn, 1, &first.hash(), &seal(1, 4))
            .unwrap();
        assert!(!kept);
        let stored = storage
            .state()
            .save_seal_if_header(&txn, 1, &second.hash(), &seal(1, 4))
            .unwrap();
        assert!(stored);
        txn.commit().unwrap();

        let txn = storage.begin_read().unwrap();
        let loaded = storage.state().get_leader_header(&txn, 1).unwrap().unwrap();
        assert_eq!(loaded.hash(), second.hash());
        let seals = storage.state().seals_for(&txn, 1).unwrap();
        assert_eq!(seals.len(), 1);
        assert_eq!(seals[0].authority_index, 1);
        let hashes = storage
            .state()
            .get_executed_tx_hashes(&txn, 1)
            .unwrap()
            .unwrap();
        assert_eq!(hashes, vec![Hash::new([3u8])]);
        assert!(storage
            .state()
            .get_leader_header(&txn, 9)
            .unwrap()
            .is_none());
    }
}
