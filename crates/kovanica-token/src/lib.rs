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
//! - singleton pool: `b"kvnc/v1/pool"` (KUNA ↔ token AMM, optional)
//! - LP shares: `b"kvnc/v1/lp/" ++ address` (one `Amount` per provider)
//!
//! Runtime hosts are expected to namespace storage per contract address.
//!
//! ## KUNA ↔ token pool (P0a)
//!
//! `token_init_pool` / `token_buy` / `token_sell` add a constant-product spot
//! market between the native KUNA account of the contract and this token. The
//! native side is the contract's real account balance (`Host::balance_of`);
//! the token side is `Token::balances[contract_address]`. There is no
//! `msg.value` in the wasm ABI, so the KUNA input is passed in `args` and
//! checked against the contract's surplus over the recorded reserve; the
//! signed `Call.value` is what actually funds it. No KUNA is ever minted or
//! burned here — the reserve covers every payout.
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
//! | `token_init_pool`  | `(u128 kuna_in, u128 tokens_in, u16 fee_bps)`     | `u128` lp |
//! | `token_buy`        | `(u128 amount_in, u128 min_out)`                  | `u128` out |
//! | `token_sell`       | `(u128 amount_in, u128 min_out)`                  | `u128` out |
//!
//! All non-pool Ok results are `()` → success return is `0` (null ptr, zero
//! len). The pool entry points return a bincode `u128`.
//! Error packing is identical to kovanica-htlc (crate docs):
//! `-(1 + ContractError::code())` within `-12..=-1`; `kovanica_alloc` /
//! `kovanica_dealloc` are exported alongside.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;

use kovanica_common::events::{
    TOKEN_APPROVAL, TOKEN_BURN, TOKEN_CREATED, TOKEN_MINT, TOKEN_POOL_INITIALIZED, TOKEN_SWAP_BUY,
    TOKEN_SWAP_SELL, TOKEN_TRANSFER,
};
use kovanica_common::{Address, Amount, ContractError, ContractResult, Host};

/// Singleton state key (FIXED): one token instance per contract address.
///
/// The host stores a contract key at `blake3(key)`, so an off-chain reader asks
/// the node for `blake3(STATE_KEY)` (see [`state_slot`]).
pub const STATE_KEY: &[u8] = b"kvnc/v1/state";

/// Storage slot the state is readable at: `blake3(STATE_KEY)`.
///
/// This is the key an operator passes to `kovanica_getStorageAt` to get the raw
/// bytes [`decode_state`] expects, and the key the web wallet uses.
pub fn state_slot() -> [u8; 32] {
    kovanica_common::hash(STATE_KEY)
}

/// Decode raw contract storage into a [`Token`].
///
/// Reads the current layout first, then the v1 layout written before `icon`
/// existed, so every token ever created stays readable off chain.
pub fn decode_state(bytes: &[u8]) -> Option<Token> {
    bincode::deserialize::<Token>(bytes)
        .ok()
        .or_else(|| bincode::deserialize::<TokenV1>(bytes).ok().map(Token::from))
}

/// Singleton KUNA ↔ token pool key (FIXED): one pool per contract address.
///
/// Stored separately from [`STATE_KEY`] so a swap never rewrites the token
/// blob (balances, allowances, icon).
pub const POOL_KEY: &[u8] = b"kvnc/v1/pool";

/// Per-provider LP share key prefix (FIXED): `LP_PREFIX ++ address`.
///
/// Kept outside [`Pool`] for the same reason as [`POOL_KEY`]: LP accounting
/// must not enlarge the blob every swap rewrites.
pub const LP_PREFIX: &[u8] = b"kvnc/v1/lp/";

/// Largest accepted swap fee, in basis points (10 %).
pub const MAX_FEE_BPS: u16 = 1_000;

/// Storage slot the pool is readable at: `blake3(POOL_KEY)`.
///
/// This is the key an operator passes to `kovanica_getStorageAt` to get the
/// raw bytes [`decode_pool`] expects.
pub fn pool_slot() -> [u8; 32] {
    kovanica_common::hash(POOL_KEY)
}

/// Decode raw contract storage into a [`Pool`].
pub fn decode_pool(bytes: &[u8]) -> Option<Pool> {
    bincode::deserialize::<Pool>(bytes).ok()
}

