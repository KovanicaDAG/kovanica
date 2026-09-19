---
name: rust-defi
description: Use when building DeFi primitives in Rust: AMMs (constant product, concentrated liquidity), lending/borrowing, order books, liquidations, oracle integration, and common DeFi security considerations.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, blockchain, defi, amm, constant-product, lending, borrowing, order-book, liquidation, oracle, twap]
    related_skills: [rust-token-standards, rust-smart-contracts, rust-utxo-ledger, rust-account-ledger]
---

# Rust DeFi

## Overview

DeFi (decentralized finance) is built on smart contracts that implement financial primitives: automated market makers (AMMs), lending/borrowing pools, order books, derivatives, and liquidations. This skill covers the core primitives and their implementation patterns. It's chain-agnostic but uses EVM-style examples for concreteness.

## When to Use

- Implementing an AMM (constant product, concentrated liquidity, etc.)
- Implementing a lending/borrowing pool
- Building an order book (limit orders, market orders)
- Designing liquidations for lending positions
- Integrating price oracles (TWAP, spot, Chainlink-style)
- Understanding DeFi security considerations (flash loans, oracle manipulation, griefing)

**Don't use for:** consensus or basic token transfers. DeFi is the application layer on top of a token/account ledger.

## AMM (Automated Market Maker)

### Constant Product AMM (x * y = k)

The simplest AMM: a pool with two tokens (x and y). The product of the reserves is constant. Swapping x for y decreases x and increases y such that x * y = k.

```rust
#[derive(Debug)]
pub struct ConstantProductPool {
    pub reserve_x: u128,
    pub reserve_y: u128,
    pub total_supply: u128,   // LP token supply
    pub lp_token: Address,    // address of the LP token
}

impl ConstantProductPool {
    /// Swap x for y (sender sends x, receives y)
    pub fn swap_x_for_y(&mut self, amount_x: u128, recipient: Address) -> Result<u128, Error> {
        let amount_x_in = amount_x;   // amount sent in
        let amount_y_out = self.get_amount_out(amount_x_in, Token::X)?;

        // Update reserves
        self.reserve_x = self.reserve_x.checked_add(amount_x_in).ok_or(Error::Overflow)?;
        // amount_y_out is removed from reserve_y
        self.reserve_y = self.reserve_y.checked_sub(amount_y_out).ok_or(Error::Overflow)?;

        // Send y to recipient
        self.send_y(recipient, amount_y_out)?;

        Ok(amount_y_out)
    }

    /// Calculate amount of y received for a given amount of x
    pub fn get_amount_out(&self, amount_in: u128, token_in: Token) -> Result<u128, Error> {
        let reserve_in = match token_in {
            Token::X => self.reserve_x,
            Token::Y => self.reserve_y,
        };
        let reserve_out = match token_in {
            Token::X => self.reserve_y,
            Token::Y => self.reserve_x,
        };

        // x * y = k => new_y = k / new_x
        // amount_out = reserve_out - new_reserve_out
        // new_reserve_out = (reserve_in * reserve_out) / (reserve_in + amount_in)

        let k = (reserve_in as u256) * (reserve_out as u256);
        let new_reserve_in = reserve_in + amount_in;
        let new_reserve_out = (k / new_reserve_in as u256) as u128;

        let amount_out = reserve_out - new_reserve_out;

        // Apply a fee (e.g., 0.3%)
        let fee = amount_out / 1000 * 3;   // 0.3% fee
        let amount_out_after_fee = amount_out - fee;

        if amount_out_after_fee == 0 {
            return Err(Error::InsufficientOutput);
        }

        Ok(amount_out_after_fee)
    }
}
```

**Fee:** constant product AMMs typically take a fee (e.g., 0.3%) on each swap. The fee increases the reserves slightly over time, and LPs earn the fees.

### Liquidity Provision (LP Tokens)

LPs deposit both tokens in proportion to the current pool ratio and receive LP tokens representing their share.

