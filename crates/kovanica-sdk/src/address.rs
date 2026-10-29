//! Address parsing helpers.

use crate::error::SdkError;
use kovanica_types::Address;

pub fn parse_address(value: &str, chain_id: u64) -> Result<Address, SdkError> {
    Address::parse_for_chain(value, chain_id).map_err(|e| SdkError::InvalidAddress(e.to_string()))
}

pub fn canonical(address: &Address) -> String {
    address.encode()
}
