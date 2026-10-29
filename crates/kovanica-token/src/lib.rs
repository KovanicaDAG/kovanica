//! Standardized Fungible Token (multi-asset) for kovanica
//!
//! Simple, minimal ERC-20-like interface adapted to account model + Wasmi.
//! Features:
//!   - mint (only minter / owner)
//!   - burn
//!   - transfer
//!   - approve / transfer_from
//!   - total_supply, balance_of, allowance
//!
//! One contract instance = one token.
//! For true multi-asset native support you can later promote this to a system precompile.
//!
//! ## Asset model
//! Token balances live in the contract's **own serialized state** (a
//! `BTreeMap`), NOT in native host balances. `transfer` / `transfer_from` /
//! `mint` / `burn` never call `Host::transfer` — only the contract's storage
//! changes. Native KUNA and token balances are therefore fully independent.
//!
//! ## Storage keys (FIXED)
//!
//! - singleton state: `b"kvnc/v1/state"` (one token per contract address)
//!
//! Runtime hosts are expected to namespace storage per contract address.
//!
//! ## WASM entry points (wasm32 only; bincode args in, bincode results out)
//!
//! | export             | args tuple                                        | Ok result |
//! |--------------------|---------------------------------------------------|-----------|
//! | `token_create`     | `(String name, String symbol, u8 decimals, u128 initial_supply, String icon)` | `()` |
//! | `token_transfer`   | `([u8;32] to, u128 amount)`                       | `()`      |
//! | `token_approve`    | `([u8;32] spender, u128 amount)`                  | `()`      |
//! | `token_transfer_from` | `([u8;32] from, [u8;32] to, u128 amount)`       | `()`      |
//! | `token_mint`       | `([u8;32] to, u128 amount)`                       | `()`      |
//! | `token_burn`       | `(u128 amount)`                                   | `()`      |
//!
//! All Ok results are `()` → success return is `0` (null ptr, zero len).
//! Error packing is identical to kovanica-htlc (crate docs):
//! `-(1 + ContractError::code())` within `-12..=-1`; `kovanica_alloc` /
//! `kovanica_dealloc` are exported alongside.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;

use kovanica_common::events::{
    TOKEN_APPROVAL, TOKEN_BURN, TOKEN_CREATED, TOKEN_MINT, TOKEN_TRANSFER,
};
use kovanica_common::{Address, Amount, ContractError, ContractResult, Host};

/// Singleton state key (FIXED): one token instance per contract address.
const STATE_KEY: &[u8] = b"kvnc/v1/state";

/// Largest accepted icon payload, in bytes. A 96×96 PNG data URI is ~3–8 KiB;
/// 8 KiB keeps the whole token state (and every wallet's decode) small.
pub const MAX_ICON_BYTES: usize = 8 * 1024;

/// bincode failed while serializing token state — infallible for these
/// field types in practice; surfaces state-layer corruption.
const ERR_BINCODE_SERIALIZE: u32 = 1;
/// bincode failed while deserializing stored token state (corrupt/foreign bytes).
const ERR_BINCODE_DESERIALIZE: u32 = 2;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Token {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Amount,
    pub owner: Address,  // can mint / change minter
    pub minter: Address, // can mint
    pub balances: BTreeMap<Address, Amount>,
    pub allowances: BTreeMap<(Address, Address), Amount>, // owner → spender → amount
    pub paused: bool,
    /// Optional artwork: `https://…` or a `data:image/…` URI. Empty when the
    /// token has none (wallets fall back to a generated mark).
    pub icon: String,
}

/// State layout written before `icon` existed (v1). Kept so a v2 contract can
/// still read reserves/tokens created by the previous wasm build.
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(dead_code)] // fields exist only to consume the v1 byte layout
struct TokenV1 {
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: Amount,
    owner: Address,
    minter: Address,
    balances: BTreeMap<Address, Amount>,
    allowances: BTreeMap<(Address, Address), Amount>,
    paused: bool,
}

