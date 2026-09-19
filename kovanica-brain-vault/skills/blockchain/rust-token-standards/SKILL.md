---
name: rust-token-standards
description: Use when implementing token standards in Rust: ERC20/ERC721/ERC1155 for EVM, SPL for Solana, CW20/CW721 for Cosmos, and common token patterns (transfers, approvals, metadata, minting, burning).
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, blockchain, tokens, ERC20, ERC721, ERC1155, SPL, CW20, CW721, fungible, non-fungible, metadata, approvals]
    related_skills: [rust-smart-contracts, rust-account-ledger, rust-defi]
---

# Rust Token Standards

## Overview

Token standards define the interface and behavior for fungible and non-fungible tokens on different blockchains. The standard determines how tokens are transferred, approved, minted, burned, and queried. Each chain ecosystem has its own standards.

| Standard | Chain | Type | Notes |
|---|---|---|---|
| **ERC20** | EVM (Ethereum, etc.) | Fungible | Transfer, approve, allowance, mint/burn (optional) |
| **ERC721** | EVM | Non-fungible (NFT) | Transfer, approve, ownerOf, tokenURI |
| **ERC1155** | EVM | Multi-token (fungible + NFT in one contract) | Batch transfers, multiple IDs |
| **SPL Token** | Solana | Fungible / NFT | SPL program, 토큰 accounts, mint/revoke authorities |
| **CW20** | Cosmos (CosmWasm) | Fungible | Transfer, approve, mint, burn, hooks |
| **CW721** | Cosmos (CosmWasm) | Non-fungible | Transfer, approve, owner, metadata |

This skill covers the common patterns and interfaces. The actual implementation depends on the chain and the framework (ink!, CosmWasm, Solana, Solidity).

## When to Use

- Implementing a fungible token (ERC20, CW20, SPL)
- Implementing an NFT (ERC721, CW721, SPL NFT)
- Implementing a multi-token contract (ERC1155)
- Handling approvals and allowances (delegated transfers)
- Managing token metadata (name, symbol, decimals, URI)
- Implementing minting and burning (permissioned or permissionless)
- Building a token that integrates with existing standards (Wallets, marketplaces, DeFi)

**Don't use for:** consensus tokens (the token standard is for user-facing tokens, not the chain's native currency or consensus mechanism).

## ERC20 (Fungible Token on EVM)

### Interface (conceptual)

```rust
// ERC20 interface — what a contract must implement
interface ERC20 {
    function totalSupply() external view returns (uint256);
    function balanceOf(address account) external view returns (uint256);
    function transfer(address to, uint256 amount) external returns (bool);
    function allowance(address owner, address spender) external view returns (uint256);
    function approve(address spender, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);

    // Optional / common extensions
    function name() external view returns (string);
    function symbol() external view returns (string);
    function decimals() external view returns (uint8);

    // Events
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
}
```

### Key Behaviors

- **Transfer:** moves tokens from the caller to another address. Emits `Transfer`.
- **Approve:** sets the allowance for a spender. Emits `Approval`.
- **TransferFrom:** spends the caller's allowance to move tokens from one address to another. Used by DeFi contracts, marketplaces.
- **Mint/Burn:** optional, extension. Not in the base ERC20 standard — added by specific contracts.

### Approval Pattern (Race Condition Warning)

The ERC20 approval pattern has a known race condition: if a user approves a spender, then changes the approval, the spender can use the old approval before the new one is set. Some standards (ERC20 with `approve` returning the previous allowance, or `increaseAllowance`/`decreaseAllowance`) mitigate this.

**Safe pattern:** use `increaseAllowance` / `decreaseAllowance` (ERC20 extensions) rather than setting an absolute allowance.

## ERC721 (Non-Fungible Token on EVM)

### Interface (conceptual)

```rust
interface ERC721 {
    function balanceOf(address owner) external view returns (uint256);
    function ownerOf(uint256 tokenId) external view returns (address);
    function transferFrom(address from, address to, uint256 tokenId) external;
    function safeTransferFrom(address from, address to, uint256 tokenId) external;
    function approve(address to, uint256 tokenId) external;
    function setApprovalForAll(address operator, bool approved) external;
    function getApproved(uint256 tokenId) external view returns (address);
    function isApprovedForAll(address owner, address operator) external view returns (bool);

    // Metadata (optional, ERC721Metadata)
    function name() external view returns (string);
    function symbol() external view returns (string);
    function tokenURI(uint256 tokenId) external view returns (string);

    // Events
    event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
    event Approval(address indexed owner, address indexed approved, uint256 indexed tokenId);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
}
```

### Key Behaviors

- **Each token has a unique ID.** `tokenId` is the identifier.
- **Transfer:** moves a specific token from one address to another.
- **Approve:** approves a specific spender to transfer a specific token.
- **SetApprovalForAll:** approves an operator to transfer all tokens of the caller.
- **Safe transfer:** checks that the recipient is a contract that can handle ERC721 (via `onERC721Received`).

