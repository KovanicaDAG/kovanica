---
name: rust-cosmos
description: Use when building on Cosmos SDK: modules (x/), CometBFT consensus, IBC, state machine, messages, gas, and building custom modules or integrating with Cosmos chains in Rust.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, blockchain, cosmos, cosmos-sdk, CometBFT, IBC, module, x/, state-machine, gas, cosmos-rust]
    related_skills: [rust-smart-contracts, rust-crosschain, rust-merkle-structures, rust-crypto]
---

# Rust Cosmos

## Overview

Cosmos SDK is a framework for building application-specific blockchains in Go. However, Rust plays a significant role in the Cosmos ecosystem:

- **CosmWasm:** Rust smart contracts that run on Cosmos SDK chains (via wasmvm).
- **Light clients and bridges:** Rust implementations of Cosmos light clients, IBC relayers, and cross-chain tooling.
- **Tooling:** Rust CLIs, indexers, and utilities for Cosmos chains.

This skill covers the Cosmos architecture, modules, IBC, and the Rust integration points. For CosmWasm contracts, see rust-smart-contracts.

## When to Use

- Building a Cosmos SDK chain (Go, but understanding the architecture helps)
- Writing CosmWasm contracts in Rust (see rust-smart-contracts)
- Building a light client of a Cosmos chain in Rust
- Building an IBC relayer or bridge in Rust
- Integrating with Cosmos chains (indexing, querying, transaction crafting)
- Understanding Cosmos state machine, messages, gas, and governance

**Don't use for:** writing Cosmos SDK modules (Go, primarily). Use Rust for CosmWasm, tooling, light clients, and IBC.

## Cosmos SDK Architecture

```
┌─────────────┐    ┌─────────────┐    ┌──────────────────┐
│  CometBFT   │───▶│ Cosmos SDK  │───▶│     Modules      │
│ (consensus  │    │ (state      │    │ (x/bank, x/stake, │
│  + p2p)     │    │  machine)   │    │  x/gov, x/ibc,   │
└─────────────┘    └─────────────┘    │  ...custom)      │
                                      └──────────────────┘
```

