---
name: rust-smart-contracts
description: Use when writing smart contracts in Rust: ink! for Substrate-based chains, CosmWasm for Cosmos-based chains, Solana program (via Anchor or native), and common patterns across contract frameworks.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, smart-contracts, ink, cosmwasm, solana, anchor, contracts, wasm]
    related_skills: [rust-wasm, rust-utxo-ledger, rust-account-ledger, rust-crypto]
---

# Rust Smart Contracts

## Overview

Smart contracts in Rust come in several frameworks, each tied to a specific chain ecosystem:

| Framework | Chain | Execution | Notes |
|---|---|---|---|
| **ink!** | Substrate/Polkadot (Wasm runtime) | Wasm on-chain | Rust-native, Substrate pallet integration |
| **CosmWasm** | Cosmos SDK chains (wasmvm) | Wasm on-chain | Rust contracts on Cosmos, IBC-native |
| **Solana (native + Anchor)** | Solana (msg! + BPF) | Native ELF (Solana runtime) | Anchor provides IDL, accounts, macros on top |
| **ewasm** (experimental) | Ethereum (Wasm proposal) | Wasm on EVM | Not mainnet-ready — skip unless researching |

Each framework has its own model for accounts/state, message dispatch, gas, and contract storage. Choose based on the target chain, not the language.

## When to Use

- Writing a contract for a Substrate-based chain (ink!)
- Writing a contract for a Cosmos SDK chain (CosmWasm)
- Writing a Solana program (native or via Anchor)
- Understanding contract storage, cross-contract calls, events, and upgrades
- Testing contracts locally (cargo-contract, wasmd, Solana test validator)

**Don't use for:** EVM contracts (Solidity/Vyper), or when the target chain doesn't support Wasm contracts.

## ink! — Substrate Contracts

```toml
[dependencies]
ink = { version = "4", default-features = false }
scale = { version = "3", default-features = false, features = ["full"] }
scale-info = "2"

[features]
std = [
    "ink/std",
    "scale/std",
    "scale-info/std",
]
```

ink! contracts compile to Wasm, run on Substrate's Wasm runtime, and use `scale` (SCALE codec) for serialization.

### Minimal ink! Contract

```rust
#![cfg_attr(not(feature = "std"), no_std)]

use ink::prelude::string::String;
use ink::env::call::{BuildCallable, Callable};
use ink::storage::Mapping;

#[ink::contract]
mod flipper {
    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        /// Constructor
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Flip the value
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Read the value
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }
}
```

**Key ink! attributes:**
- `#[ink::contract]` — marks the module as a contract
- `#[ink(storage)]` — marks the struct as contract storage (persisted across calls)
- `#[ink(constructor)]` — a constructor (called on contract deployment)
- `#[ink(message)]` — an exported message (callable from outside)

### Storage

ink! storage uses `Mapping` for key-value storage and `Lazy` for lazy-initialized values:

```rust
use ink::storage::{Mapping, Lazy};

#[ink(storage)]
pub struct MyContract {
    owner: Lazy<AccountId>,
    balances: Mapping<AccountId, Balance>,
    count: Lazy<u64>,
}
```

`Lazy<T>` initializes on first access (useful for values that start at default). `Mapping<K, V>` is a key-value store.

### Messages and Callbacks

```rust
#[ink(message)]
pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<(), Error> {
    let from = self.env().caller();
    let from_balance = self.balances.get(&from).unwrap_or(0);
    if from_balance < value {
        return Err(Error::InsufficientBalance);
    }
    self.balances.insert(&from, &(from_balance - value));
    self.balances.insert(&to, &(self.balances.get(&to).unwrap_or(0) + value));
    Ok(())
}
```

### Events

```rust
#[ink(event)]
pub struct Transfer {
    from: AccountId,
    to: AccountId,
    value: Balance,
}

// Emit in a message
self.env().emit_event(Transfer { from, to, value });
```

### Cross-Contract Calls

```rust
use ink::env::call::BuildCallable;

#[ink(message)]
pub fn call_other(&mut self, other: AccountId) {
    let other_contract = OtherContract::new(other);
    other_contract.some_message().call().unwrap();
}
```

### Testing (cargo-contract)

