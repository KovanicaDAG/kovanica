---
name: rust-embedded
description: Use when writing Rust for embedded targets: no_std, embedded-hal, cortex-m, cortex-a, esp-riscv, defmt, panic handlers, memory layout, linker scripts, and embedded async.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, embedded, no_std, cortex-m, esp32, defmt, linker, panic-handler, embedded-hal]
    related_skills: [rust-basics, rust-async]
---

# Rust Embedded

## Overview

Embedded Rust targets systems without an OS (`no_std`), with constrained memory, specific hardware (microcontrollers, single-board computers), and direct hardware access. The ecosystem centers on `embedded-hal` (hardware abstraction traits), `cortex-m`/`cortex-a` (ARM support), `esp-riscv` / `esp-hal` (ESP32), and `defmt` (deferred formatting for efficient logging).

Embedded Rust is a different mindset from application Rust: no heap, no standard library, careful memory layout, and the compiler must know the exact target.

## When to Use

- Writing firmware for a microcontroller (ARM Cortex-M, ESP32, RISC-V)
- Building a `no_std` library that runs on bare-metal
- Setting up a new embedded project with the right linker script and memory layout
- Using `defmt` for logging instead of `println!` (which requires stdout)
- Configuring interrupts, GPIO, clocks, and peripherals through HAL crates

**Don't use for:** Linux applications running on embedded hardware (those are `std` applications), or desktop/embedded cross-over where `std` is available.

## `no_std` — Running Without the Standard Library

```toml
# Cargo.toml
[profile.release]
panic = "abort"   # no unwinding on embedded — smaller and faster
```

```rust
// lib.rs or main.rs
#![no_std]
#![no_main]   // if using a custom entry point (e.g., cortex-m)

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}   // or defmt::panic, or blink an LED, or log and loop
}
```

`#![no_std]` removes the standard library. You get `core` (the subset of `std` without OS dependencies) and can add `alloc` for a heap (if you have one).

### `core` vs `std`

| `std` | `core` | Notes |
|---|---|---|
| `Vec`, `String`, `Box` | `alloc::Vec`, `alloc::String`, `alloc::Box` | with `#![feature(alloc)]` or `extern crate alloc` |
| `println!` | `defmt::info!` / `uretic::println` | depends on platform |
| `HashMap` | `hashbrown` or `no_std` hashmap crate | `core` doesn't have HashMap |
| `thread` / `std::net` | — | no OS, no threads or networking in core |
| `Iterator`, `Option`, `Result`, `mem`, `ptr` | Same in `core` | `core` has the essentials |

**Importing `alloc`:**

```rust
#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
```

## Memory Layout and Linker Scripts

Embedded targets need a linker script that places sections at specific addresses (Flash for code, RAM for data).

```toml
# .cargo/config.toml
[build]
target = "thumbv7em-none-eabihf"   # Cortex-M4 with FPU, example

[target.thumbv7em-none-eabihf]
rustflags = [
  "-C", "link-arg=--nmagic",
  "-C", "link-arg=-Tlink.x",   # use the linker script from the target crate
]
```

### Memory Map (Conceptual)

```
Flash (ROM):
  0x08000000 — reset vector
  0x08000004 — initial stack pointer
  0x08000008+ — code

RAM:
  0x20000000 — .bss (zero-initialized data)
  0x20000000+ — .data (initialized data, copied from Flash at startup)
  0x20000000+ — .stack
```

The exact addresses depend on the chip. Use the chip's reference manual.

### Common Linker Script Features

- Place `.vector_table` at the start of Flash (for Cortex-M).
- Place `.bss` in RAM, zeroed at startup.
- Place `.data` in RAM, initialized from Flash copy at startup.
- Define stacks (main stack, process stack for Cortex-M).

Use `cortex-m-rt`'s `link.x` script as a base for Cortex-M projects.

## Entry Point and `cortex-m-rt`

For Cortex-M microcontrollers:

```toml
[dependencies]
cortex-m = "0.7"
cortex-m-rt = "0.7"
```

