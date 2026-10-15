# KUNA Gas Cost Schedule (Lane 3b)

> Deterministic WASM metering via wasmi fuel. This schedule defines the canonical gas costs for contract execution. All values are in **gas units** (1 fuel = 1 gas unit).

## 1. ExecutionConfig Parameters

```rust
pub struct ExecutionConfig {
    pub gas_limit: u64,           // Maximum gas per call (default: 10,000,000)
    pub memory_limit_pages: u32,  // Maximum linear memory pages (default: 16 = 1 MiB)
}
```

- `gas_limit` maps directly to wasmi **fuel** (`Store::set_fuel`).
- `memory_limit_pages` enforced by `StoreLimitsBuilder::memory_size()`.

## 2. Base Costs (Intrinsic)

| Operation | Gas Cost | Notes |
|-----------|----------|-------|
| Call base fee | 2,100 | Charged for every contract call (covers host setup, argument copy) |
| Memory page | 3,000 | Per additional 64 KiB page beyond initial allocation |
| Storage read | 200 | Per `kovanica_storage_get` call |
| Storage write | 5,000 | Per `kovanica_storage_set` call (includes namespace hashing) |
| Balance read | 100 | Per `kovanica_balance_of` call |
| Native transfer | 1,500 | Per `kovanica_transfer` call |
| Event emit | 500 | Per `kovanica_emit_event` call |
| Contract creation | 32,000 | `Deploy` transaction (code hash, account init) |

## 3. Host Function Gas Costs (env imports)

Each host function consumes fuel in addition to its base cost:

| Host Function | Base Gas | Additional Per-Call Cost |
|---------------|----------|--------------------------|
| `kovanica_caller` | 0 | 50 |
| `kovanica_contract_address` | 0 | 50 |
| `kovanica_block_height` | 0 | 10 |
| `kovanica_timestamp` | 0 | 10 |
| `kovanica_balance_of` | 100 | 50 |
| `kovanica_transfer` | 1,500 | 200 (includes balance checks) |
| `kovanica_storage_get` (len=0) | 200 | 50 |
| `kovanica_storage_get` (copy) | 200 | 50 + 2 per 32 bytes copied |
| `kovanica_storage_set` | 5,000 | 100 + 2 per 32 bytes written |
| `kovanica_emit_event` | 500 | 20 + 1 per byte of topic+data |

## 4. Wasmi Instruction Fuel (Instruction-Level)

Wasmi consumes fuel per WebAssembly instruction executed. The following table documents the **canonical fuel costs** for the interpreter (these are wasmi internal constants, not configurable, but documented here for auditability):

| Instruction Category | Fuel Cost | Examples |
|---------------------|-----------|----------|
| Numeric (i32/i64/f32/f64) | 1 | `i32.add`, `i64.mul`, `f32.div` |
| Parametric (drop, select) | 1 | `drop`, `select` |
| Variable (local/global get/set) | 2 | `local.get`, `global.set` |
| Memory (load/store) | 5 | `i32.load`, `i64.store` |
| Memory (size/grow) | 100 | `memory.size`, `memory.grow` |
| Control (br/br_if/br_table) | 1 | `br`, `br_if` |
| Call (call/call_indirect) | 10 | `call`, `call_indirect` |
| Return | 1 | `return` |
| Unreachable | 1 | `unreachable` |

**Note**: These are wasmi's internal fuel costs. The `gas_limit` in `ExecutionConfig` sets the total fuel budget. Contracts that exceed the budget trap with `RuntimeError::OutOfGas`.

## 5. Gas Refunds / Unused Gas

- **No refunds**: Unused gas is not returned to the caller. This is intentional for simplicity and DoS resistance.
- The `gas_used` field in `TransactionReceipt` reports `gas_limit - remaining_fuel` at call completion.

## 6. Determinism Guarantees

- **Fuel is deterministic**: wasmi's fuel metering is instruction-accurate and deterministic across runs.
- **No host non-determinism**: Host functions (`kovanica_*`) consume fixed fuel amounts; no `random` or `time` sources are available to contracts.
- **Memory limit is hard**: `memory.grow` beyond `memory_limit_pages` traps deterministically.
- **Argument/result copy**: Bounded by `MAX_BUFFER_BYTES` (64 MiB) in `kovanica-runtime`.

## 7. Fee Calculation (On-Chain)

The **transaction fee** (paid in native KUNA) is separate from the **gas limit**:

```
fee = gas_price * gas_used
```

- `gas_price` is set by the transaction sender (`Transaction.fee` / `gas_limit`).
- `gas_used` is recorded in the receipt.
- Minimum fee floor: `max(1, block_subsidy / 500_000)` atoms per byte of transaction (per tokenomics).

## 8. Configuration Constants (kovanica-node / genesis)

```toml
# config.example.toml (or genesis config)
[execution]
default_gas_limit = 10_000_000
default_memory_limit_pages = 16          # 1 MiB
max_gas_limit = 50_000_000               # Cap per call
max_memory_limit_pages = 256             # 16 MiB cap
```

## 9. Testing Gas Costs

Unit tests in `kovanica-runtime` verify:
- `low_gas_rejects_an_infinite_guest_loop` — fuel exhaustion traps
- `memory_grow_is_limited_by_configured_page_cap` — memory cap enforced
- `packed_contract_error_codes_are_decoded` — error packing deterministic

E2E test in `kovanica-execution` (`wasmi_e2e_kovanica_token_wasm`):
- Native and wasm paths produce identical state transitions
- Gas metering works across both paths

## 10. Future: Gas Cost Governance

Gas costs are **consensus-critical** once activated. Any change requires:
1. RFC proposing the change
2. Activation at a specific block height (via `init_genesis` or upgrade)
3. Migration tooling if existing contracts are affected

---

## Appendix: Fuel-to-Gas Conversion

In the current implementation, **1 wasmi fuel unit = 1 gas unit**. The `ExecutionConfig.gas_limit` is passed directly to `Store::set_fuel()`.

This 1:1 mapping may be adjusted via a multiplicative factor if wasmi's instruction costs prove too granular/coarse relative to desired economic parameters. Such a change would be a hard fork.