```rust
pub fn add_liquidity(&mut self, amount_x: u128, amount_y: u128) -> Result<u128, Error> {
    // Check that the ratio matches the current pool ratio (within tolerance)
    let ideal_y = (amount_x as u256) * (self.reserve_y as u256) / (self.reserve_x as u256);
    let tolerance = ideal_y / 100;   // 1% tolerance

    if amount_y < ideal_y - tolerance || amount_y > ideal_y + tolerance {
        return Err(Error::RatioMismatch);
    }

    // Update reserves
    self.reserve_x = self.reserve_x.checked_add(amount_x).ok_or(Error::Overflow)?;
    self.reserve_y = self.reserve_y.checked_add(amount_y).ok_or(Error::Overflow)?;

    // Mint LP tokens
    let lp_amount = if self.total_supply == 0 {
        // First liquidity: LP tokens = sqrt(amount_x * amount_y) or similar
        // Simplified: proportional to reserves
        (amount_x + amount_y) / 2   // example, not precise
    } else {
        // Proportional to existing reserves
        let total = self.reserve_x + self.reserve_y;
        let contribution = amount_x + amount_y;
        (self.total_supply as u256 * contribution as u256 / total as u256) as u128
    };

    self.total_supply = self.total_supply.checked_add(lp_amount).ok_or(Error::Overflow)?;

    // Send LP tokens to the provider
    self.mint_lp(lp_amount)?;

    Ok(lp_amount)
}
```

### Price in a Constant Product AMM

The spot price is `reserve_y / reserve_x` (for x → y). As trades happen, the price moves along the curve.

**Slippage:** large trades move the price significantly. The constant product curve ensures that larger trades get worse prices.

### Concentrated Liquidity (Uniswap V3-style)

Concentrated liquidity allows LPs to provide liquidity in a specific price range, improving capital efficiency.

```rust
#[derive(Debug)]
pub struct ConcentratedPosition {
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub liquidity: u128,   // amount of liquidity in this range
    pub owner: Address,
}

pub struct ConcentratedPool {
    pub current_tick: i32,
    pub tick_data: HashMap<i32, TickInfo>,
    pub positions: Vec<ConcentratedPosition>,
}
```

**Concept:** the price moves through "ticks" (discrete price points). Liquidity is active only when the price is within the position's range. As the price crosses a tick, liquidity is added/removed.

**Implementation complexity:** concentrated liquidity is significantly more complex than constant product. It requires tick management, liquidity tracking per tick, and complex swap math.

## Lending / Borrowing

### Pool Model

A lending pool allows users to deposit tokens (become lenders, earn interest) and borrow tokens (become borrowers, pay interest).

```rust
#[derive(Debug)]
pub struct LendingPool {
    pub deposits: HashMap<Address,Deposit>,   // user deposits
    pub borrows: HashMap<Address, Borrow>,     // user borrows
    pub total_deposits: u128,
    pub total_borrows: u128,
    pub interest_rate: InterestRateModel,
    pub collateral_factor: u8,   // e.g., 75% — max loan-to-value
}
```

### Deposit and Borrow

```rust
pub fn deposit(&mut self, amount: u128) -> Result<(), Error> {
    let caller = self.env().caller();
    self.deposits.entry(caller).and_modify(|d| *d += amount).or_insert(amount);
    self.total_deposits += amount;
    // Transfer tokens from caller to pool
    self.transfer_in(caller, amount)?;
    Ok(())
}

pub fn borrow(&mut self, amount: u128, collateral_token: Address) -> Result<(), Error> {
    let caller = self.env().caller();

    // Check collateral: user must have deposited collateral
    let collateral = self.deposits.get(&caller).unwrap_or(&0);
    let max_borrow = (collateral as u128 * self.collateral_factor as u128) / 100;
    if amount > max_borrow {
        return Err(Error::ExceedsBorrowLimit);
    }

    // Check liquidity: pool must have enough tokens
    if self.total_borrows + amount > self.total_deposits {
        return Err(Error::InsufficientLiquidity);
    }

    self.borrows.entry(caller).and_modify(|b| *b += amount).or_insert(amount);
    self.total_borrows += amount;
    self.transfer_out(caller, amount)?;
    Ok(())
}
```

### Interest Rate Model

Interest rates adjust based on utilization (borrows / deposits). Higher utilization → higher rates.