```rust
#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception, ExceptionFrame};
use cortex_m::asm;

#[entry]
fn main() -> ! {
    // Initialization (clocks, GPIO, etc.)
    // ...

    loop {
        // main loop
    }
}

#[exception]
fn DefaultHandler(irqn: i16) {
    // Default exception handler (for unhandled interrupts)
}

#[exception]
fn HardFault(_, _frame: &ExceptionFrame) -> ! {
    // Hard fault handler — usually log and loop
    loop {}
}
```

`#[entry]` marks the reset handler. `#[exception]` marks interrupt handlers.

## `embedded-hal` — Hardware Abstraction

```toml
[dependencies]
embedded-hal = "1"
embedded-hal-busy = "0.1"   # for blocking/timeout patterns
```

`embedded-hal` provides traits for common peripherals: GPIO, UART, I2C, SPI, ADC, PWM, timers. Implementations are provided by HAL crates for specific chips.

```rust
use embedded_hal::digital::OutputPin;
use embedded_hal::prelude::*;

fn blink<P: OutputPin>(led: &mut P) -> Result<(), P::Error> {
    led.set_high()?;      // LED on
    cortex_m::asm::delay(1_000_000);   // busy-wait delay (platform-specific)
    led.set_low()?;       // LED off
    cortex_m::asm::delay(1_000_000);
    Ok(())
}
```

**`embedded-hal` traits** (1.x):
- `digital::InputPin`, `digital::OutputPin`, `digital::Toggleable`
- `serial::Read`, `serial::Write`
- `i2c::I2c`
- `spi::SpiDevice`
- `adc::Read`
- `pwm::SetDutyCycle`
- `timer::Delay`

Implementations vary by chip — consult the specific HAL crate.

## `defmt` — Deferred Formatting Logger

```toml
[dependencies]
defmt = "0.3"
defmt-rtt = "0.4"   # transport via RTT (Segger RTT)
```

```rust
#![no_std]
#![no_main]

use defmt::prelude::*;
use defmt_rtt as _;   // for RTT transport

#[cortex_m_rt::entry]
fn main() -> ! {
    let x = 42;
    defmt::info!("Starting up, x = {}", x);
    defmt::error!("Something went wrong");

    loop {}
}
```

**Why defmt over println:**
- `println!` requires a serial implementation and blocks on I/O.
- `defmt` formats at compile time into binary strings, transmits efficiently, and is decoded on the host side.
- Much faster and less memory-intensive than `println!`.

### Host-side decoding

```bash
# With probe-rs
probe-rs rtt --select-uart 0   # view RTT output

# Or defmt-print
defmt-print -c "target/defmt.json"   # decode binary logs
```

`defmt::panic!` is the equivalent of `panic!` in defmt context:

```rust
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    defmt::panic!("Panicked: {}", info);
}
```

## Chip-Specific HALs

| Platform | Crate(s) |
|---|---|
| ARM Cortex-M (generic) | `cortex-m`, `cortex-m-rt` |
| STM32 (various) | `stm32f4xx-hal`, `stm32g0xx-hal`, etc. (per series) |
| ESP32 (Xtensa) | `esp-hal`, `esp-riscv` or `esp32-hal` |
| ESP32-C3/C6 (RISC-V) | `esp-hal` |
| nRF52 (Nordic) | `nrf-hal` |
| RP2040 (Raspberry Pi Pico) | `rp2040-hal` |
| RISC-V generic | `riscv` |

Each chip family has its own HAL crate that implements `embedded-hal` traits for the chip's peripherals.

## Interrupts

For Cortex-M, `cortex-m-rt` handles interrupt vectors. Define handlers with `#[interrupt]`:

```rust
use cortex_m_rt::interrupt;

#[interrupt]
fn USART1() {
    // Handle USART1 interrupt
    // Clear flags, read data, etc.
}
```

Interrupt handlers must be fast. Do minimal work in the handler; defer processing to the main loop or a task queue.

## Embedded Async

Async on embedded is possible but constrained: no OS thread scheduler, so async must be driven by a timer or interrupt.

```toml
[dependencies]
embassy-executor = "0.5"
embassy-time = "0.3"
```

