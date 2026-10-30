//! Generic multi-party escrow for kovanica (complement to HTLC/vault).

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;

use kovanica_common::events::{ESCROW_APPROVED, ESCROW_CREATED, ESCROW_REFUNDED, ESCROW_RELEASED};
use kovanica_common::{hash, Address, Amount, ContractError, ContractResult, Host, Timestamp};

pub type EscrowId = [u8; 32];

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Escrow {
    pub creator: Address,
    pub parties: Vec<Address>,
    pub threshold: u8,
    pub approvals: Vec<Address>,
    pub amount: Amount,
    pub deadline: Timestamp,
    pub released: bool,
    pub refunded: bool,
}

fn store_key(id: &EscrowId) -> Vec<u8> {
    let mut k = Vec::with_capacity(16 + 32);
    k.extend_from_slice(b"kvnc/v1/escrow/");
    k.extend_from_slice(id);
    k
}

fn escrow_address(id: &EscrowId) -> Address {
    let mut buf = Vec::with_capacity(14 + 32);
    buf.extend_from_slice(b"escrow/escrow");
    buf.extend_from_slice(id);
    hash(&buf)
}

fn derive_id(
    creator: &Address,
    parties: &[Address],
    threshold: u8,
    amount: Amount,
    deadline: Timestamp,
) -> EscrowId {
    let mut parties_sorted = parties.to_vec();
    parties_sorted.sort();
    let mut buf = Vec::new();
    buf.extend_from_slice(b"escrow/id");
    buf.extend_from_slice(creator);
    for p in &parties_sorted {
        buf.extend_from_slice(p);
    }
    buf.push(threshold);
    buf.extend_from_slice(&amount.to_le_bytes());
    buf.extend_from_slice(&deadline.to_le_bytes());
    hash(&buf)
}

fn load(host: &impl Host, id: &EscrowId) -> ContractResult<Escrow> {
    let key = store_key(id);
    let bytes = host.storage_get(&key).ok_or(ContractError::NotFound)?;
    bincode::deserialize(&bytes).map_err(|_| ContractError::InvalidInput)
}

fn store(host: &mut impl Host, id: &EscrowId, e: &Escrow) -> ContractResult<()> {
    let key = store_key(id);
    let bytes = bincode::serialize(e).map_err(|_| ContractError::InvalidInput)?;
    host.storage_set(&key, &bytes);
    Ok(())
}

pub fn create(
    host: &mut impl Host,
    parties: Vec<Address>,
    threshold: u8,
    amount: Amount,
    deadline: Timestamp,
) -> ContractResult<EscrowId> {
    if parties.is_empty() || threshold == 0 || threshold as usize > parties.len() || amount == 0 {
        return Err(ContractError::InvalidInput);
    }
    let mut uniq = parties.clone();
    uniq.sort();
    uniq.dedup();
    if uniq.len() != parties.len() {
        return Err(ContractError::InvalidInput);
    }

    let creator = host.caller();
    let id = derive_id(&creator, &uniq, threshold, amount, deadline);
    let key = store_key(&id);
    if host.storage_get(&key).is_some() {
        return Err(ContractError::AlreadyExists);
    }

    let escrow_addr = escrow_address(&id);
    host.transfer(&creator, &escrow_addr, amount)?;

    let e = Escrow {
        creator,
        parties: uniq,
        threshold,
        approvals: Vec::new(),
        amount,
        deadline,
        released: false,
        refunded: false,
    };
    store(host, &id, &e)?;
    host.emit_event(ESCROW_CREATED, &id);
    Ok(id)
}

pub fn approve(host: &mut impl Host, id: EscrowId) -> ContractResult<()> {
    let caller = host.caller();
    let mut e = load(host, &id)?;
    if e.released || e.refunded {
        return Err(ContractError::InvalidInput);
    }
    if !e.parties.contains(&caller) {
        return Err(ContractError::Unauthorized);
    }
    if e.approvals.contains(&caller) {
        return Err(ContractError::AlreadyExists);
    }
    e.approvals.push(caller);
    store(host, &id, &e)?;
    host.emit_event(ESCROW_APPROVED, &id);
    Ok(())
}