```rust
pub struct InterestRateModel {
    pub base_rate: u8,        // base APY (e.g., 2%)
    pub optimal_utilization: u8,   // e.g., 80%
    pub slope1: u8,           // rate increase below optimal
    pub slope2: u8,           // rate increase above optimal
}

impl InterestRateModel {
    pub fn get_rate(&self, utilization: u8) -> u8 {
        if utilization <= self.optimal_utilization {
            self.base_rate + utilization * self.slope1 / 100
        } else {
            self.base_rate + self.optimal_utilization * self.slope1 / 100
                + (utilization - self.optimal_utilization) * self.slope2 / 100
        }
    }
}
```

**Interest accrual:** interest compounds over time. In a smart contract, interest is typically accrued on each interaction (deposit, borrow, repay) by updating a cumulative rate.

### Liquidation

When a borrower's collateral falls below the required level (due to price movement), the position can be liquidated. A liquidator repays some of the borrow and receives the collateral at a discount.

```rust
pub fn liquidate(&mut self, borrower: Address, repay_amount: u128) -> Result<u128, Error> {
    let borrow = self.borrows.get(&borrower).ok_or(Error::NoBorrow)?;
    let collateral = self.get_collateral_balance(borrower)?;

    // Check if the position is under-collateralized
    let ltv = (*borrow as f64) / (collateral as f64);
    if ltv <= self.collateral_factor as f64 / 100.0 {
        return Err(Error::PositionHealthy);
    }

    // Liquidator repays `repay_amount` of the borrow
    // Receives collateral at a discount (e.g., 5% discount)
    let collateral_to_receive = repay_amount * (100 + LIQUIDATION_DISCOUNT) / 100;

    // Update balances
    self.borrows.insert(borrower, *borrow - repay_amount);
    self.total_borrows -= repay_amount;
    self.transfer_in(repay_amount)?;
    self.transfer_out_collateral(borrower, collateral_to_receive)?;

    Ok(collateral_to_receive)
}
```

**Liquidation discount:** the liquidator receives collateral worth more than the repay amount (a discount), incentivizing liquidations.

## Order Books

An order book matches buy and sell orders. It can be on-chain (all orders in a contract) or off-chain (orders signed off-chain, matched on-chain).

### On-Chain Order Book (simplified)

```rust
#[derive(Debug)]
pub struct Order {
    pub id: u64,
    pub maker: Address,
    pub side: Side,           // Buy or Sell
    pub price: u128,          // price in quote token per base token
    pub amount: u128,
    pub filled: u128,
    pub signed: Signature,    // maker's signature
}

pub struct OrderBook {
    pub bids: Vec<Order>,    // buy orders (sorted by price descending)
    pub asks: Vec<Order>,    // sell orders (sorted by price ascending)
    pub next_order_id: u64,
}
```

### Matching

```rust
pub fn match_orders(&mut self) -> Result<Vec<Trade>, Error> {
    let mut trades = Vec::new();

    while let (Some(bid), Some(ask)) = (self.bids.first(), self.asks.first()) {
        if bid.price >= ask.price {
            let fill_amount = std::cmp::min(bid.amount - bid.filled, ask.amount - ask.filled);
            let trade_price = ask.price;   // or mid-price, depending on design

            // Execute the trade
            self.execute_trade(bid.maker, ask.maker, fill_amount, trade_price)?;

            trades.push(Trade {
                bid_maker: bid.maker,
                ask_maker: ask.maker,
                amount: fill_amount,
                price: trade_price,
            });

            // Update filled amounts, remove filled orders
            // ...
        } else {
            break;   // no more matches possible
        }
    }

    Ok(trades)
}
```

**On-chain order book costs:** every order placement and match is on-chain, which is expensive. Many order books use off-chain order books with on-chain settlement (orders signed off-chain, matched by a keeper, settled on-chain).

## Oracles

DeFi protocols need price feeds. Oracles provide prices (spot, TWAP, aggregated).

### TWAP (Time-Weighted Average Price)

TWAP is a common on-chain price source: the average price over a time window, resistant to short-term manipulation.