`embassy` is a modern async embedded framework:

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize hardware
    // ...

    loop {
        do_something().await;
        Timer::after_millis(1000).await;   // non-blocking delay
    }
}
```

**Considerations:**
- Async on embedded requires a runtime (embassy-executor) that's driven by hardware timers or interrupts.
- Not all peripherals have async drivers.
- Memory usage is higher than bare-metal (future storage).

## Panic and Error Handling in `no_std`

```rust
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    defmt::panic!("{}", info);
    loop {}
}
```

Without unwinding (panic = "abort"), panics don't run destructors. Be careful with resources that need cleanup — use `defmt::panic!` or explicit error handling with `Result` before panicking.

## Testing Embedded Code

Embedded tests are harder than desktop tests. Strategies:

- **Host-side tests** of pure logic: write unit tests that run on `x86_64` (pure functions, no hardware access).
- **QEMU** for some targets: `cargo test --target thumbv7em-none-eabihf` with QEMU running the binary.
- **Hardware-in-the-loop**: run tests on the actual board, capture defmt output.

For host-side logic tests:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_parser() {
        // pure logic, no hardware — test on host
    }
}
```

## Verification Checklist

- [ ] Can set up a `no_std` crate with `#![no_std]`, `#![no_main]`, and a `#[panic_handler]`
- [ ] Can configure a target in `.cargo/config.toml` and install the target with `rustup target add`
- [ ] Can use `cortex-m-rt` entry point and exception handlers
- [ ] Can use `embedded-hal` traits to write chip-agnostic peripheral code
- [ ] Can use `defmt` for logging and understand its compile-time formatting
- [ ] Understands the memory layout (Flash for code, RAM for data/stack)
- [ ] Can write a basic blinky program for a Cortex-M chip
- [ ] Can use `embassy` for async embedded where appropriate
- [ ] Knows that `std` types (Vec, String, HashMap) require `alloc` in `no_std`
- [ ] Understands that embedded code has no heap by default — use static allocation or a fixed-size arena

## Common Pitfalls

1. **Using `std` types without `alloc`.** `Vec`, `String`, `Box` are in `alloc`, not `core`. In `no_std` without `alloc`, they're not available.

2. **Forgetting to zero `.bss`.** The linker script and startup code must zero `.bss` before `main`. `cortex-m-rt` handles this, but custom entry points must do it manually.

3. **Not initializing `.data`.** Initialized static variables must be copied from Flash to RAM at startup. If the startup code doesn't do this, they're zero or garbage.

4. **Using `println!` in `no_std`.** `println!` requires `std` or a platform-specific implementation. Use `defmt` or a UART implementation.

5. **Assuming `#[interrupt]` handlers are reentrant.** Each interrupt handler runs to completion unless it yields. Long handlers block other interrupts. Keep them short.

6. **Forgetting to enable the peripheral clock.** On many chips (STM32, etc.), peripherals are clock-gated by default. Enable the clock before accessing the peripheral registers.

7. **Not handling the case where a peripheral is already in use.** If multiple parts of the code access the same peripheral, ensure exclusive access (via ownership, Mutex, or careful design).

8. **Using busy-wait delays everywhere.** Busy-wait (`asm::delay`) blocks the CPU. Use timers and interrupts for non-blocking operation, or use async with embassy.

9. **Ignoring the stack size.** Embedded systems have limited RAM. The stack size is configured in the linker script. If the stack overflows, behavior is undefined (corruption, hard fault).

10. **Testing only on host.** Embedded code that compiles on host may not work on the target (different endianness, alignment, timing). Test on the actual hardware or in QEMU.

## Quick Reference

| Task | Tool/Crate |
|---|---|
| Bare-metal entry | `cortex-m-rt` (`#[entry]`) |
| Hardware abstraction | `embedded-hal` + chip HAL crate |
| Logging | `defmt` (+ transport crate like `defmt-rtt`) |
| Panic handler | custom `#[panic_handler]` or `defmt::panic` |
| Async | `embassy-executor` + `embassy-time` |
| Target config | `.cargo/config.toml` + `rustup target add` |
| Linker script | chip-specific or `cortex-m-rt` `link.x` |
| Stack/heap | configured in linker script; `alloc` for heap |
| Testing | host-side for pure logic, QEMU or hardware for full integration |
