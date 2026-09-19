---
name: rust-substrate
description: Use when building on Substrate: FRAME pallets, runtime construction, `sprite`/stake, balances, governance, custom pallets, Wasm runtime, weight / fees, and common Substrate patterns.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, blockchain, substrate, FRAME, pallet, runtime, wasm, weight, fees, balances, staking, governance]
    related_skills: [rust-smart-contracts, rust-consensus-pos, rust-crypto, rust-merkle-structures]
---

# Rust Substrate

## Overview

Substrate is a blockchain framework by Parity that lets you build custom blockchains with a modular architecture. The core concepts are:

- **Runtime:** the state transition function, compiled to Wasm.
- **FRAME:** a set of pallets (modules) that provide common functionality (balances, staking, governance, etc.).
- **Pallets:** individual modules that you compose to build a runtime. You can use existing pallets or write custom ones.
- **Wasm runtime:** the runtime is Wasm, allowing forkless upgrades (the runtime can be upgraded by a governance proposal).

This skill covers Substrate basics, FRAME pallets, custom pallets, and common patterns. Substrate is a large framework — this is an orientation.

## When to Use

- Building a custom blockchain with Substrate
- Writing a FRAME pallet for custom logic
- Configuring a runtime with existing pallets
- Understanding Substrate's account model, storage, and extrinsics
- Designing weight/fees for a Substrate chain
- Using Substrate's consensus, networking, and client stack

**Don't use for:** simple smart contract deployment (use ink! for that), or building a chain without wanting Substrate's architecture. Substrate is a full framework — if you just need a contract, ink! is lighter.

## Substrate Architecture

```
┌─────────────┐    ┌─────────────┐    ┌──────────────────┐
│   Client    │───▶│  Runtime    │───▶│  FRAME Pallets   │
│ (polkadot-  │    │ (Wasm)      │    │ (balances,       │
│  substrate) │    │             │    │  staking, etc.)   │
└─────────────┘    └─────────────┘    └──────────────────┘
      │                   │
      ▼                   ▼
  Networking          Storage
  (p2p)               (trie)
```

- **Client:** the node software (networking, consensus, storage, RPC).
- **Runtime:** Wasm bytecode that defines the state transition function.
- **FRAME pallets:** Rust modules compiled into the runtime.

## FRAME Pallets (Existing)

Substrate ships with many pallets:

| Pallet | Purpose |
|---|---|
| `pallet_balances` | Account balances, transfers |
| `pallet_staking` | Stake, validators, rewards |
| `pallet_session` | Session management, validator sets |
| `pallet_authorship` | Block authorship (slot authorship) |
| `pallet_grandpa` | GRANDPA finalized consensus (PoS) |
| `pallet_babe` | BABE block production (PoS, slot-based) |
| `pallet_im_online` | Validator online attestation |
| `pallet_offences` | Slashing for offences |
| `pallet_treasury` | Treasury funds, tips |
| `pallet_society` | Ring-signing, membership |
| `pallet_collator-selection` | Collator selection (for parachains) |
| `pallet_governance` (various) | Democracy, council, technical committee, referenda |
| `pallet_filters` | Filter dispatchables by account |
| `pallet_preimage` | Preimage funding |
| `pallet_proxy` | Proxy accounts, delegate calls |
| `pallet_recovery` | Account recovery |
| `pallet_vesting` | Vesting schedules |

These pallets are composable — you select which ones to include in your runtime.

## Writing a Custom Pallet

A custom pallet is a Rust module that uses the `frame_support` and `frame_system` crates.

```toml
[pallet]
name = "my_pallet"
version = "1.0.0"
authors = ["You"]
homepage = "https://example.com"

[dependencies]
frame-support = { version = "10", default-features = false }
frame-system = { version = "10", default-features = false }
sp-std = "8"
sp-weights = "8"

[features]
default = ["std"]
std = [
    "frame-support/std",
    "frame-system/std",
    "sp-std/std",
    "sp-weights/std",
]
```

### Minimal Pallet

```rust
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_module, decl_storage, decl_event, decl_error, pallet_prelude::*,
    traits::Randomness,
};
use frame_system::pallet_prelude::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + sp_std::ok::unwrap::Into<u32> {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    pub type MyValue<T: Config> = StorageMap<_, Blake2_128Truncator, u32, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ValueSet(u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidValue,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(<T as Config>::WeightInfo::set_value())]
        pub fn set_value(origin: OriginFor<T>, key: u32, value: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(value > 0, Error::<T>::InvalidValue)?;

            MyValue::<T>::insert(&key, value);
            Self::deposit_event(Event::ValueSet(key, value));
            Ok(())
        }
    }
}
```