### Metadata

ERC721 metadata (name, symbol, tokenURI) is optional but standard. `tokenURI` returns a link to the token's metadata (JSON with image, attributes, etc.). This is how NFTs display in wallets and marketplaces.

## ERC1155 (Multi-Token on EVM)

ERC1155 allows a single contract to manage multiple token types (fungible and non-fungible) with a single `id`.

```rust
interface ERC1155 {
    function balanceOf(address owner, uint256 id) external view returns (uint256);
    function transferFrom(address from, address to, uint256 id, uint256 amount) external;
    function safeTransferFrom(address from, address to, uint256 id, uint256 amount, bytes calldata data) external;
    function approve(address operator, bool approved) external;
    function setApprovalForAll(address operator, bool approved) external;

    // Batch transfers
    function safeTransferFrom(address from, address to, uint256[] calldata ids, uint256[] calldata amounts, bytes calldata data) external;

    // URIs (optional)
    function uri(uint256 id) external view returns (string);

    // Events
    event Transfer(address indexed from, address indexed to, uint256 id, uint256 amount);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
    event URI(string value, uint256 indexed id);
}
```

**Use cases:** gaming (multiple items in one contract), batching transfers (gas-efficient), mixed fungible/NFT collections.

## SPL Token (Solana)

SPL tokens are managed by the SPL Token program (a system program on Solana). Token behavior is defined by the program, and each token type is a **mint**.

### Token Account Model

- **Mint:** represents a token type (supply, decimals, mint authority, revoke authority).
- **Token account:** an account that holds tokens of a specific mint (owner, amount, mint).
- **Associated Token Account (ATA):** a deterministically derived token account for a wallet + mint.

### SPL Token Operations

```rust
// Using the SPL token program (conceptual, via solana-sdk or anchor)
use spl_token::instruction::*;

// Create a mint
let create_mint_ix = create_account(
    payer_pubkey,
    mint_pubkey,
    mint_authority_pubkey,
    token_program_id,
    rent_lamports,
    bytes_per_mint,
)?;

// Mint tokens
let mint_ix = mint_to(
    mint_pubkey,
    token_account_pubkey,
    mint_authority_pubkey,
    amount,
)?;

// Transfer tokens
let transfer_ix = transfer(
    source_token_account,
    destination_token_account,
    owner_pubkey,
    amount,
)?;

// Approve (delegate spending)
let approve_ix = approve(
    token_account_pubkey,
    delegate_pubkey,
    owner_pubkey,
    amount,
)?;
```

### SPL NFT (Token with 1 decimal, fixed supply)

SPL NFTs are SPL tokens with 0 or 1 decimal and a limited supply (usually 1). The NFT metadata is often stored in a separate account (Metaplex metadata program).

```rust
// Create an SPL NFT (mint with supply = 1, decimals = 0)
let create_nft_mint_ix = create_account(...)?;   // mint account
// Set mint authority to the creator, revoke authority to the creator
// Set supply to 1, decimals to 0
```

### Metaplex Metadata (SPL NFT metadata)

SPL NFT metadata (name, symbol, URI, creators) is stored in a separate Metaplex metadata account, referenced by the mint.

```rust
// Metaplex metadata account (separate from the SPL mint)
// Contains: name, symbol, URI, creators, seller fee basis points, etc.
```

## CW20 (Cosmos Fungible Token)

