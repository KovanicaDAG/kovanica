#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use kovanica_common::events::{NFT_APPROVAL, NFT_BURN, NFT_COLLECTION_CREATED, NFT_MINT, NFT_TRANSFER};
use kovanica_common::{ContractError, ContractResult, Host};
use kovanica_common::Address;

pub const STATE_KEY: &[u8] = b"kvnc/v1/state";
pub type CollectionId = [u8; 32];
pub type TokenId = u128;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TokenRecord { pub owner: Address, pub approved: Option<Address>, pub token_uri: String }

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NftState {
    pub name: String, pub symbol: String, pub base_uri: String,
    pub owner: Address, pub minter: Address,
    pub tokens: BTreeMap<TokenId, TokenRecord>,
}

impl NftState {
    pub fn load(host: &impl Host) -> ContractResult<Self> {
        let bytes = host.storage_get(STATE_KEY).ok_or(ContractError::NotFound)?;
        bincode::deserialize(&bytes).map_err(|_| ContractError::Custom(2))
    }
    fn store(&self, host: &mut impl Host) -> ContractResult<()> {
        let bytes = bincode::serialize(self).map_err(|_| ContractError::Custom(1))?;
        host.storage_set(STATE_KEY, &bytes);
        Ok(())
    }
}

pub struct Nft;
impl Nft {
    pub fn create_collection(host: &mut impl Host, name: String, symbol: String, base_uri: String) -> ContractResult<CollectionId> {
        if host.storage_get(STATE_KEY).is_some() { return Err(ContractError::AlreadyExists); }
        let caller = host.caller();
        let state = NftState { name, symbol, base_uri, owner: caller, minter: caller, tokens: BTreeMap::new() };
        state.store(host)?;
        host.emit_event(NFT_COLLECTION_CREATED, &[]);
        Ok(host.contract_address())
    }
    fn load(host: &impl Host) -> ContractResult<NftState> { NftState::load(host) }
    fn save(host: &mut impl Host, state: &NftState) -> ContractResult<()> { state.store(host) }

    pub fn mint(host: &mut impl Host, _c: CollectionId, to: Address, token_id: TokenId, token_uri: String) -> ContractResult<()> {
        let mut state = Self::load(host)?;
        let caller = host.caller();
        if caller != state.owner && caller != state.minter { return Err(ContractError::Unauthorized); }
        if state.tokens.contains_key(&token_id) { return Err(ContractError::AlreadyExists); }
        state.tokens.insert(token_id, TokenRecord { owner: to, approved: None, token_uri });
        Self::save(host, &state)?;
        let mut data = [0u8; 32];
        data[..16].copy_from_slice(&token_id.to_le_bytes());
        host.emit_event(NFT_MINT, &data);
        let _ = to;
        Ok(())
    }

    pub fn transfer(host: &mut impl Host, _c: CollectionId, to: Address, token_id: TokenId) -> ContractResult<()> {
        let mut state = Self::load(host)?;
        let caller = host.caller();
        let rec = state.tokens.get(&token_id).ok_or(ContractError::NotFound)?;
        if rec.owner != caller { return Err(ContractError::Unauthorized); }
        let mut rec = rec.clone();
        rec.owner = to; rec.approved = None;
        state.tokens.insert(token_id, rec);
        Self::save(host, &state)?;
        let mut data = [0u8; 32];
        data[..16].copy_from_slice(&token_id.to_le_bytes());
        host.emit_event(NFT_TRANSFER, &data);
        Ok(())
    }
    pub fn approve(host: &mut impl Host, _c: CollectionId, spender: Address, token_id: TokenId) -> ContractResult<()> {
        let mut state = Self::load(host)?;
        let caller = host.caller();
        let rec = state.tokens.get(&token_id).ok_or(ContractError::NotFound)?;
        if rec.owner != caller { return Err(ContractError::Unauthorized); }
        let mut rec = rec.clone();
        rec.approved = Some(spender);
        state.tokens.insert(token_id, rec);
        Self::save(host, &state)?;
        let mut data = [0u8; 32];
        data[..16].copy_from_slice(&token_id.to_le_bytes());
        host.emit_event(NFT_APPROVAL, &data);
        Ok(())
    }

