//! Bincode argument bytes for a native `TransactionKind::Call`.
//!
//! `method` must be one of [`crate::ENTRY_POINTS`]. `args` must be the bytes
//! from the matching `encode_*` function. That is the layout
//! [`crate::contracts`] decodes. It is not JSON, and it is not the body of
//! the off-chain `htlc_create` / `vault_create` / `token_create` RPC methods.
//!
//! These helpers do not submit anything. A caller builds a signed `Call`
//! and sends it to a node of their own choosing.

use kvnc_common::{Amount, Timestamp};
use kvnc_vault::VestingSchedule;
use serde::Serialize;

/// Failure while encoding call arguments. The inputs are plain data, so this
/// only fires if bincode itself rejects the value.
#[derive(Debug)]
pub struct EncodeError(bincode::Error);

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "contract call argument encode failed: {}", self.0)
    }
}

impl std::error::Error for EncodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

fn enc<T: Serialize>(value: &T) -> Result<Vec<u8>, EncodeError> {
    bincode::serialize(value).map_err(EncodeError)
}

/// `htlc_create(claimer, amount, hash_lock, expiry)`.
pub fn encode_htlc_create(
    claimer: [u8; 32],
    amount: Amount,
    hash_lock: [u8; 32],
    expiry: Timestamp,
) -> Result<Vec<u8>, EncodeError> {
    enc(&(claimer, amount, hash_lock, expiry))
}

/// `htlc_claim(id, preimage)`.
pub fn encode_htlc_claim(id: [u8; 32], preimage: &[u8]) -> Result<Vec<u8>, EncodeError> {
    enc(&(id, preimage.to_vec()))
}

/// `htlc_refund(id)`.
pub fn encode_htlc_refund(id: [u8; 32]) -> Result<Vec<u8>, EncodeError> {
    enc(&id)
}

/// `vault_create(beneficiary, amount, schedule)`.
pub fn encode_vault_create(
    beneficiary: [u8; 32],
    amount: Amount,
    schedule: &VestingSchedule,
) -> Result<Vec<u8>, EncodeError> {
    enc(&(beneficiary, amount, schedule))
}

/// `vault_claim(id)`.
pub fn encode_vault_claim(id: [u8; 32]) -> Result<Vec<u8>, EncodeError> {
    enc(&id)
}

/// `vault_cancel(id)`.
pub fn encode_vault_cancel(id: [u8; 32]) -> Result<Vec<u8>, EncodeError> {
    enc(&id)
}

/// `multisig_create(owners, threshold)`.
pub fn encode_multisig_create(owners: &[[u8; 32]], threshold: u32) -> Result<Vec<u8>, EncodeError> {
    enc(&(owners.to_vec(), threshold))
}

/// `multisig_propose(id, to, amount, data)`.
pub fn encode_multisig_propose(
    id: [u8; 32],
    to: [u8; 32],
    amount: Amount,
    data: &[u8],
) -> Result<Vec<u8>, EncodeError> {
    enc(&(id, to, amount, data.to_vec()))
}

/// `multisig_confirm(id, tx_id)`.
pub fn encode_multisig_confirm(id: [u8; 32], tx_id: u64) -> Result<Vec<u8>, EncodeError> {
    enc(&(id, tx_id))
}

/// `multisig_execute(id, tx_id)`.
pub fn encode_multisig_execute(id: [u8; 32], tx_id: u64) -> Result<Vec<u8>, EncodeError> {
    enc(&(id, tx_id))
}

/// `token_create(name, symbol, decimals, initial_supply)`.
pub fn encode_token_create(
    name: &str,
    symbol: &str,
    decimals: u8,
    initial_supply: Amount,
) -> Result<Vec<u8>, EncodeError> {
    enc(&(
        name.to_string(),
        symbol.to_string(),
        decimals,
        initial_supply,
    ))
}

/// `token_transfer(to, amount)`.
pub fn encode_token_transfer(to: [u8; 32], amount: Amount) -> Result<Vec<u8>, EncodeError> {
    enc(&(to, amount))
}

/// `token_approve(spender, amount)`.
pub fn encode_token_approve(spender: [u8; 32], amount: Amount) -> Result<Vec<u8>, EncodeError> {
    enc(&(spender, amount))
}

/// `token_transfer_from(from, to, amount)`.
pub fn encode_token_transfer_from(
    from: [u8; 32],
    to: [u8; 32],
    amount: Amount,
) -> Result<Vec<u8>, EncodeError> {
    enc(&(from, to, amount))
}

/// `token_mint(to, amount)`.
pub fn encode_token_mint(to: [u8; 32], amount: Amount) -> Result<Vec<u8>, EncodeError> {
    enc(&(to, amount))
}