```rust
#[derive(Debug)]
pub struct TwapOracle {
    pub price_history: Vec<(u64, u128)>,   // (timestamp, price)
    pub window: u64,                        // window length in seconds
}

impl TwapOracle {
    pub fn update_price(&mut self, price: u128) {
        let now = current_timestamp();
        self.price_history.push((now, price));
        // Prune old entries outside the window
        self.price_history.retain(|(ts, _)| now - *ts <= self.window);
    }

    pub fn get_twap(&self) -> Result<u128, Error> {
        let now = current_timestamp();
        let window_start = now - self.window;

        let relevant: Vec<u128> = self.price_history
            .iter()
            .filter(|(ts, _)| *ts >= window_start)
            .map(|(_, price)| *price)
            .collect();

        if relevant.is_empty() {
            return Err(Error::NoPriceData);
        }

        // Simple average (or weighted by time)
        let sum: u128 = relevant.iter().sum();
        Ok(sum / relevant.len() as u128)
    }
}
```

**TWAP manipulation:** while TWAP is more resistant than spot, a large trade can still manipulate it over a short window. Use longer windows for security.

### Oracle Integration

DeFi protocols integrate with oracles (Chainlink, custom TWAP, etc.) to get prices for collateral valuation, liquidations, and swaps.

```rust
pub fn get_price(token: Address) -> Result<u128, Error> {
    // Use an oracle contract or off-chain oracle feed
    oracle_contract.get_price(token)
}
```

## DeFi Security Considerations

### Flash Loans

Flash loans allow borrowing large amounts without collateral, as long as the borrow is repaid in the same transaction. They can be used to manipulate prices (oracle attacks) or exploit protocol logic.

**Defense:** use TWAP oracles instead of spot prices, or check prices after a delay. Flash loans can't manipulate TWAP over a long window.

### Oracle Manipulation

If a protocol uses a spot price from a DEX as its oracle, an attacker can manipulate the spot price with a large trade and exploit the protocol (e.g., borrow against inflated collateral).

**Defense:** use TWAP, aggregated oracles, or off-chain oracles.

### Griefing

Attackers can grief protocols by front-running, sandwiching, or forcing liquidations.

- **Front-running:** transaction ordering manipulation (MEV).
- **Sandwich attacks:** buy before a victim's trade, sell after (in AMMs).
- **Liquidation griefing:** force liquidations by manipulating prices.

**Mitigation:** commit-reveal schemes, private mempools, MEV-aware designs, liquidation delays.

### Reentrancy

Reentrancy attacks can drain funds if a protocol's functions call external contracts before updating internal state.

**Defense:** checks-effects-interactions pattern, reentrancy guards.

### Access Control

Minting, admin functions, and parameter changes must be properly access-controlled. A single missing check can be catastrophic.

## Verification Checklist

- [ ] Can implement a constant product AMM with swap and liquidity provision
- [ ] Can calculate the spot price and understand slippage in an AMM
- [ ] Can implement a basic lending pool with deposits, borrows, and collateral checks
- [ ] Can explain liquidation and the liquidation discount
- [ ] Can explain TWAP and why it's more resistant to manipulation than spot price
- [ ] Can implement a simple order book with bid/ask matching
- [ ] Understands the risks of flash loans and oracle manipulation
- [ ] Understands the checks-effects-interactions pattern for reentrancy

## Common Pitfalls

1. **Using spot price as the only oracle.** Spot prices can be manipulated in a single block/transaction. Use TWAP or aggregated oracles.

2. **Not checking for overflow/underflow.** DeFi math involves large numbers and fractions. Use checked arithmetic and safe math libraries.

3. **Precision loss in fixed-point math.** Many DeFi protocols use fixed-point representations (e.g., 18 decimals). Rounding errors can accumulate. Use appropriate precision and rounding.

4. **Reentrancy in transfer functions.** If a swap or liquidation triggers a callback to the caller, reentrancy is possible. Use guards.

5. **Allowing infinite approvals.** Infinite approvals give broad permissions. Encourage limited approvals or use permit-style signatures.

6. **Not handling the first liquidity case in AMMs.** The first depositor sets the initial ratio. Handle this case specially.

7. **Liquidation without a discount.** Without a liquidation discount, there's no incentive to liquidate. With too large a discount, liquidators can extract excessive value.

8. **Ignoring MEV / front-running.** Order books and AMMs are susceptible to MEV. Consider commit-reveal or other mitigations if relevant.

9. **Not testing with edge cases (zero reserves, max values, precision limits).** DeFi math has many edge cases. Test thoroughly.

10. **Not considering gas costs in on-chain DeFi.** On-chain order books and complex AMMs have high gas costs. Factor this into design.