    pub fn transfer_from(host: &mut impl Host, _c: CollectionId, from: Address, to: Address, token_id: TokenId) -> ContractResult<()> {
        let mut state = Self::load(host)?;
        let caller = host.caller();
        let rec = state.tokens.get(&token_id).ok_or(ContractError::NotFound)?;
        if rec.owner != from { return Err(ContractError::InvalidInput); }
        let approved = rec.approved;
        if caller != from && approved != Some(caller) { return Err(ContractError::Unauthorized); }
        let mut rec = rec.clone();
        rec.owner = to; rec.approved = None;
        state.tokens.insert(token_id, rec);
        Self::save(host, &state)?;
        let mut data = [0u8; 32];
        data[..16].copy_from_slice(&token_id.to_le_bytes());
        host.emit_event(NFT_TRANSFER, &data);
        Ok(())
    }

    pub fn set_base_uri(host: &mut impl Host, _c: CollectionId, base_uri: String) -> ContractResult<()> {
        let mut state = Self::load(host)?;
        let caller = host.caller();
        if caller != state.owner { return Err(ContractError::Unauthorized); }
        state.base_uri = base_uri;
        Self::save(host, &state)?;
        Ok(())
    }

    pub fn burn(host: &mut impl Host, _c: CollectionId, token_id: TokenId) -> ContractResult<()> {
        let mut state = Self::load(host)?;
        let caller = host.caller();
        let rec = state.tokens.get(&token_id).ok_or(ContractError::NotFound)?;
        if rec.owner != caller { return Err(ContractError::Unauthorized); }
        state.tokens.remove(&token_id);
        Self::save(host, &state)?;
        let mut data = [0u8; 32];
        data[..16].copy_from_slice(&token_id.to_le_bytes());
        host.emit_event(NFT_BURN, &data);
        Ok(())
    }

    pub fn owner_of(host: &impl Host, token_id: TokenId) -> ContractResult<Address> {
        let state = Self::load(host)?;
        let rec = state.tokens.get(&token_id).ok_or(ContractError::NotFound)?;
        Ok(rec.owner)
    }

    pub fn state_slot() -> [u8; 32] { kovanica_common::hash(STATE_KEY) }

    pub fn decode_state(bytes: &[u8]) -> Option<NftState> { bincode::deserialize(bytes).ok() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_common::MemHost;
    fn addr(n: u8) -> Address { let mut a = [0u8;32]; a[0]=n; a }
    fn new_collection(owner: u8) -> (MemHost, CollectionId) {
        let contract = addr(99);
        let mut host = MemHost::new(addr(owner), contract);
        let cid = Nft::create_collection(&mut host, "Test".into(), "TST".into(), "ipfs://x/".into()).unwrap();
        (host, cid)
    }
    #[test]
    fn create_collection_works() {
        let (host, cid) = new_collection(1);
        let st = Nft::decode_state(&host.storage_get(STATE_KEY).unwrap()).unwrap();
        assert_eq!(st.name, "Test");
        assert_eq!(cid, addr(99));
    }
    #[test]
    fn mint_and_transfer() {
        let (mut host, cid) = new_collection(1);
        Nft::mint(&mut host, cid, addr(2), 1, "1.json".into()).unwrap();
        assert_eq!(Nft::owner_of(&host, 1).unwrap(), addr(2));
        host.set_caller(addr(2));
        Nft::transfer(&mut host, cid, addr(3), 1).unwrap();
        assert_eq!(Nft::owner_of(&host, 1).unwrap(), addr(3));
    }
    #[test]
    fn approve_and_transfer_from() {
        let (mut host, cid) = new_collection(1);
        Nft::mint(&mut host, cid, addr(2), 1, "1.json".into()).unwrap();
        host.set_caller(addr(2));
        Nft::approve(&mut host, cid, addr(4), 1).unwrap();
        host.set_caller(addr(4));
        Nft::transfer_from(&mut host, cid, addr(2), addr(5), 1).unwrap();
        assert_eq!(Nft::owner_of(&host, 1).unwrap(), addr(5));
    }
    #[test]
    fn unauthorized_mint() {
        let (mut host, cid) = new_collection(1);
        host.set_caller(addr(7));
        assert_eq!(Nft::mint(&mut host, cid, addr(2), 1, "x".into()), Err(ContractError::Unauthorized));
    }
    #[test]
    fn burn() {
        let (mut host, cid) = new_collection(1);
        Nft::mint(&mut host, cid, addr(2), 1, "1.json".into()).unwrap();
        host.set_caller(addr(2));
        Nft::burn(&mut host, cid, 1).unwrap();
        assert_eq!(Nft::owner_of(&host, 1), Err(ContractError::NotFound));
    }
}