impl From<TokenV1> for Token {
    fn from(v1: TokenV1) -> Self {
        Token {
            name: v1.name,
            symbol: v1.symbol,
            decimals: v1.decimals,
            total_supply: v1.total_supply,
            owner: v1.owner,
            minter: v1.minter,
            balances: v1.balances,
            allowances: v1.allowances,
            paused: v1.paused,
            icon: String::new(),
        }
    }
}

impl Token {
    /// Deploy a new token (singleton state — one per contract address).
    /// Caller becomes owner + minter and receives the initial supply.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        host: &mut impl Host,
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: Amount,
        icon: String,
    ) -> ContractResult<()> {
        if decimals > 18 {
            return Err(ContractError::InvalidInput);
        }
        if Self::load(host).is_ok() {
            // Singleton model: never silently replace an existing token.
            return Err(ContractError::AlreadyExists);
        }
        Self::validate_icon(&icon)?;

        let caller = host.caller();
        let mut balances = BTreeMap::new();
        if initial_supply > 0 {
            balances.insert(caller, initial_supply);
        }

        let token = Token {
            name,
            symbol,
            decimals,
            total_supply: initial_supply,
            owner: caller,
            minter: caller,
            balances,
            allowances: BTreeMap::new(),
            paused: false,
            icon,
        };

        Self::store(host, &token)?;
        host.emit_event(TOKEN_CREATED, &[]);
        Ok(())
    }

    /// Transfer tokens from the caller to `to` (internal balances only —
    /// no native `Host::transfer`).
    pub fn transfer(host: &mut impl Host, to: Address, amount: Amount) -> ContractResult<()> {
        let mut token = Self::load(host)?;
        if token.paused {
            return Err(ContractError::Paused);
        }
        let from = host.caller();
        Self::move_balance(&mut token, &from, &to, amount)?;
        Self::store(host, &token)?;
        host.emit_event(TOKEN_TRANSFER, &[]);
        Ok(())
    }

    /// Approve `spender` to move up to `amount` of the caller's tokens.
    pub fn approve(host: &mut impl Host, spender: Address, amount: Amount) -> ContractResult<()> {
        let mut token = Self::load(host)?;
        let owner = host.caller();
        token.allowances.insert((owner, spender), amount);
        Self::store(host, &token)?;
        host.emit_event(TOKEN_APPROVAL, &[]);
        Ok(())
    }

    /// Move tokens from `from` to `to` using the caller's allowance.
    /// The allowance is decremented on success; a failed attempt (e.g.
    /// insufficient balance) leaves state untouched because `store` is not
    /// reached.
    pub fn transfer_from(
        host: &mut impl Host,
        from: Address,
        to: Address,
        amount: Amount,
    ) -> ContractResult<()> {
        let mut token = Self::load(host)?;
        if token.paused {
            return Err(ContractError::Paused);
        }
        let spender = host.caller();

        let key = (from, spender);
        let allowance = token.allowances.get(&key).copied().unwrap_or(0);
        if allowance < amount {
            return Err(ContractError::InsufficientBalance);
        }
        token.allowances.insert(key, allowance - amount);

        Self::move_balance(&mut token, &from, &to, amount)?;
        Self::store(host, &token)?;
        host.emit_event(TOKEN_TRANSFER, &[]);
        Ok(())
    }

    /// Mint `amount` tokens to `to`. Only the current minter may mint.
    pub fn mint(host: &mut impl Host, to: Address, amount: Amount) -> ContractResult<()> {
        let mut token = Self::load(host)?;
        if host.caller() != token.minter {
            return Err(ContractError::Unauthorized);
        }
        if token.paused {
            return Err(ContractError::Paused);
        }

        let new_supply = token
            .total_supply
            .checked_add(amount)
            .ok_or(ContractError::Overflow)?;
        token.total_supply = new_supply;

        let bal = token.balances.entry(to).or_insert(0);
        *bal = bal.checked_add(amount).ok_or(ContractError::Overflow)?;

        Self::store(host, &token)?;
        host.emit_event(TOKEN_MINT, &[]);
        Ok(())
    }

    /// Burn `amount` tokens from the caller's balance, reducing total supply.
    pub fn burn(host: &mut impl Host, amount: Amount) -> ContractResult<()> {
        let mut token = Self::load(host)?;
        let from = host.caller();
        let bal = token.balances.get(&from).copied().unwrap_or(0);
        if bal < amount {
            return Err(ContractError::InsufficientBalance);
        }
        token.balances.insert(from, bal - amount);
        token.total_supply = token.total_supply.saturating_sub(amount);
        Self::store(host, &token)?;
        host.emit_event(TOKEN_BURN, &[]);
        Ok(())
    }

    // ---------- helpers ----------

    fn move_balance(
        token: &mut Token,
        from: &Address,
        to: &Address,
        amount: Amount,
    ) -> ContractResult<()> {
        let from_bal = token.balances.get(from).copied().unwrap_or(0);
        if from_bal < amount {
            return Err(ContractError::InsufficientBalance);
        }
        let to_bal = token.balances.get(to).copied().unwrap_or(0);
        let new_to = to_bal.checked_add(amount).ok_or(ContractError::Overflow)?;
        // Both checks passed — mutate.
        token.balances.insert(*from, from_bal - amount);
        token.balances.insert(*to, new_to);
        Ok(())
    }

    /// Icon payload rule for `create`.
    fn validate_icon(icon: &str) -> ContractResult<()> {
        if icon.is_empty() {
            return Ok(());
        }
        if icon.len() > MAX_ICON_BYTES {
            return Err(ContractError::InvalidInput);
        }
        if !(icon.starts_with("https://") || icon.starts_with("data:image/")) {
            return Err(ContractError::InvalidInput);
        }
        // No control characters: a wallet may render this verbatim.
        if icon
            .chars()
            .any(|c| (c as u32) < 0x20 || (c as u32) == 0x7f)
        {
            return Err(ContractError::InvalidInput);
        }
        Ok(())
    }

    fn store(host: &mut impl Host, token: &Token) -> ContractResult<()> {
        let bytes =
            bincode::serialize(token).map_err(|_| ContractError::Custom(ERR_BINCODE_SERIALIZE))?;
        host.storage_set(STATE_KEY, &bytes);
        Ok(())
    }

    fn load(host: &impl Host) -> ContractResult<Token> {
        let bytes = host.storage_get(STATE_KEY).ok_or(ContractError::NotFound)?;
        // Current layout first; then the v1 layout without `icon`, so tokens
        // created by the previous contract build stay readable.
        if let Ok(token) = bincode::deserialize::<Token>(&bytes) {
            return Ok(token);
        }
        bincode::deserialize::<TokenV1>(&bytes)
            .map(Token::from)
            .map_err(|_| ContractError::Custom(ERR_BINCODE_DESERIALIZE))
    }
}