**Key macros:**
- `#[frame_support::pallet]` — marks the module as a pallet.
- `#[pallet::config]` — pallet configuration trait.
- `#[pallet::storage]` — storage items (persistent state).
- `#[pallet::event]` — events emitted by the pallet.
- `#[pallet::error]` — errors returned by dispatchables.
- `#[pallet::call]` — dispatchable functions (extrinsics).

### Pallet Configuration

The `Config` trait specifies what the pallet needs from the runtime:

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    type RuntimeEvent: ...;
    type WeightInfo: WeightInfo;
    type MyParameter: Get<u32>;   // custom associated type
}
```

When including the pallet in the runtime, you implement `Config` with the concrete types.

### Storage

```rust
// StorageMap: key-value map
#[pallet::storage]
pub type Balances<T: Config> = StorageMap<_, Blake2_128Truncator, T::AccountId, u128, ValueQuery>;

// StorageValue: single value
#[pallet::storage]
pub type TotalSupply<T: Config> = StorageValue<_, u128, ValueQuery>;

// StorageDoubleMap: two-key map
#[pallet::storage]
pub type UserScores<T: Config> = StorageDoubleMap<_, Blake2_128Truncator, T::AccountId, u32, u32, ValueQuery>;
```

Storage is persistent across blocks. It's accessed via the `pallet::storage` macros and is part of the runtime's state trie.

### Events

```rust
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    Transfer(T::AccountId, T::AccountId, u128),
    SomethingHappened(u32),
}
```

Events are emitted with `Self::deposit_event(Event::...)` and are stored in the block's event record. Indexers and light clients can query events.

### Errors

```rust
#[pallet::error]
pub enum Error<T> {
    InsufficientBalance,
    NotAuthorized,
    Overflow,
}
```

Errors are returned from dispatchables with `Err(Error::<T>::InsufficientBalance.into())`.

### Dispatchables (Call)

```rust
#[pallet::call]
impl<T: Config> Pallet<T> {
    #[pallet::weight(100)]
    pub fn transfer(origin: OriginFor<T>, to: T::AccountId, amount: u128) -> DispatchResult {
        let from = ensure_signed(origin)?;

        let from_balance = Balances::<T>::get(&from);
        ensure!(from_balance >= amount, Error::<T>::InsufficientBalance)?;

        Balances::<T>::insert(&from, from_balance - amount);
        Balances::<T>::mutate(&to, |b| *b += amount);

        Self::deposit_event(Event::Transfer(from, to, amount));
        Ok(())
    }
}
```

## Runtime Construction

The runtime is a Rust crate that assembles pallets and implements the runtime API.

```rust
// runtime/src/lib.rs (conceptual)
#![no_std]
#![excluded_register_panic_handler]

pub use pallet_balances;
pub use pallet_staking;
pub use pallet_session;
pub use pallet_grandpa;
pub use pallet_my_pallet;

use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstantZero, PalletInfo, WeightInfo},
};
use frame_system::limits::{BlockLength, BlockWeights};
use sp_runtime::{create_runtime_str, generic, impl_opaque_keys};

// Parameter types
parameter_types! {
    pub const BlockHashCount: u64 = 2400;
    pub const Version: u32 = 1;
    pub const SS58Prefix: u8 = 42;
}

// Block weights
parameter_types! {
    pub const MaximumBlockWeight: Weight = 2_000_000_000;
    pub const MaximumBlockLength: u32 = 500000 * 1024;   // 500 KB
}

impl frame_system::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type BlockHashCount = BlockHashCount;
    type Version = Version;
    type SS58Prefix = SS58Prefix;
    type BlockWeights = BlockWeights;
    type BlockLength = BlockLength;
    // ...
}