/// Constant-product pool between the contract's native KUNA balance and this
/// token's contract balance.
///
/// Invariant after every successful operation:
/// `token_reserve == Token::balances[contract_address]`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Pool {
    /// Native KUNA atoms backing the pool; always fits in the host's `u64`.
    pub kuna_reserve: Amount,
    /// Token base units held by the contract; the token balance is the source
    /// of truth and this field mirrors it.
    pub token_reserve: Amount,
    /// Swap fee in basis points, retained in the reserve for LP yield.
    pub fee_bps: u16,
    /// Sum of all LP shares.
    pub lp_total: Amount,
    /// Owner switch that stops both swap directions.
    pub paused: bool,
}

impl Pool {
    /// Load the singleton pool, `NotFound` when it was never initialized.
    pub fn load(host: &impl Host) -> ContractResult<Pool> {
        let bytes = host.storage_get(POOL_KEY).ok_or(ContractError::NotFound)?;
        decode_pool(&bytes).ok_or(ContractError::Custom(ERR_BINCODE_DESERIALIZE))
    }

    /// Persist the singleton pool.
    pub fn store(host: &mut impl Host, pool: &Pool) -> ContractResult<()> {
        let bytes =
            bincode::serialize(pool).map_err(|_| ContractError::Custom(ERR_BINCODE_SERIALIZE))?;
        host.storage_set(POOL_KEY, &bytes);
        Ok(())
    }
}

fn lp_key(address: &Address) -> alloc::vec::Vec<u8> {
    let mut key = alloc::vec::Vec::with_capacity(LP_PREFIX.len() + address.len());
    key.extend_from_slice(LP_PREFIX);
    key.extend_from_slice(address);
    key
}

/// LP shares of `address`, `0` when it never provided liquidity.
pub fn lp_shares(host: &impl Host, address: &Address) -> Amount {
    host.storage_get(&lp_key(address))
        .and_then(|bytes| bincode::deserialize::<Amount>(&bytes).ok())
        .unwrap_or(0)
}

/// Persist `amount` LP shares for `address`.
pub fn set_lp_shares(host: &mut impl Host, address: &Address, amount: Amount) {
    // `u128` always serializes, so the only failure is an internal bincode bug.
    if let Ok(bytes) = bincode::serialize(&amount) {
        host.storage_set(&lp_key(address), &bytes);
    }
}

/// Deterministic floor(`sqrt(n)`), integer-only (no float, no panic).
pub fn isqrt(n: Amount) -> Amount {
    if n < 2 {
        return n;
    }
    // `2^ceil(bits/2)` is an upper bound on the root; Newton descends from it.
    let bits = 128 - n.leading_zeros();
    let mut x = 1u128 << bits.div_ceil(2);
    loop {
        let next = (x + n / x) / 2;
        if next >= x {
            return x;
        }
        x = next;
    }
}