// ---------- WASM entry points (wasm32 only) ----------

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use kovanica_common::WasmHost;

    fn error_code(e: ContractError) -> i64 {
        -1 - e.code() as i64
    }

    fn ok_out<T: serde::Serialize>(value: &T) -> i64 {
        match bincode::serialize(value) {
            Ok(bytes) => {
                if bytes.is_empty() {
                    0 // () results — null ptr, zero len
                } else {
                    // capacity == len so host-side kovanica_dealloc(ptr, len) is exact.
                    let mut buf = Vec::with_capacity(bytes.len());
                    buf.extend_from_slice(&bytes);
                    let ptr = buf.as_ptr();
                    let len = buf.len();
                    core::mem::forget(buf); // ownership → host
                    ((ptr as u32 as i64) << 32) | (len as u32 as i64)
                }
            }
            Err(_) => error_code(ContractError::Custom(ERR_BINCODE_SERIALIZE)),
        }
    }

    unsafe fn args_slice<'a>(ptr: i32, len: i32) -> Option<&'a [u8]> {
        if len < 0 || (ptr <= 0 && len > 0) {
            None
        } else if len == 0 {
            Some(&[])
        } else {
            Some(core::slice::from_raw_parts(ptr as *const u8, len as usize))
        }
    }

    fn decode_args<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, i64> {
        bincode::deserialize(bytes).map_err(|_| error_code(ContractError::InvalidInput))
    }

    /// Vec-based allocation for the host (fixed ABI).
    #[no_mangle]
    pub extern "C" fn kovanica_alloc(len: i32) -> i32 {
        if len <= 0 {
            return 0;
        }
        let mut v: Vec<u8> = Vec::with_capacity(len as usize);
        let ptr = v.as_mut_ptr();
        core::mem::forget(v);
        ptr as i32
    }

    /// # Safety
    /// `ptr`/`len` must be a pair previously handed to the host.
    #[no_mangle]
    pub extern "C" fn kovanica_dealloc(ptr: i32, len: i32) {
        if ptr == 0 || len <= 0 {
            return;
        }
        unsafe {
            drop(Vec::from_raw_parts(ptr as *mut u8, 0, len as usize));
        }
    }

    #[no_mangle]
    pub extern "C" fn token_create(args_ptr: i32, args_len: i32) -> i64 {
        let Some(bytes) = (unsafe { args_slice(args_ptr, args_len) }) else {
            return error_code(ContractError::InvalidInput);
        };
        let (name, symbol, decimals, initial_supply, icon): (String, String, u8, Amount, String) =
            match decode_args(bytes) {
                Ok(v) => v,
                Err(c) => return c,
            };
        let mut host = WasmHost::new();
        match Token::create(&mut host, name, symbol, decimals, initial_supply, icon) {
            Ok(()) => ok_out(&()),
            Err(e) => error_code(e),
        }
    }

    #[no_mangle]
    pub extern "C" fn token_transfer(args_ptr: i32, args_len: i32) -> i64 {
        let Some(bytes) = (unsafe { args_slice(args_ptr, args_len) }) else {
            return error_code(ContractError::InvalidInput);
        };
        let (to, amount): (Address, Amount) = match decode_args(bytes) {
            Ok(v) => v,
            Err(c) => return c,
        };
        let mut host = WasmHost::new();
        match Token::transfer(&mut host, to, amount) {
            Ok(()) => ok_out(&()),
            Err(e) => error_code(e),
        }
    }

    #[no_mangle]
    pub extern "C" fn token_approve(args_ptr: i32, args_len: i32) -> i64 {
        let Some(bytes) = (unsafe { args_slice(args_ptr, args_len) }) else {
            return error_code(ContractError::InvalidInput);
        };
        let (spender, amount): (Address, Amount) = match decode_args(bytes) {
            Ok(v) => v,
            Err(c) => return c,
        };
        let mut host = WasmHost::new();
        match Token::approve(&mut host, spender, amount) {
            Ok(()) => ok_out(&()),
            Err(e) => error_code(e),
        }
    }

    #[no_mangle]
    pub extern "C" fn token_transfer_from(args_ptr: i32, args_len: i32) -> i64 {
        let Some(bytes) = (unsafe { args_slice(args_ptr, args_len) }) else {
            return error_code(ContractError::InvalidInput);
        };
        let (from, to, amount): (Address, Address, Amount) = match decode_args(bytes) {
            Ok(v) => v,
            Err(c) => return c,
        };
        let mut host = WasmHost::new();
        match Token::transfer_from(&mut host, from, to, amount) {
            Ok(()) => ok_out(&()),
            Err(e) => error_code(e),
        }
    }

    #[no_mangle]
    pub extern "C" fn token_mint(args_ptr: i32, args_len: i32) -> i64 {
        let Some(bytes) = (unsafe { args_slice(args_ptr, args_len) }) else {
            return error_code(ContractError::InvalidInput);
        };
        let (to, amount): (Address, Amount) = match decode_args(bytes) {
            Ok(v) => v,
            Err(c) => return c,
        };
        let mut host = WasmHost::new();
        match Token::mint(&mut host, to, amount) {
            Ok(()) => ok_out(&()),
            Err(e) => error_code(e),
        }
    }

    #[no_mangle]
    pub extern "C" fn token_burn(args_ptr: i32, args_len: i32) -> i64 {
        let Some(bytes) = (unsafe { args_slice(args_ptr, args_len) }) else {
            return error_code(ContractError::InvalidInput);
        };
        let amount: Amount = match decode_args(bytes) {
            Ok(v) => v,
            Err(c) => return c,
        };
        let mut host = WasmHost::new();
        match Token::burn(&mut host, amount) {
            Ok(()) => ok_out(&()),
            Err(e) => error_code(e),
        }
    }
}