/// `token_burn(amount)`.
pub fn encode_token_burn(amount: Amount) -> Result<Vec<u8>, EncodeError> {
    enc(&amount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::execute_contract_call;
    use crate::ExecutionError;
    use kvnc_common::ContractError;
    use kvnc_storage::Storage;
    use kvnc_types::Address;
    use kvnc_vault::VestingSchedule;

    fn addr(tag: u8) -> Address {
        Address([tag; 32])
    }

    fn deploy(storage: &Storage, address: &Address) {
        let txn = storage.begin_write().expect("write");
        let mut account = storage
            .state()
            .get_account_or_default_write(&txn, address)
            .expect("account");
        account.code = b"\0asm-test-contract".to_vec();
        account.code_hash = kvnc_common::hash(&account.code);
        storage
            .state()
            .set_account(&txn, address, &account)
            .expect("set");
        txn.commit().expect("commit");
    }

    #[test]
    fn encoders_round_trip_the_dispatcher_layout() {
        let id = [7u8; 32];
        let other = [9u8; 32];
        let schedule = VestingSchedule::Absolute { unlock_at: 50 };

        let htlc_create = encode_htlc_create(other, 10, id, 99).unwrap();
        let decoded: ([u8; 32], Amount, [u8; 32], Timestamp) =
            bincode::deserialize(&htlc_create).unwrap();
        assert_eq!(decoded, (other, 10, id, 99));

        let claim = encode_htlc_claim(id, b"preimage").unwrap();
        let decoded: ([u8; 32], Vec<u8>) = bincode::deserialize(&claim).unwrap();
        assert_eq!(decoded, (id, b"preimage".to_vec()));

        assert_eq!(
            bincode::deserialize::<[u8; 32]>(&encode_htlc_refund(id).unwrap()).unwrap(),
            id
        );

        let vault = encode_vault_create(other, 5, &schedule).unwrap();
        let decoded: ([u8; 32], Amount, VestingSchedule) = bincode::deserialize(&vault).unwrap();
        assert_eq!(decoded.0, other);
        assert_eq!(decoded.1, 5);
        assert_eq!(decoded.2, schedule);

        let owners = vec![[1u8; 32], [2u8; 32]];
        let created = encode_multisig_create(&owners, 2).unwrap();
        let decoded: (Vec<[u8; 32]>, u32) = bincode::deserialize(&created).unwrap();
        assert_eq!(decoded, (owners, 2));

        let proposed = encode_multisig_propose(id, other, 3, b"data").unwrap();
        let decoded: ([u8; 32], [u8; 32], Amount, Vec<u8>) =
            bincode::deserialize(&proposed).unwrap();
        assert_eq!(decoded, (id, other, 3, b"data".to_vec()));

        assert_eq!(
            bincode::deserialize::<([u8; 32], u64)>(&encode_multisig_confirm(id, 4).unwrap())
                .unwrap(),
            (id, 4)
        );
        assert_eq!(
            bincode::deserialize::<([u8; 32], u64)>(&encode_multisig_execute(id, 4).unwrap())
                .unwrap(),
            (id, 4)
        );

        let token = encode_token_create("N", "S", 9, 1_000).unwrap();
        let decoded: (String, String, u8, Amount) = bincode::deserialize(&token).unwrap();
        assert_eq!(decoded, ("N".into(), "S".into(), 9, 1_000));

        assert_eq!(
            bincode::deserialize::<([u8; 32], Amount)>(&encode_token_transfer(other, 4).unwrap())
                .unwrap(),
            (other, 4)
        );
        assert_eq!(
            bincode::deserialize::<([u8; 32], Amount)>(&encode_token_approve(other, 4).unwrap())
                .unwrap(),
            (other, 4)
        );
        assert_eq!(
            bincode::deserialize::<([u8; 32], [u8; 32], Amount)>(
                &encode_token_transfer_from(id, other, 4).unwrap()
            )
            .unwrap(),
            (id, other, 4)
        );
        assert_eq!(
            bincode::deserialize::<([u8; 32], Amount)>(&encode_token_mint(other, 4).unwrap())
                .unwrap(),
            (other, 4)
        );
        assert_eq!(
            bincode::deserialize::<Amount>(&encode_token_burn(4).unwrap()).unwrap(),
            4
        );
        assert_eq!(
            bincode::deserialize::<[u8; 32]>(&encode_vault_claim(id).unwrap()).unwrap(),
            id
        );
        assert_eq!(
            bincode::deserialize::<[u8; 32]>(&encode_vault_cancel(id).unwrap()).unwrap(),
            id
        );
    }

    #[test]
    fn encoded_token_calls_execute_on_a_local_store() {
        let dir = tempfile::tempdir().expect("tempdir");
        let storage = Storage::new(dir.path().join("state.redb")).expect("storage");
        let contract = addr(30);
        let caller = addr(1);
        let recipient = [4u8; 32];
        deploy(&storage, &contract);

        let create = encode_token_create("Test", "TST", 9, 1_000).unwrap();
        execute_contract_call("token_create", contract, caller, 1, 1, &create, &storage)
            .expect("token_create");

        let transfer = encode_token_transfer(recipient, 400).unwrap();
        execute_contract_call(
            "token_transfer",
            contract,
            caller,
            2,
            2,
            &transfer,
            &storage,
        )
        .expect("token_transfer");

        let over = encode_token_transfer(recipient, 700).unwrap();
        let err = execute_contract_call("token_transfer", contract, caller, 3, 3, &over, &storage)
            .expect_err("balance is 600");
        assert!(
            matches!(
                err,
                ExecutionError::Contract(ContractError::InsufficientBalance)
            ),
            "{err:?}"
        );
    }
}