- **CometBFT:** consensus engine (BFT consensus, p2p networking, mempool). The chain's consensus layer.
- **Cosmos SDK:** the state machine (application logic). Handles transactions, state, modules.
- **Modules (x/*):** individual modules for specific functionality (bank for tokens, stake for PoS, gov for governance, IBC for interoperability, etc.).

## CometBFT Consensus

CometBFT is a Byzantine Fault Tolerant (BFT) consensus engine. It produces blocks via a round-based protocol, with validators voting on blocks.

### Consensus Steps (simplified)

1. **Propose:** a validator is chosen to propose a block (round-robin or stake-based).
2. **Prevote:** validators prevote on the proposed block.
3. **Precommit:** if > 2/3 prevotes, validators precommit.
4. **Commit:** if > 2/3 precommits, the block is committed.

**Finality:** CometBFT provides immediate finality (once a block is committed, it's final). This is different from PoW chains (probabilistic finality).

### Validator Set

The validator set is determined by the staking module (x/stake). Validators are bonded (staked) tokens, and the set can change over time (epochs).

## Cosmos SDK Modules (x/)

### Common Modules

| Module | Purpose |
|---|---|
| `x/bank` | Token transfers (coin movement between accounts) |
| `x/stake` | Proof-of-stake: validators, delegation, rewards, slashing |
| `x/gov` | On-chain governance: proposals, voting, execution |
| `x/ibc` | Inter-Blockchain Communication (IBC) protocol |
| `x/account` | Account types (fee payer, base account) |
| `x/auth` | Transaction authentication (signatures, accounts) |
| `x/upgrade` | Module upgrades, store migrations |
| `x/evidence` | Evidence of misbehavior (slashing) |
| `x/slashing` | Slashing logic (downtime, double-signing) |
| `x/mint` | Token inflation (block rewards) |
| `x/distribution` | Reward distribution to validators/delegators |
| `x/liquid_stake` | Liquid staking (optional) |

### Module Structure

A Cosmos SDK module has:
- **Types:** messages, events, query structures.
- **Keeper:** the module's logic (state access, message handling).
- **MsgServer:** gRPC/REST query and message handlers.
- **BeginBlock / EndBlock:** hooks called at the start/end of each block (for inflation, rewards, slashing).
- **Interfaces:** `ibc` module interfaces for IBC connections.

## IBC (Inter-Blockchain Communication)

IBC is Cosmos's cross-chain messaging protocol. It allows chains to send packets (messages) to each other, with light client verification of the counterparty chain.

### IBC Architecture

```
Chain A                         Chain B
┌──────────┐    packet     ┌──────────┐
│  IBC     │─── send ─────▶│  IBC     │
│  Module  │               │  Module  │
│          │               │          │
│  Light   │◀── verify ────│  Light   │
│  Client  │               │  Client  │
└──────────┘               └──────────┘
```

1. **Packet creation:** Chain A creates an IBC packet (source, destination, data).
2. **Packet commitment:** the packet is committed to Chain A's state (Merkle proof).
3. **Relayer:** a relayer moves the packet + proof to Chain B.
4. **Verification:** Chain B's light client of Chain A verifies the proof.
5. **Packet processing:** Chain B processes the packet (delivers the message).
6. **Acknowledgement:** Chain B sends an acknowledgement back to Chain A.

### IBC Components

- **Connections:** a link between two chains (with client identifiers).
- **Channels:** a channel over a connection, for a specific port/protocol.
- **Ports:** a module's IBC port (e.g., `transfer` port for the transfer module).
- **Packets:** the messages sent over channels.
- **Clients:** light clients of counterparty chains (e.g., `ibc/client` for Tendermint light clients).

### IBC in Rust (Light Client)

Cosmos light clients can be implemented in Rust for verification in other contexts (bridges, indexers, etc.).

```rust
// Conceptual: Tendermint light client in Rust
pub struct TendermintLightClient {
    trusted_height: u64,
    trusted_hash: Hash,
    trusted validators: ValidatorSet,
}

impl TendermintLightClient {
    pub fn verify_header(&mut self, header: &TendermintHeader) -> Result<(), Error> {
        // Verify the header's validators signature (> 2/3 of trusted validators)
        // Verify the header links to the previous trusted header
        // Update the trusted state
    }

    pub fn verify_packet_commitment(
        &self,
        packet: &IbcPacket,
        proof: &IbcProof,
        height: u64,
    ) -> Result<(), Error> {
        // Verify the Merkle proof of the packet commitment at the given height
        // Verify the height is within the trusted range
    }
}
```

## CosmWasm (Rust on Cosmos)

CosmWasm is the Rust smart contract engine for Cosmos SDK chains. Contracts are Wasm, run via `wasmvm`.

**See rust-smart-contracts** for CosmWasm contract development.

### CosmWasm Integration with Cosmos

- CosmWasm contracts use the same account model as native Cosmos modules.
- Contracts can send IBC packets (via `IbcMsg`).
- Contracts interact with native modules via `CosmosMsg` (e.g., bank transfers, staking operations).
- Queries to native modules are available to contracts (e.g., querying bank balances).

## Cosmos Messages and Transactions

A Cosmos transaction contains messages (Msgs). Each message is handled by a specific module.

```rust
// Conceptual: Cosmos SDK message
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MsgTransfer {
    pub from_address: String,
    pub to_address: String,
    pub amount: Coin,
}

// Transaction contains multiple messages
pub struct Tx {
    pub messages: Vec<Msg>,
    pub signatures: Vec<Signature>,
    pub memo: String,
    pub fee: Fee,
    pub gas_limit: u64,
}
```

**Gas:** each message has a gas cost. The total gas is the sum of message gas costs + overhead. Gas is paid in the chain's token.

## Gas and Fees in Cosmos

```rust
pub struct Fee {
    pub amount: Vec<Coin>,   // fee tokens
    pub gas_limit: u64,
}
```

The fee is paid upfront. Any unused gas is refunded (partial refund). The gas price is determined by the fee market or set by the validator.

## State Machine

The Cosmos SDK is a state machine: each block applies a set of transactions, transitioning the state. The state is stored in a Merkle trie (IAVL trie or similar).

```
State → apply block (transactions) → new state
```

The state root is included in the block header, enabling light client verification.

## Custom Cosmos Modules (Go, but Rust-aware)

Custom modules are typically written in Go, but Rust developers can understand the structure:

```go
// Conceptual Go module structure
type MsgServer struct {
    Keeper Keeper
}

func (k Keeper) Transfer(ctx context.Context, msg MsgTransfer) error {
    // Check sender balance
    // Deduct from sender, add to recipient
    // Emit event
    return nil
}
```

For Rust developers working with Cosmos, the key points are:
- Modules expose `MsgServer` (gRPC/REST) and `QueryServer`.
- Messages are authenticated via signatures (auth module).
- State is accessed via the keeper (which wraps the store).
- Events are emitted for indexing.

## Verification Checklist

- [ ] Can describe CometBFT consensus (propose, prevote, precommit, commit) and finality
- [ ] Can describe the Cosmos SDK state machine (block → transactions → state transition)
- [ ] Can describe common Cosmos SDK modules (bank, stake, gov, IBC)
- [ ] Can describe IBC architecture (connections, channels, packets, light clients, relayers)
- [ ] Can describe CosmWasm's role in Cosmos (Rust contracts via wasmvm)
- [ ] Can describe Cosmos messages and transactions (multimsg, gas, fees)
- [ ] Can describe gas and fee mechanics in Cosmos
- [ ] Understands that Cosmos finality is immediate (BFT), unlike PoW

## Common Pitfalls

1. **Confusing Cosmos SDK with CosmWasm.** Cosmos SDK is the chain framework (Go). CosmWasm is the smart contract engine (Rust/Wasm). They're related but different.

2. **Assuming IBC is automatic.** IBC requires configuration (connections, channels, relayers). It's not "on" by default.

3. **Not understanding finality.** Cosmos has immediate finality (BFT). Once a block is committed, it's final. This is different from PoW chains.

4. **Ignoring gas in Cosmos transactions.** Every message costs gas. Underestimating gas leads to transaction failures.

5. **Not verifying IBC proofs correctly.** IBC relies on light client verification of remote chain headers. If the light client is buggy, the bridge can be fooled.

6. **Assuming all Cosmos chains are compatible.** Each chain can customize its modules. IBC works between them, but the application-level semantics may differ.

7. **Not considering validator set changes.** The validator set changes over time (epochs). Light clients must track these changes.

8. **Not handling IBC timeouts.** IBC packets have timeouts (height or timestamp). If the packet isn't processed in time, it times out.

9. **Assuming CosmWasm contracts have the same capabilities as native modules.** CosmWasm contracts have limited access to the chain's state (via queries and messages). They can't do everything a native module can.

10. **Not testing IBC relayer behavior.** Relayers move packets. Test relayer logic with simulated chains to ensure packets are relayed correctly.