/// Constant-product quote with a fee that stays in the reserve (LP yield).
///
/// `in_after = amount_in * (10_000 - fee_bps) / 10_000`, then
/// `out = in_after * r_out / (r_in + in_after)`, floored so the pool never
/// loses to rounding. `None` on invalid fee, zero input/reserve, or overflow.
pub fn quote(amount_in: Amount, r_in: Amount, r_out: Amount, fee_bps: u16) -> Option<Amount> {
    if fee_bps > MAX_FEE_BPS || amount_in == 0 || r_in == 0 || r_out == 0 {
        return None;
    }
    let keep = 10_000u128 - u128::from(fee_bps);
    let in_after = amount_in.checked_mul(keep)?.checked_div(10_000)?;
    if in_after == 0 {
        return Some(0);
    }
    let numerator = in_after.checked_mul(r_out)?;
    let denominator = r_in.checked_add(in_after)?;
    numerator.checked_div(denominator)
}

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

    // ---------- KUNA ↔ token pool (P0a) ----------

    /// Initialize the KUNA ↔ token pool and mint the first LP shares.
    ///
    /// Only `owner` / `minter` may seed it. The native KUNA side is funded by
    /// the signed `Call.value` that precedes this entry point; because the
    /// wasm ABI has no `msg.value`, `kuna_in` is validated against the
    /// contract's balance surplus over its recorded reserve and the whole
    /// surplus is swept into the reserve. Returns the LP shares minted.
    pub fn init_pool(
        host: &mut impl Host,
        kuna_in: Amount,
        tokens_in: Amount,
        fee_bps: u16,
    ) -> ContractResult<Amount> {
        let mut token = Self::load(host)?;
        if host.storage_get(POOL_KEY).is_some() {
            return Err(ContractError::AlreadyExists);
        }
        let caller = host.caller();
        if caller != token.owner && caller != token.minter {
            return Err(ContractError::Unauthorized);
        }
        if kuna_in == 0 || tokens_in == 0 || fee_bps > MAX_FEE_BPS {
            return Err(ContractError::InvalidInput);
        }

        let contract = host.contract_address();
        let surplus = host.balance_of(&contract);
        if kuna_in > surplus {
            return Err(ContractError::InvalidInput);
        }

        let lp = isqrt(
            kuna_in
                .checked_mul(tokens_in)
                .ok_or(ContractError::Overflow)?,
        );
        if lp == 0 {
            return Err(ContractError::InvalidInput);
        }

        Self::move_balance(&mut token, &caller, &contract, tokens_in)?;
        let pool = Pool {
            // Sweep any donation that arrived before initialization.
            kuna_reserve: host.balance_of(&contract),
            token_reserve: token.balances.get(&contract).copied().unwrap_or(0),
            fee_bps,
            lp_total: lp,
            paused: false,
        };

        Self::store(host, &token)?;
        Pool::store(host, &pool)?;
        set_lp_shares(host, &caller, lp);
        host.emit_event(TOKEN_POOL_INITIALIZED, &[]);
        Ok(lp)
    }

    /// Buy tokens with KUNA: `amount_in` is the native amount attached to the
    /// call, `min_out` the slippage floor. Returns the tokens received.
    pub fn buy(host: &mut impl Host, amount_in: Amount, min_out: Amount) -> ContractResult<Amount> {
        let mut token = Self::load(host)?;
        let mut pool = Pool::load(host)?;
        if pool.paused || token.paused {
            return Err(ContractError::Paused);
        }
        if amount_in == 0 {
            return Err(ContractError::InvalidInput);
        }

        let contract = host.contract_address();
        let surplus = host.balance_of(&contract).saturating_sub(pool.kuna_reserve);
        if amount_in > surplus {
            return Err(ContractError::InvalidInput);
        }

        let out = quote(
            amount_in,
            pool.kuna_reserve,
            pool.token_reserve,
            pool.fee_bps,
        )
        .ok_or(ContractError::InvalidInput)?;
        if out == 0 || out < min_out {
            return Err(ContractError::InvalidInput);
        }
        if out > pool.token_reserve {
            return Err(ContractError::InsufficientBalance);
        }

        let caller = host.caller();
        let balance = token.balances.get(&caller).copied().unwrap_or(0);
        let new_balance = balance.checked_add(out).ok_or(ContractError::Overflow)?;
        let contract_balance = token.balances.get(&contract).copied().unwrap_or(0);
        let new_contract = contract_balance
            .checked_sub(out)
            .ok_or(ContractError::InsufficientBalance)?;
        token.balances.insert(caller, new_balance);
        token.balances.insert(contract, new_contract);
        pool.token_reserve -= out;
        // The attached value (plus any donation) is now the reserve.
        pool.kuna_reserve = host.balance_of(&contract);

        Self::store(host, &token)?;
        Pool::store(host, &pool)?;
        host.emit_event(TOKEN_SWAP_BUY, &[]);
        Ok(out)
    }

    /// Sell tokens for KUNA: `amount_in` is taken from the caller's token
    /// balance, `min_out` the slippage floor. Returns the KUNA paid out.
    pub fn sell(
        host: &mut impl Host,
        amount_in: Amount,
        min_out: Amount,
    ) -> ContractResult<Amount> {
        let mut token = Self::load(host)?;
        let mut pool = Pool::load(host)?;
        if pool.paused || token.paused {
            return Err(ContractError::Paused);
        }
        if amount_in == 0 {
            return Err(ContractError::InvalidInput);
        }

        let out = quote(
            amount_in,
            pool.token_reserve,
            pool.kuna_reserve,
            pool.fee_bps,
        )
        .ok_or(ContractError::InvalidInput)?;
        if out == 0 || out < min_out {
            return Err(ContractError::InvalidInput);
        }
        if out > u64::MAX as u128 {
            return Err(ContractError::Overflow);
        }

        let caller = host.caller();
        let contract = host.contract_address();
        Self::move_balance(&mut token, &caller, &contract, amount_in)?;
        pool.token_reserve = pool
            .token_reserve
            .checked_add(amount_in)
            .ok_or(ContractError::Overflow)?;
        pool.kuna_reserve = pool
            .kuna_reserve
            .checked_sub(out)
            .ok_or(ContractError::InsufficientBalance)?;

        Self::store(host, &token)?;
        Pool::store(host, &pool)?;
        // Reserve is already debited; a failed transfer is dropped with the
        // whole overlay by the host, never committed.
        host.transfer(&contract, &caller, out)?;
        host.emit_event(TOKEN_SWAP_SELL, &[]);
        Ok(out)
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

    /// Icon payload rule for `create`: empty, an `https://` URL, or a
    /// `data:image/…` URI, at most [`MAX_ICON_BYTES`], no control characters.
    ///
    /// Public so a client can reject a bad `--icon` before it pays a fee.
    pub fn validate_icon(icon: &str) -> ContractResult<()> {
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
        // Current layout first, then the v1 layout (shared with off-chain
        // readers through `decode_state`), so tokens created by the previous
        // contract build stay readable.
        decode_state(&bytes).ok_or(ContractError::Custom(ERR_BINCODE_DESERIALIZE))
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

    #[no_mangle]
    pub extern "C" fn token_init_pool(args_ptr: i32, args_len: i32) -> i64 {
        let Some(bytes) = (unsafe { args_slice(args_ptr, args_len) }) else {
            return error_code(ContractError::InvalidInput);
        };
        let (kuna_in, tokens_in, fee_bps): (Amount, Amount, u16) = match decode_args(bytes) {
            Ok(v) => v,
            Err(c) => return c,
        };
        let mut host = WasmHost::new();
        match Token::init_pool(&mut host, kuna_in, tokens_in, fee_bps) {
            Ok(lp) => ok_out(&lp),
            Err(e) => error_code(e),
        }
    }

    #[no_mangle]
    pub extern "C" fn token_buy(args_ptr: i32, args_len: i32) -> i64 {
        let Some(bytes) = (unsafe { args_slice(args_ptr, args_len) }) else {
            return error_code(ContractError::InvalidInput);
        };
        let (amount_in, min_out): (Amount, Amount) = match decode_args(bytes) {
            Ok(v) => v,
            Err(c) => return c,
        };
        let mut host = WasmHost::new();
        match Token::buy(&mut host, amount_in, min_out) {
            Ok(out) => ok_out(&out),
            Err(e) => error_code(e),
        }
    }

    #[no_mangle]
    pub extern "C" fn token_sell(args_ptr: i32, args_len: i32) -> i64 {
        let Some(bytes) = (unsafe { args_slice(args_ptr, args_len) }) else {
            return error_code(ContractError::InvalidInput);
        };
        let (amount_in, min_out): (Amount, Amount) = match decode_args(bytes) {
            Ok(v) => v,
            Err(c) => return c,
        };
        let mut host = WasmHost::new();
        match Token::sell(&mut host, amount_in, min_out) {
            Ok(out) => ok_out(&out),
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

        // The same bytes decode off chain, which is what a wallet does with
        // `kovanica_getStorageAt`.
        let decoded = decode_state(&v1).expect("v1 decodes");
        assert_eq!(decoded.symbol, "KUNA");
        assert_eq!(decoded.icon, "");
    }

    #[test]
    fn decode_state_reads_both_layouts_and_rejects_garbage() {
        let host = new_token(addr(A), 700);
        let bytes = host.storage_get(STATE_KEY).unwrap();
        let decoded = decode_state(&bytes).expect("v2 decodes");
        assert_eq!(decoded.total_supply, 700);
        assert_eq!(decoded.owner, addr(A));
        assert!(decode_state(b"not a token").is_none());
    }

    #[test]
    fn state_slot_is_the_host_storage_key() {
        // The host namespaces every contract key as blake3(key); the slot the
        // web wallet and the CLI query must match it byte for byte, so this
        // asserts the published value instead of re-deriving it.
        const HOST_SLOT: [u8; 32] = [
            0xd4, 0x90, 0x05, 0x9b, 0x20, 0xa5, 0x1c, 0xd3, 0x1a, 0xd8, 0x72, 0x40, 0x21, 0x23,
            0x1c, 0x58, 0x1d, 0x67, 0xa6, 0x2d, 0x29, 0x46, 0xf7, 0xf2, 0xa4, 0x40, 0x5d, 0xd8,
            0xae, 0x73, 0x8d, 0xf1,
        ];
        assert_eq!(state_slot(), HOST_SLOT);
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

    // ---------- pool math ----------

    #[test]
    fn isqrt_matches_floor_for_edges_and_large_values() {
        for (n, root) in [
            (0, 0),
            (1, 1),
            (2, 1),
            (3, 1),
            (4, 2),
            (8, 2),
            (9, 3),
            (15, 3),
            (16, 4),
            (17, 4),
        ] {
            assert_eq!(isqrt(n), root, "isqrt({n})");
        }
        assert_eq!(isqrt(u128::from(u64::MAX)), 4_294_967_295);
        assert_eq!(isqrt(Amount::MAX), u128::from(u64::MAX));
        // No panic on a non-square large value either.
        let big = Amount::MAX - 1;
        let root = isqrt(big);
        assert!(root * root <= big);
        assert!(big / (root + 1) < root + 1);
    }

    #[test]
    fn quote_floors_and_keeps_the_fee_in_the_reserve() {
        let (r_in, r_out, fee) = (1_000_000u128, 2_000_000u128, 30u16);
        let out = quote(10_000, r_in, r_out, fee).unwrap();
        let in_after = 10_000 * 9_970 / 10_000;
        assert_eq!(out, in_after * r_out / (r_in + in_after));
        // Floor: the exact product never exceeds the quote.
        assert!(out * (r_in + in_after) <= in_after * r_out);

        // k after the swap (fee retained) is at least k before.
        let k_before = r_in * r_out;
        let k_after = (r_in + 10_000) * (r_out - out);
        assert!(k_after >= k_before);

        assert_eq!(quote(0, r_in, r_out, fee), None);
        assert_eq!(quote(10_000, 0, r_out, fee), None);
        assert_eq!(quote(10_000, r_in, 0, fee), None);
        assert_eq!(quote(10_000, r_in, r_out, 1_001), None);
        // Zero fee leaves the whole input in the curve.
        assert_eq!(
            quote(1_000, r_in, r_out, 0),
            Some(1_000 * r_out / (r_in + 1_000))
        );
        // Too small to move a base unit → Some(0), never a panic.
        assert_eq!(quote(1, 1_000_000_000, 1, 10), Some(0));
        // Overflowing multiplication → None.
        assert_eq!(quote(Amount::MAX, r_in, r_out, fee), None);
    }

    // ---------- pool lifecycle ----------

    fn init_pool(host: &mut MemHost, kuna_in: Amount, tokens_in: Amount, fee: u16) -> Amount {
        host.set_balance(addr(CONTRACT), kuna_in);
        Token::init_pool(host, kuna_in, tokens_in, fee).expect("init_pool")
    }

    #[test]
    fn init_pool_mints_lp_and_moves_tokens_into_the_contract() {
        let mut host = new_token(addr(A), 1_000_000);
        let lp = init_pool(&mut host, 1_000_000, 1_000_000, 30);
        assert_eq!(lp, 1_000_000);

        let token = Token::load(&host).unwrap();
        let pool = Pool::load(&host).unwrap();
        assert_eq!(pool.kuna_reserve, 1_000_000);
        assert_eq!(pool.token_reserve, 1_000_000);
        assert_eq!(pool.fee_bps, 30);
        assert_eq!(pool.lp_total, lp);
        assert!(!pool.paused);
        assert_eq!(token.balances.get(&addr(A)).copied().unwrap(), 0);
        assert_eq!(
            token.balances.get(&addr(CONTRACT)).copied().unwrap(),
            1_000_000
        );
        assert_eq!(pool.token_reserve, balance_of_token(&token, addr(CONTRACT)));
        assert_eq!(lp_shares(&host, &addr(A)), lp);

        let events = host.events();
        assert_eq!(events.last().unwrap().0, TOKEN_POOL_INITIALIZED);
    }

    #[test]
    fn init_pool_rejects_unauthorized_duplicate_and_bad_inputs() {
        let mut host = new_token(addr(A), 1_000_000);
        host.set_balance(addr(CONTRACT), 1_000_000);
        host.set_caller(addr(B));
        assert_eq!(
            Token::init_pool(&mut host, 1_000, 1_000, 30),
            Err(ContractError::Unauthorized)
        );

        host.set_caller(addr(A));
        assert_eq!(
            Token::init_pool(&mut host, 0, 1_000, 30),
            Err(ContractError::InvalidInput)
        );
        assert_eq!(
            Token::init_pool(&mut host, 1_000, 0, 30),
            Err(ContractError::InvalidInput)
        );
        assert_eq!(
            Token::init_pool(&mut host, 1_000, 1_000, 1_001),
            Err(ContractError::InvalidInput)
        );
        // More KUNA than the contract account holds (no attached value).
        assert_eq!(
            Token::init_pool(&mut host, 1_000_001, 1_000, 30),
            Err(ContractError::InvalidInput)
        );

        init_pool(&mut host, 1_000, 1_000, 30);
        assert_eq!(
            Token::init_pool(&mut host, 1_000, 1_000, 30),
            Err(ContractError::AlreadyExists)
        );
    }

    #[test]
    fn buy_moves_reserves_and_credits_the_buyer() {
        let mut host = new_token(addr(A), 1_000_000);
        init_pool(&mut host, 1_000_000, 1_000_000, 30);

        let before = Pool::load(&host).unwrap();
        let expected = quote(10_000, before.kuna_reserve, before.token_reserve, 30).unwrap();
        // Simulate `Call.value`: the contract account grows by amount_in.
        host.set_balance(addr(CONTRACT), before.kuna_reserve + 10_000);
        host.set_caller(addr(B));
        let out = Token::buy(&mut host, 10_000, expected).unwrap();
        assert_eq!(out, expected);

        let token = Token::load(&host).unwrap();
        let after = Pool::load(&host).unwrap();
        assert_eq!(balance_of_token(&token, addr(B)), out);
        assert_eq!(after.token_reserve, before.token_reserve - out);
        assert_eq!(after.kuna_reserve, before.kuna_reserve + 10_000);
        assert_eq!(after.lp_total, before.lp_total);
        assert_eq!(
            after.token_reserve,
            balance_of_token(&token, addr(CONTRACT))
        );
        assert_eq!(host.events().last().unwrap().0, TOKEN_SWAP_BUY);
    }

    #[test]
    fn buy_rejects_bad_slippage_and_unattached_value_without_mutating() {
        let mut host = new_token(addr(A), 1_000_000);
        init_pool(&mut host, 1_000_000, 1_000_000, 30);
        let before_token = Token::load(&host).unwrap();
        let before_pool = Pool::load(&host).unwrap();
        let expected = quote(
            10_000,
            before_pool.kuna_reserve,
            before_pool.token_reserve,
            30,
        )
        .unwrap();

        host.set_balance(addr(CONTRACT), before_pool.kuna_reserve + 10_000);
        host.set_caller(addr(B));
        assert_eq!(
            Token::buy(&mut host, 10_000, expected + 1),
            Err(ContractError::InvalidInput)
        );
        assert_eq!(Token::load(&host).unwrap(), before_token);
        assert_eq!(Pool::load(&host).unwrap(), before_pool);
        assert_eq!(host.balance_of(&addr(B)), 0);

        // Claiming more than the contract's surplus over the reserve fails.
        assert_eq!(
            Token::buy(&mut host, 10_001, 0),
            Err(ContractError::InvalidInput)
        );
        assert_eq!(Token::load(&host).unwrap(), before_token);
        assert_eq!(Pool::load(&host).unwrap(), before_pool);
    }

    #[test]
    fn buy_sweeps_a_donation_into_the_reserve() {
        let mut host = new_token(addr(A), 1_000_000);
        init_pool(&mut host, 1_000_000, 1_000_000, 30);
        let before = Pool::load(&host).unwrap();

        // Attached value plus a 500-atom donation.
        host.set_balance(addr(CONTRACT), before.kuna_reserve + 10_000 + 500);
        host.set_caller(addr(B));
        Token::buy(&mut host, 10_000, 0).unwrap();

        let after = Pool::load(&host).unwrap();
        assert_eq!(after.kuna_reserve, before.kuna_reserve + 10_500);
    }

    #[test]
    fn sell_pays_native_kuna_and_moves_the_token_side() {
        let mut host = new_token(addr(A), 1_000_000);
        init_pool(&mut host, 1_000_000, 1_000_000, 30);
        host.set_balance(addr(CONTRACT), 1_000_000 + 10_000);
        host.set_caller(addr(B));
        let bought = Token::buy(&mut host, 10_000, 0).unwrap();

        let before_pool = Pool::load(&host).unwrap();
        let sell = bought / 2;
        let expected = quote(
            sell,
            before_pool.token_reserve,
            before_pool.kuna_reserve,
            before_pool.fee_bps,
        )
        .unwrap();
        let native_before = host.balance_of(&addr(B));
        let out = Token::sell(&mut host, sell, expected).unwrap();
        assert_eq!(out, expected);

        let token = Token::load(&host).unwrap();
        let after = Pool::load(&host).unwrap();
        assert_eq!(balance_of_token(&token, addr(B)), bought - sell);
        assert_eq!(after.token_reserve, before_pool.token_reserve + sell);
        assert_eq!(after.kuna_reserve, before_pool.kuna_reserve - out);
        assert_eq!(host.balance_of(&addr(B)), native_before + out);
        assert_eq!(after.lp_total, before_pool.lp_total);
        assert_eq!(
            after.token_reserve,
            balance_of_token(&token, addr(CONTRACT))
        );
        assert_eq!(host.events().last().unwrap().0, TOKEN_SWAP_SELL);
    }

    #[test]
    fn sell_without_tokens_leaves_state_untouched() {
        let mut host = new_token(addr(A), 1_000_000);
        init_pool(&mut host, 1_000_000, 1_000_000, 30);
        let before_token = Token::load(&host).unwrap();
        let before_pool = Pool::load(&host).unwrap();

        host.set_caller(addr(B));
        assert_eq!(
            Token::sell(&mut host, 1_000, 0),
            Err(ContractError::InsufficientBalance)
        );
        assert_eq!(Token::load(&host).unwrap(), before_token);
        assert_eq!(Pool::load(&host).unwrap(), before_pool);
        assert_eq!(host.balance_of(&addr(CONTRACT)), 1_000_000);
    }

    #[test]
    fn paused_pool_and_token_block_both_directions() {
        let mut host = new_token(addr(A), 1_000_000);
        init_pool(&mut host, 1_000_000, 1_000_000, 30);
        host.set_caller(addr(B));

        let mut pool = Pool::load(&host).unwrap();
        pool.paused = true;
        Pool::store(&mut host, &pool).unwrap();
        assert_eq!(Token::buy(&mut host, 1, 0), Err(ContractError::Paused));
        assert_eq!(Token::sell(&mut host, 1, 0), Err(ContractError::Paused));

        pool.paused = false;
        Pool::store(&mut host, &pool).unwrap();
        let mut token = Token::load(&host).unwrap();
        token.paused = true;
        Token::store(&mut host, &token).unwrap();
        assert_eq!(Token::buy(&mut host, 1, 0), Err(ContractError::Paused));
        assert_eq!(Token::sell(&mut host, 1, 0), Err(ContractError::Paused));
    }

    #[test]
    fn missing_pool_is_not_found() {
        let mut host = new_token(addr(A), 1_000_000);
        assert_eq!(Token::buy(&mut host, 1, 0), Err(ContractError::NotFound));
        assert_eq!(Token::sell(&mut host, 1, 0), Err(ContractError::NotFound));
    }

    #[test]
    fn huge_amounts_overflow_cleanly() {
        let mut host = new_token(addr(A), Amount::MAX);
        // kuna_in * tokens_in overflows → Overflow, no panic.
        host.set_balance(addr(CONTRACT), Amount::MAX);
        assert_eq!(
            Token::init_pool(&mut host, Amount::MAX, Amount::MAX, 30),
            Err(ContractError::Overflow)
        );
        // A sane pool, then a quote whose multiplication overflows → InvalidInput.
        init_pool(&mut host, 1_000, 1_000, 30);
        host.set_balance(addr(CONTRACT), Amount::MAX);
        assert_eq!(
            Token::buy(&mut host, Amount::MAX, 0),
            Err(ContractError::InvalidInput)
        );
        assert_eq!(Pool::load(&host).unwrap().token_reserve, 1_000);
    }

    #[test]
    fn pool_slot_is_blake3_of_the_pool_key_and_differs_from_state() {
        assert_eq!(pool_slot(), kovanica_common::hash(POOL_KEY));
        assert_eq!(pool_slot(), kovanica_common::hash(b"kvnc/v1/pool"));
        assert_ne!(pool_slot(), state_slot());
    }

    fn balance_of_token(token: &Token, address: Address) -> Amount {
        token.balances.get(&address).copied().unwrap_or(0)
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