pub fn release(host: &mut impl Host, id: EscrowId, to: Address) -> ContractResult<()> {
    let caller = host.caller();
    let mut e = load(host, &id)?;
    if e.released || e.refunded {
        return Err(ContractError::InvalidInput);
    }
    if e.approvals.len() < e.threshold as usize {
        return Err(ContractError::ThresholdNotMet);
    }
    if caller != e.creator && !e.parties.contains(&caller) {
        return Err(ContractError::Unauthorized);
    }
    let escrow_addr = escrow_address(&id);
    host.transfer(&escrow_addr, &to, e.amount)?;
    e.released = true;
    store(host, &id, &e)?;
    host.emit_event(ESCROW_RELEASED, &id);
    Ok(())
}

pub fn refund(host: &mut impl Host, id: EscrowId) -> ContractResult<()> {
    let mut e = load(host, &id)?;
    if e.released || e.refunded {
        return Err(ContractError::InvalidInput);
    }
    if host.timestamp() < e.deadline {
        return Err(ContractError::NotExpired);
    }
    let escrow_addr = escrow_address(&id);
    host.transfer(&escrow_addr, &e.creator, e.amount)?;
    e.refunded = true;
    store(host, &id, &e)?;
    host.emit_event(ESCROW_REFUNDED, &id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_common::MemHost;
    use kovanica_common::{Address, Amount, ContractError, Timestamp};

    fn addr(n: u8) -> Address {
        let mut a = [0u8; 32];
        a[0] = n;
        a
    }
    fn funded_host(sender: Address, contract: Address, balance: Amount, ts: Timestamp) -> MemHost {
        let mut h = MemHost::new(sender, contract);
        h.set_balance(sender, balance);
        h.set_timestamp(ts);
        h
    }

    #[test]
    fn create_escrows_and_refund_after_deadline() {
        let c = addr(9);
        let mut host = funded_host(addr(1), c, 1_000, 10);
        let parties = vec![addr(2), addr(3)];
        let id = create(&mut host, parties.clone(), 2, 300, 100).unwrap();
        assert_eq!(host.balance_of(&addr(1)), 700);
        let esc = escrow_address(&id);
        assert_eq!(host.balance_of(&esc), 300);

        host.set_caller(addr(2));
        approve(&mut host, id).unwrap();
        host.set_caller(addr(3));
        approve(&mut host, id).unwrap();

        host.set_timestamp(50);
        host.set_caller(addr(1));
        assert_eq!(refund(&mut host, id), Err(ContractError::NotExpired));

        host.set_timestamp(100);
        refund(&mut host, id).unwrap();
        assert_eq!(host.balance_of(&addr(1)), 1_000);
    }

    #[test]
    fn release_when_threshold_met() {
        let c = addr(9);
        let mut host = funded_host(addr(1), c, 1_000, 10);
        let parties = vec![addr(2), addr(3)];
        let id = create(&mut host, parties, 1, 200, 100).unwrap();

        host.set_caller(addr(2));
        approve(&mut host, id).unwrap();

        release(&mut host, id, addr(4)).unwrap();
        assert_eq!(host.balance_of(&addr(4)), 200);
    }

    #[test]
    fn threshold_not_met_blocks_release() {
        let c = addr(9);
        let mut host = funded_host(addr(1), c, 1_000, 10);
        let parties = vec![addr(2), addr(3)];
        let id = create(&mut host, parties, 2, 200, 100).unwrap();

        host.set_caller(addr(2));
        approve(&mut host, id).unwrap();

        host.set_caller(addr(1));
        assert_eq!(
            release(&mut host, id, addr(4)),
            Err(ContractError::ThresholdNotMet)
        );
    }

    #[test]
    fn duplicate_create_rejected() {
        let c = addr(9);
        let mut host = funded_host(addr(1), c, 1_000, 10);
        let parties = vec![addr(2)];
        let _ = create(&mut host, parties.clone(), 1, 100, 100).unwrap();
        host.set_balance(addr(1), 1_000);
        let res = create(&mut host, parties, 1, 100, 100);
        assert_eq!(res, Err(ContractError::AlreadyExists));
    }
}