```bash
cargo install cargo-contract
cargo contract build   # builds Wasm + metadata
cargo contract test    # runs unit tests in a simulated Wasm environment
cargo contract instantiate --args ... -- 자선 --salary  # deploy
cargo contract call --args ...  # call a message
```

## CosmWasm — Cosmos Contracts

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
cosmwasm_std = "1.5"
cosmwasm_storage = "1.5"
schema_encoding = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
cosmwasm-schema = "1.5"
```

CosmWasm contracts run on `wasmvm` in Cosmos SDK chains. They use `cosmwasm_std` for the API (messages, storage, queries).

### Minimal CosmWasm Contract

```rust
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, StdResult,
    Response, StdError, Querier,
};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct State {
    pub count: i32,
}

pub const STATE: Item<State> = Item::new("state");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State { count: msg.count };
    STATE.save(deps.storage, &state)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Increment {} => execute_increment(deps, info),
    }
}

fn execute_increment(deps: DepsMut, _info: MessageInfo) -> StdResult<Response> {
    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.count += 1;
        Ok(state)
    })?;
    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => query_get_count(deps),
    }
}

fn query_get_count(deps: Deps) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    to_json_binary(&state)
}
```

**CosmWasm concepts:**
- `instantiate` — constructor, called on deployment with `InstantiateMsg`
- `execute` — state-mutating messages
- `query` — read-only queries
- `CosmWasmMsg` — messages to other contracts or native Cosmos modules
- `SubMsg` — sub-messages for composing multiple actions in one transaction

```rust
use cosmwasm_std::{CosmWasmMsg, StargateMsg, WasmMsg};

fn call_other_contract() -> StdResult<Response> {
    Ok(Response::new()
        .add_message(CosmWasmMsg::Execute {
            contract_addr: "other_contract_address".into(),
            msg: to_json_binary(&OtherMsg::DoSomething {})).unwrap(),
            funds: vec![],
        }))
}
```

### Interoperability with Cosmos (IBC)

CosmWasm contracts can send IBC packets:

```rust
use cosmwasm_std::IbcMsg;

fn send_ibc_packet() -> StdResult<Response> {
    Ok(Response::new()
        .add_message(IbcMsg::Transfer {
            source_channel: "channel-0".into(),
            source_port: "transfer".into(),
            sender: "address".into(),
            recipient: "cosmos1...".into(),
            amount: Uint128::new(1000000u128),
            timeout_height: None,
            timeout_timestamp: None,
        }))
}
```

## Solana — Native Programs and Anchor

### Native Solana Program

```rust
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    clan, entrypoint, entrypoint::ProgramResult,
    msg, pubkey::Pubkey,
    sysvar,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_info = &accounts[..];
    let account = next_account_info(accounts_info)?;

    msg!("Processing instruction");
    // ...
    Ok(())
}
```

Solana programs are ELF binaries loaded by the Solana runtime. They use `solana_program` for the on-chain API.

### Anchor Framework

```toml
[dependencies]
anchor-lang = "0.30"

[lib]
crate-type = ["cdylib", "lib"]
```

Anchor is the dominant framework for Solana development. It provides IDL (Interface Definition Language), account macros, error handling, and cross-program invocation helpers.

```rust
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTosaoBWtnG5QKoEz3YtsGuRqK");

#[program]
pub mod my_program {
    use anchor_lang::prelude::*;

    pub fn initialize(ctx: Context<Initialize>, data: u64) -> Result<()> {
        ctx.accounts.my_account.data = data;
        msg!("Initialized with data: {}", data);
        Ok(())
    }

    pub fn update(ctx: Context<Update>, new_data: u64) -> Result<()> {
        ctx.accounts.my_account.data = new_data;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8)]
    pub my_account: Account<'info, MyAccount>,
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub my_account: Account<'info, MyAccount>,
}

#[account]
pub struct MyAccount {
    pub data: u64,
}
```

**Anchor macros:**
- `#[program]` — marks the module, generates dispatch
- `#[derive(Accounts)]` — deserializes accounts from the transaction
- `#[account]` — defines an account type
- `init`, `mut`, `payer` — account constraints

**Anchor advantages:**
- IDL auto-generated — enables TypeScript/JS clients
- Account validation (types, signs, owner checks)
- Error handling with `anchor_lang::error`