CW20 is the Cosmos SDK equivalent of ERC20, for CosmWasm contracts.

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub sender: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_supply: Option<Coin>,
    pub mint_enabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TransferMsg {
    pub amount: Uint128,
    pub recipient: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApproveMsg {
    pub spender: String,
    pub amount: Uint128,
}
```

### CW20 Interface

```rust
pub enum ExecuteMsg {
    Transfer { amount: Uint128, recipient: String },
    Approve { spender: String, amount: Uint128 },
    IncreaseAllowance { spender: String, amount: Uint128 },
    DecreaseAllowance { spender: String, amount: Uint128 },
    Send { contract: String, amount: Uint128, msg: Binary },
    Mint { recipient: String, amount: Uint128 },
    Burn { amount: Uint128 },
}

pub enum QueryMsg {
    Balance { address: String },
    TotalSupply {},
    Allowance { owner: String, spender: String },
    TokenInfo {},
}
```

**CW20 hooks:** CW20 supports a `Send` message that calls a contract with a message after the transfer (for hook-based integrations).

## CW721 (Cosmos NFT)

CW721 is the Cosmos SDK equivalent of ERC721.

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub seller: String,
    pub name: String,
    pub symbol: String,
    pub mints: Vec<MintMsg>,   // initial mints
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TransferMsg {
    pub token_id: String,
    pub recipient: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApprovalMsg {
    pub token_id: String,
    pub approval: Approval,   // address + expiration
}
```

### CW721 Query

```rust
pub enum QueryMsg {
    OwnerOf { token_id: String },
    ContractsOwners { limit: u32, start_after: Option<String> },
    Approval { token_id: String },
    Metadata { token_id: String },
}
```

## Common Token Patterns

### Transfer Event

All token standards emit a transfer event on transfer. This is the primary way indexers and wallets track token movements.

```rust
// Transfer event (conceptual across standards)
event Transfer {
    from: Address,
    to: Address,
    amount: u64,        // or value for ERC20
    token_id: Option<u64>,   // for NFT / multi-token
}
```

### Approval / Allowance

For delegated spending (DeFi, marketplaces), tokens use approvals:
- **ERC20:** approve a spender for an amount.
- **ERC721:** approve a spender for a specific token, or setApprovalForAll.
- **SPL:** approve a delegate on a token account.
- **CW20/CW721:** similar approval patterns.

### Minting and Burning

- **Mint:** create new tokens (increases supply). Usually permissioned (only the admin/minter can mint).
- **Burn:** destroy tokens (decreases supply). Can be permissioned or permissionless (users burn their own tokens).

```rust
// Permissionless burn (user burns their own tokens)
pub fn burn(&mut self, amount: u64) -> Result<(), Error> {
    let caller = self.env().caller();
    let balance = self.balances.get(&caller).unwrap_or(0);
    if balance < amount {
        return Err(Error::InsufficientBalance);
    }
    self.balances.insert(&caller, &(balance - amount));
    self.total_supply -= amount;
    self.env().emit_event(Transfer { from: caller, to: Address::zero(), amount });
    Ok(())
}

// Permissioned mint (admin mints new tokens)
pub fn mint(&mut self, to: Address, amount: u64) -> Result<(), Error> {
    if self.env().caller() != self.admin {
        return Err(Error::NotAuthorized);
    }
    let current = self.balances.get(&to).unwrap_or(0);
    self.balances.insert(&to, &(current + amount));
    self.total_supply += amount;
    self.env().emit_event(Transfer { from: Address::zero(), to, amount });
    Ok(())
}
```

### Decimals

Fungible tokens have decimals (usually 18 for ERC20, 9 for SPL, configurable for CW20). Decimals determine the smallest representable unit.

```rust
// Display amount with decimals
pub fn display_amount(raw: u64, decimals: u8) -> String {
    let divisor = 10u128.pow(decimals);
    format!("{:.*}", decimals as usize, raw as f64 / divisor as f64)
}
```

### Metadata

Token metadata (name, symbol, URI for NFTs) is stored in the contract (ERC20/721) or in a separate metadata structure (SPL + Metaplex, CW721 metadata).

For NFTs, metadata is often a JSON file hosted off-chain (IPFS, HTTP), referenced by the `tokenURI` or metadata URL.

## Verification Checklist

- [ ] Can describe the ERC20 interface and common operations (transfer, approve, transferFrom)
- [ ] Can describe the ERC721 interface and NFT-specific operations (transferFrom, approve, tokenURI)
- [ ] Can describe ERC1155 multi-token and batch transfers
- [ ] Can describe SPL Token program operations (mint, transfer, approve, ATA)
- [ ] Can describe CW20 / CW721 interfaces for Cosmos
- [ ] Can implement a basic transfer with event emission
- [ ] Can implement an approval pattern with allowance tracking
- [ ] Can implement minting and burning (permissioned or permissionless)
- [ ] Understands the role of decimals in fungible tokens
- [ ] Understands that transfer events are the primary mechanism for tracking token movement

## Common Pitfalls

1. **Not handling approvals correctly.** Approvals are a major source of bugs (race conditions, insufficient allowance checks). Implement carefully.

2. **Minting without access control.** Unrestricted minting breaks token economics. Always restrict minting to authorized accounts.

3. **Burn without checking balance.** Burning more than the balance is an error. Check before burning.

4. **Forgetting to emit transfer events.** Indexers and wallets rely on transfer events. Always emit them.

5. **Inconsistent decimals.** If a token has decimals, all amounts must be handled with decimals in mind (internal representation vs display).

6. **ERC721 `safeTransferFrom` without checking recipient contract.** A safe transfer checks that the recipient can handle ERC721 (via `onERC721Received`). Without it, tokens can be sent to a contract that can't handle them (lost).

7. **SPL token account not initialized.** SPL tokens require a token account for each wallet + mint combination. Sending tokens to an uninitialized account fails.

8. **Metadata pointing to a broken URI.** NFT metadata URIs (IPFS, HTTP) can become unavailable. Consider pinning or using a reliable storage mechanism.

9. **Approving infinite allowance.** Approving a large or infinite allowance gives the spender broad permissions. Use the minimum necessary allowance.

10. **Not considering reentrancy on transfers.** If a transfer triggers a contract call (e.g., hook, safe transfer), reentrancy is possible. Use checks-effects-interactions or reentrancy guards.