// Construct the runtime
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Randomness: pallet_randomness_collective_flip,
        Timestamp: pallet_timestamp,
        AuthorityLookup: pallet_authority_lookup,
        balances: pallet_balances,
        transactionPayment: pallet_transaction_payment,
        authorship: pallet_authorship,
        session: pallet_session,
        grandpa: pallet_grandpa,
        my_pal: pallet_my_pallet,
    }
);
```

The `construct_runtime!` macro assembles the pallets into a runtime. The runtime is compiled to Wasm and deployed to nodes.

## Weight and Fees

Substrate uses a weight system: every dispatchable has a weight (estimated computational cost). Fees are calculated from weight × price per unit weight.

```rust
use frame_support::weights::{constants::{EraImplementations, BlockExecutionWeight, ExtrinsicBaseWeight}, Perbill};

parameter_types! {
    pub const TransactionByteFee: u128 = 1_000;   // per byte
    pub const WeightToFee: u128 = 1;              // weight unit to token conversion
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightToFee = WeightToFeePolynomial;
    type OnChargeTransaction = ...;
    type FeeMultiplierDelta = ...;
}
```

**Weight:** estimated execution time (in weight units). The weight should reflect the actual computational cost (storage reads/writes, computation).

**Fee:** weight × price + byte fee. The fee is paid by the extrinsic caller.

## Consensus in Substrate

Substrate supports pluggable consensus. Common choices:

- **GRANDPA + BABE:** probabilistic block production (BABE) + deterministic finality (GRANDPA). PoS-style.
- **Sha3 PoW:** proof-of-work consensus (legacy).
- **HotStuff / other:** custom consensus.

For PoS chains, BABE produces blocks, GRANDPA finalizes them.

```rust
use pallet_grandpa::Config;
use pallet_babe::Config;

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type KeyOwnerProof = ...;
    type Call = Call;
    type grader = ...;
    // ...
}

impl pallet_babe::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type EpochConfiguration = EpochConfig;
    type AuthorId = ValidatorId;
    // ...
}
```

## Storage and Trie

Substrate uses a Merkle trie for runtime storage. Each pallet's storage is part of the trie. The trie root is included in the block header, allowing state proofs.

```rust
// Storage is accessed via the macros — it's all in the trie
// State proofs can be generated for storage items
// Light clients verify proofs against the trie root in the header
```

## Forkless Upgrades

Because the runtime is Wasm, it can be upgraded without a hard fork. A governance proposal can replace the runtime Wasm code.

```rust
// Runtime upgrade via pallet_upgrades or governance
// The new Wasm runtime is submitted, verified, and scheduled
// Nodes upgrade automatically when the new runtime is finalized
```

## Verification Checklist

- [ ] Can describe Substrate's architecture: client, runtime (Wasm), FRAME pallets
- [ ] Can write a minimal custom pallet with storage, events, errors, and a dispatchable
- [ ] Can explain the `Config` trait and how a pallet is configured in the runtime
- [ ] Can explain Substrate's storage model (trie-based, persistent)
- [ ] Can explain weight and fees in Substrate
- [ ] Can describe GRANDPA + BABE as a consensus option
- [ ] Can describe forkless runtime upgrades
- [ ] Can include an existing FRAME pallet in a runtime (conceptually)
- [ ] Understands that Substrate pallets are compiled into the Wasm runtime

## Common Pitfalls

1. **Using `unwrap()` in pallet code.** Pallets should return `DispatchResult` and handle errors gracefully. Panicking in a pallet can stall the chain.

2. **Not setting correct weights.** Weights affect fees and block capacity. Underestimated weights lead to overuse; overestimated weights lead to high fees.

3. **Not handling storage reads/writes efficiently.** Storage is expensive (trie operations). Minimize storage accesses in hot paths.

4. **Storing large data in pallet storage.** Storage has costs and limits. Don't store large blobs; use off-chain storage or reference data.

5. **Not considering upgradeability.** If a pallet's storage format changes, it may break existing state. Plan for migrations (`on_runtime_upgrade`).

6. **Using `frame_system::ensure_signed` incorrectly.** `ensure_signed` extracts the caller from the origin. It returns an error if the origin is not a signed account (e.g., root, pallet).

7. **Not implementing `OnTimestampSet` or similar traits correctly.** Some pallets require specific trait implementations. Read the pallet docs.

8. **Assuming Wasm runtime execution is fast.** Wasm execution has overhead. Benchmark pallet dispatchables with `frame_benchmarking` to get accurate weights.

9. **Not handling `Root` origin properly.** The `Root` origin (passed by governance or certain pallets) has special privileges. Don't assume all origins are signed accounts.

10. **Not testing pallets with `frame_support::test`.** Substrate provides test utilities for pallets. Use them to test storage, events, and dispatchables.