### Cross-Program Invocation (CPI)

```rust
use anchor_lang::program::Invoke;

pub fn call_system_program(ctx: Context<'_, '_, '_, 'info, System>) -> Result<()> {
    // Anchor handles CPI via the Program type
}
```

## Testing Smart Contracts

### ink! Testing

```bash
cargo contract test   # runs unit tests in Wasm environment
cargo test            # runs Rust unit tests (with std feature)
```

### CosmWasm Testing

```rust
use cosmwasm_std::testing::{execute_entry_point, instantiate_entry_point, query_entry_point};

#[test]
fn test_increment() {
    let mut deps = mock_deps();
    let msg = InstantiateMsg { count: 0 };
    let res = instantiate_entry_point(deps.as_mut(), mock_env(), mock_info("sender", &[]), msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    let msg = ExecuteMsg::Increment {};
    execute_entry_point(deps.as_mut(), mock_env(), mock_info("sender", &[]), msg).unwrap();
    let res = query_entry_point(deps.as_ref(), mock_env(), QueryMsg::GetCount).unwrap();
    let state: State = from_json(&res).unwrap();
    assert_eq!(state.count, 1);
}
```

### Solana Testing

```bash
# Use Solana test validator
solana-test-validator

# Or use Anchor's test framework with TypeScript
anchor test
```

Anchor's `anchor test` runs TypeScript tests via `Mocha`/`Chai` against a local test validator.

## Contract Upgrade Patterns

- **ink!:** contracts are immutable by default. Upgrades require a separate upgrade mechanism (e.g., proxy pattern) or using Substrate's pallet-contracts with a Wasm hash change.
- **CosmWasm:** contracts are immutable. Use migration with `MigrationMsg` and a new contract that reads old storage, or use the `cw3` multisig + migration pattern.
- **Solana:** `ProgramData` account holds the program, and upgrades are signed by the upgrade authority. Anchor provides `UpgradeChecked` / `UpgradeUnchecked` instructions.

## Verification Checklist

- [ ] Can write a minimal ink! contract with storage, constructor, and messages
- [ ] Can write a minimal CosmWasm contract with instantiate, execute, query
- [ ] Can write an Anchor Solana program with accounts and a program entry
- [ ] Can emit events in ink! and CosmWasm
- [ ] Can do cross-contract calls in CosmWasm (`CosmWasmMsg`) and ink! (callable)
- [ ] Can write tests for CosmWasm with `mock_deps`
- [ ] Can build and deploy an ink! contract with `cargo contract`
- [ ] Understands the storage model for each framework (Mapping in ink!, Item in CosmWasm, Account in Anchor)
- [ ] Understands that contracts are generally immutable and upgrades require explicit patterns

## Common Pitfalls

1. **Assuming contracts are upgradeable by default.** Most Wasm-based contracts are immutable. Upgrades require explicit patterns (proxy, migration, program upgrade authority).

2. **Not validating caller in ink! messages.** `self.env().caller()` returns the caller. Not checking it can lead to unauthorized access.

3. **Using excessive storage in CosmWasm.** Each storage write has a gas cost. Minimize reads/writes, batch where possible.

4. **Forgetting to initialize Lazy values in ink!.** `Lazy<T>` initializes on first access. If the default isn't what you want, set it explicitly.

5. **Not testing cross-contract calls in CosmWasm.** The `CosmWasmMsg` path needs testing with the full dispatch — use `mock_deps` but ensure the sub-messages execute correctly.

6. **Anchor account space miscalculation.** If an account's space is too small, subsequent writes fail. Calculate space carefully: 8 bytes discriminator + fields.

7. **Not handling errors in CPI.** Cross-program calls can fail. Handle the `Result` and return appropriate errors.

8. **Using `std` in CosmWasm contracts.** CosmWasm contracts run in `no_std` (wasmvm). Don't use `std` — use `cosmwasm_std` types.

9. **Not verifying the program ID in Anchor.** Anchor's `declare_id!` must match the deployed program ID. Mismatch leads to account deserialization failures.

10. **Ignoring gas limits.** Each chain has gas/compute budget limits. Write efficient contracts and test with realistic workloads.