// ---------- native tests ----------

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_common::MemHost;

    fn addr(tag: u8) -> Address {
        [tag; 32]
    }

    const A: u8 = 1;
    const B: u8 = 2;
    const SPENDER: u8 = 3;
    const CONTRACT: u8 = 9;

    fn new_token(caller: Address, supply: Amount) -> MemHost {
        let mut host = MemHost::new(caller, addr(CONTRACT));
        Token::create(
            &mut host,
            String::from("Kovanica"),
            String::from("KUNA"),
            8,
            supply,
            String::new(),
        )
        .unwrap();
        host
    }

    #[test]
    fn create_with_initial_supply() {
        let host = new_token(addr(A), 1_000);
        let t = Token::load(&host).unwrap();
        assert_eq!(t.name, "Kovanica");
        assert_eq!(t.symbol, "KUNA");
        assert_eq!(t.decimals, 8);
        assert_eq!(t.total_supply, 1_000);
        assert_eq!(t.owner, addr(A));
        assert_eq!(t.minter, addr(A));
        assert_eq!(t.balances.get(&addr(A)).copied().unwrap(), 1_000);
    }

    #[test]
    fn duplicate_create_rejected() {
        let mut host = new_token(addr(A), 1_000);
        assert_eq!(
            Token::create(
                &mut host,
                String::from("X"),
                String::from("X"),
                8,
                500,
                String::new()
            ),
            Err(ContractError::AlreadyExists)
        );
    }

    #[test]
    fn create_rejects_invalid_decimals() {
        let mut host = MemHost::new(addr(A), addr(CONTRACT));
        assert_eq!(
            Token::create(
                &mut host,
                String::from("X"),
                String::from("X"),
                19,
                0,
                String::new()
            ),
            Err(ContractError::InvalidInput)
        );
    }

    #[test]
    fn create_stores_the_icon() {
        let mut host = MemHost::new(addr(A), addr(CONTRACT));
        let icon = "data:image/png;base64,iVBORw0KGgo=";
        Token::create(
            &mut host,
            String::from("Art"),
            String::from("ART"),
            2,
            10,
            String::from(icon),
        )
        .unwrap();
        assert_eq!(Token::load(&host).unwrap().icon, icon);
    }

    #[test]
    fn create_without_icon_is_allowed() {
        let host = new_token(addr(A), 1);
        assert_eq!(Token::load(&host).unwrap().icon, "");
    }

    #[test]
    fn create_rejects_bad_icon_payloads() {
        for bad in [
            "http://example.com/a.png",       // not https
            "javascript:alert(1)",            // not an image scheme
            "data:text/html,<script>",        // not an image data URI
            "data:image/png;base64,\u{0}bad", // control character
        ] {
            let mut host = MemHost::new(addr(A), addr(CONTRACT));
            assert_eq!(
                Token::create(
                    &mut host,
                    String::from("X"),
                    String::from("X"),
                    8,
                    0,
                    String::from(bad)
                ),
                Err(ContractError::InvalidInput),
                "icon [{bad}] must be rejected"
            );
        }
    }

    #[test]
    fn create_rejects_oversized_icon() {
        let mut host = MemHost::new(addr(A), addr(CONTRACT));
        let icon = alloc::format!("data:image/png;base64,{}", "A".repeat(MAX_ICON_BYTES));
        assert_eq!(
            Token::create(&mut host, String::from("X"), String::from("X"), 8, 0, icon),
            Err(ContractError::InvalidInput)
        );
    }

    #[test]
    fn v1_state_without_icon_still_loads() {
        // Bytes written by the previous build: the same struct minus `icon`.
        let mut host = new_token(addr(A), 100);
        let v2 = host.storage_get(STATE_KEY).unwrap();
        let mut token = Token::load(&host).unwrap();
        token.icon = String::new();
        let v1 = bincode::serialize(&TokenV1 {
            name: token.name.clone(),
            symbol: token.symbol.clone(),
            decimals: token.decimals,
            total_supply: token.total_supply,
            owner: token.owner,
            minter: token.minter,
            balances: token.balances.clone(),
            allowances: token.allowances.clone(),
            paused: token.paused,
        })
        .unwrap();
        assert!(v1.len() < v2.len(), "v1 has no icon bytes");
        host.storage_set(STATE_KEY, &v1);
        let loaded = Token::load(&host).unwrap();
        assert_eq!(loaded.icon, "");
        assert_eq!(loaded.total_supply, 100);
    }

    #[test]
    fn transfer_moves_internal_balances_not_native() {
        let mut host = new_token(addr(A), 1_000);
        // Fund a NATIVE balance — must remain untouched by token ops.
        host.set_balance(addr(A), 500);

        Token::transfer(&mut host, addr(B), 300).unwrap();

        let t = Token::load(&host).unwrap();
        assert_eq!(t.balances.get(&addr(A)).copied().unwrap(), 700);
        assert_eq!(t.balances.get(&addr(B)).copied().unwrap(), 300);
        assert_eq!(t.total_supply, 1_000);

        // Native balances unchanged — token model uses storage only.
        assert_eq!(host.balance_of(&addr(A)), 500);
        assert_eq!(host.balance_of(&addr(B)), 0);
    }

    #[test]
    fn transfer_insufficient_balance() {
        let mut host = new_token(addr(A), 100);
        assert_eq!(
            Token::transfer(&mut host, addr(B), 200),
            Err(ContractError::InsufficientBalance)
        );
        // State untouched.
        let t = Token::load(&host).unwrap();
        assert_eq!(t.balances.get(&addr(A)).copied().unwrap(), 100);
    }

    #[test]
    fn approve_and_transfer_from_with_allowance_decrement() {
        let mut host = new_token(addr(A), 1_000);

        Token::approve(&mut host, addr(SPENDER), 40).unwrap();
        {
            let t = Token::load(&host).unwrap();
            assert_eq!(
                t.allowances
                    .get(&(addr(A), addr(SPENDER)))
                    .copied()
                    .unwrap(),
                40
            );
        }

        // Spender moves 30 of A's tokens to B.
        host.set_caller(addr(SPENDER));
        Token::transfer_from(&mut host, addr(A), addr(B), 30).unwrap();

        let t = Token::load(&host).unwrap();
        assert_eq!(t.balances.get(&addr(A)).copied().unwrap(), 970);
        assert_eq!(t.balances.get(&addr(B)).copied().unwrap(), 30);
        // Allowance decremented 40 → 10.
        assert_eq!(
            t.allowances
                .get(&(addr(A), addr(SPENDER)))
                .copied()
                .unwrap(),
            10
        );

        // Over the remaining allowance → rejected, allowance unchanged.
        assert_eq!(
            Token::transfer_from(&mut host, addr(A), addr(B), 20),
            Err(ContractError::InsufficientBalance)
        );
        let t = Token::load(&host).unwrap();
        assert_eq!(
            t.allowances
                .get(&(addr(A), addr(SPENDER)))
                .copied()
                .unwrap(),
            10
        );

        // Exactly the remainder → ok, allowance now 0.
        Token::transfer_from(&mut host, addr(A), addr(B), 10).unwrap();
        let t = Token::load(&host).unwrap();
        assert_eq!(
            t.allowances
                .get(&(addr(A), addr(SPENDER)))
                .copied()
                .unwrap(),
            0
        );
    }

    #[test]
    fn transfer_from_without_allowance_rejected() {
        let mut host = new_token(addr(A), 1_000);
        host.set_caller(addr(SPENDER));
        assert_eq!(
            Token::transfer_from(&mut host, addr(A), addr(B), 10),
            Err(ContractError::InsufficientBalance)
        );
    }

    #[test]
    fn transfer_from_insufficient_balance_leaves_allowance_intact() {
        let mut host = new_token(addr(A), 50);
        Token::approve(&mut host, addr(SPENDER), 100).unwrap();
        host.set_caller(addr(SPENDER));
        // Allowance covers it, balance does not.
        assert_eq!(
            Token::transfer_from(&mut host, addr(A), addr(B), 80),
            Err(ContractError::InsufficientBalance)
        );
        let t = Token::load(&host).unwrap();
        // Allowance untouched (store never reached on failure).
        assert_eq!(
            t.allowances
                .get(&(addr(A), addr(SPENDER)))
                .copied()
                .unwrap(),
            100
        );
        assert_eq!(t.balances.get(&addr(A)).copied().unwrap(), 50);
    }

    #[test]
    fn mint_only_minter_and_burn() {
        let mut host = new_token(addr(A), 1_000); // A is owner + minter

        // Non-minter cannot mint.
        host.set_caller(addr(B));
        assert_eq!(
            Token::mint(&mut host, addr(B), 50),
            Err(ContractError::Unauthorized)
        );

        // Minter mints to B.
        host.set_caller(addr(A));
        Token::mint(&mut host, addr(B), 50).unwrap();
        let t = Token::load(&host).unwrap();
        assert_eq!(t.total_supply, 1_050);
        assert_eq!(t.balances.get(&addr(B)).copied().unwrap(), 50);

        // B burns some of their own tokens.
        host.set_caller(addr(B));
        Token::burn(&mut host, 20).unwrap();
        let t = Token::load(&host).unwrap();
        assert_eq!(t.total_supply, 1_030);
        assert_eq!(t.balances.get(&addr(B)).copied().unwrap(), 30);

        // Burning more than balance → rejected.
        assert_eq!(
            Token::burn(&mut host, 100),
            Err(ContractError::InsufficientBalance)
        );
        // Burning zero is a no-op success.
        assert_eq!(Token::burn(&mut host, 0), Ok(()));
    }

    #[test]
    fn mint_overflow_rejected() {
        let mut host = new_token(addr(A), Amount::MAX);
        assert_eq!(
            Token::mint(&mut host, addr(B), 1),
            Err(ContractError::Overflow)
        );
    }

    #[test]
    fn state_survives_new_host_instance() {
        let mut h1 = new_token(addr(A), 1_000);
        Token::transfer(&mut h1, addr(B), 250).unwrap();

        // Second host instance over the SAME storage cell.
        let mut h2 = MemHost::with_shared(h1.shared(), addr(B), addr(CONTRACT));
        let t = Token::load(&h2).unwrap();
        assert_eq!(t.balances.get(&addr(B)).copied().unwrap(), 250);
        Token::transfer(&mut h2, addr(SPENDER), 50).unwrap();
        let t = Token::load(&h2).unwrap();
        assert_eq!(t.balances.get(&addr(SPENDER)).copied().unwrap(), 50);

        // Fresh host cannot see the token.
        let mut fresh = MemHost::new(addr(A), addr(CONTRACT));
        assert_eq!(Token::load(&fresh), Err(ContractError::NotFound));
        assert_eq!(
            Token::transfer(&mut fresh, addr(B), 1),
            Err(ContractError::NotFound)
        );
    }
}
