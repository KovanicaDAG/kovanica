//! Transaction builders and signing helpers.

use kovanica_crypto::sign;
use kovanica_types::crypto::SigningKey;
use kovanica_types::signing::SigningContext;
use kovanica_types::transaction::{Transaction, TransactionKind};
use kovanica_types::{Address, Hash, Signature};

pub fn new_unsigned(sender: Address, kind: TransactionKind, nonce: u64, fee: u64) -> Transaction {
    Transaction {
        sender,
        nonce,
        kind,
        fee,
        signature: Signature([0u8; 64]),
        hash: Hash([0u8; 32]),
    }
}

pub fn sign_tx(tx: &mut Transaction, key: &SigningKey, ctx: &SigningContext) {
    tx.hash = tx.signing_hash(ctx);
    tx.signature = sign(key, &tx.hash.0);
}

pub fn build(
    key: &SigningKey,
    ctx: &SigningContext,
    kind: TransactionKind,
    nonce: u64,
    fee: u64,
) -> Transaction {
    let sender = kovanica_types::Address(key.verifying_key().to_bytes());
    let mut tx = new_unsigned(sender, kind, nonce, fee);
    sign_tx(&mut tx, key, ctx);
    tx
}

// Convenience builders
pub fn transfer(
    key: &SigningKey,
    ctx: &SigningContext,
    to: Address,
    amount: u64,
    nonce: u64,
    fee: u64,
) -> Transaction {
    build(
        key,
        ctx,
        TransactionKind::Transfer { to, amount },
        nonce,
        fee,
    )
}
pub fn stake(
    key: &SigningKey,
    ctx: &SigningContext,
    amount: u64,
    nonce: u64,
    fee: u64,
) -> Transaction {
    build(key, ctx, TransactionKind::Stake { amount }, nonce, fee)
}
pub fn unstake(
    key: &SigningKey,
    ctx: &SigningContext,
    amount: u64,
    nonce: u64,
    fee: u64,
) -> Transaction {
    build(key, ctx, TransactionKind::Unstake { amount }, nonce, fee)
}
pub fn deploy(
    key: &SigningKey,
    ctx: &SigningContext,
    code: Vec<u8>,
    nonce: u64,
    fee: u64,
) -> Transaction {
    build(key, ctx, TransactionKind::Deploy { code }, nonce, fee)
}
#[allow(clippy::too_many_arguments)]
pub fn call(
    key: &SigningKey,
    ctx: &SigningContext,
    contract: Address,
    value: u64,
    method: String,
    args: Vec<u8>,
    gas_limit: u64,
    nonce: u64,
    fee: u64,
) -> Transaction {
    build(
        key,
        ctx,
        TransactionKind::Call {
            contract,
            value,
            method,
            args,
            gas_limit,
        },
        nonce,
        fee,
    )
}
pub fn evidence(
    key: &SigningKey,
    ctx: &SigningContext,
    evidence: kovanica_types::EquivocationEvidence,
    nonce: u64,
    fee: u64,
) -> Transaction {
    build(key, ctx, TransactionKind::Evidence { evidence }, nonce, fee)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_types::signing::SigningContext;

    const CTX: SigningContext = SigningContext::new(4);

    #[test]
    fn transfer_signs_and_verifies() {
        let key = kovanica_crypto::generate_keypair().0;
        let to = kovanica_types::Address([2u8; 32]);
        let tx = transfer(&key, &CTX, to, 100, 0, 10);
        assert!(tx.verify_signature(&CTX));
    }

    #[test]
    fn bincode_roundtrip() {
        let key = kovanica_crypto::generate_keypair().0;
        let to = kovanica_types::Address([2u8; 32]);
        let tx = transfer(&key, &CTX, to, 100, 0, 10);
        let bytes = bincode::serialize(&tx).unwrap();
        let dec: Transaction = bincode::deserialize(&bytes).unwrap();
        assert_eq!(tx.hash, dec.hash);
        assert_eq!(tx.signature, dec.signature);
    }
}
