//! Kovanica SDK: transaction builders, signing, address parsing, RPC client.

#![deny(unsafe_code)]

pub mod address;
pub mod error;
pub mod fee;
pub mod contracts;
pub mod tx;

#[cfg(feature = "keystore")]
pub mod keystore;

#[cfg(feature = "http")]
pub mod rpc;

pub use address::{canonical as canonical_address, parse_address};
pub use error::SdkError;
pub use fee::*;
pub use contracts::*;
pub use tx::*;

#[cfg(feature = "keystore")]
pub use keystore::signing_key_from_keystore;

#[cfg(feature = "http")]
pub use rpc::RpcClient;
