#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod storage;

pub use storage::{decode, encode, encode_error, storage_key};