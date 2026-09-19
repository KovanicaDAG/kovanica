---
name: rust-advanced
description: Use when writing non-trivial Rust: traits as bounds/associated types, generics, lifetimes in structs and functions, DST/unsized types, coercion, variance, and when the compiler needs explicit lifetime annotations.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, traits, generics, lifetimes, DST, variance, coercion]
    related_skills: [rust-basics, rust-async, rust-macros, rust-ffi-unsafe]
---

# Rust Advanced

## Overview

Once ownership and borrowing are second nature, Rust's surface area shifts to: expressing abstractions with traits and generics, making the borrow checker work with complex data structures, and using the type system to catch more bugs at compile time. This skill covers the features that separate "I can write Rust" from "I can write good, idiomatic, flexible Rust."

## When to Use

- Designing APIs that take/return generic types or trait objects
- Writing structs that hold references (struct lifetime parameters)
- Needing `impl Trait` in return position, generic bounds, or where clauses
- Working with unsized types, trait objects, or dynamic dispatch tradeoffs
- Figuring out why lifetime elision doesn't apply and why you need to write `<'a>`
- Coercions: `&mut T` → `&T`, `T` → `&T`, `Vec<T>` → `IntoIterator`, unsized coercion
- Variance questions (why `&'a T` is covariant in `'a` and `T`, `&mut T` is invariant in `T`)
- Associated types vs generic parameters on traits

**Don't use for:** async (separate skill), macros (separate skill), or performance tuning (separate skill).

## Traits: Design and Use

### Trait Bounds vs `impl Trait` vs Trait Objects

Three ways to use a trait in a function signature, each with different semantics:

```rust
// Trait bound — monomorphized, static dispatch, caller picks type
fn process<T: Summary>(items: &[T]) { /* ... */ }

// `impl Trait` in argument position — sugar for the above, same semantics
fn process(items: &[impl Summary]) { /* ... */ }

// `impl Trait` in return position — opaque type, compiler chooses, callee picks
fn make_summary() -> impl Summary {
    User { name: "x".into(), active: true }
}

// Trait object — dynamic dispatch, type erased, runs through vtable
fn process_dyn(items: &[Box<dyn Summary>]) { /* ... */ }
```

**Decision tree:**
- Does the caller need to choose the concrete type? → generic bound or `impl Trait` in arg.
- Does the callee want to hide the concrete type from the caller? → `impl Trait` in return.
- Do you need to store heterogeneous types that all implement the trait in one collection? → `Box<dyn Trait>` (or `&dyn Trait` for borrowed).
- Is performance critical and the set of types small/fixed? → generics / static dispatch.
- Is the set of types open-ended and you don't care about the small vtable cost? → trait objects.

### Associated Types vs Generic Parameters

```rust
// BAD — generic parameter on trait, caller must specify
trait Container<T> {
    fn get(&self) -> Option<&T>;
}

// GOOD — associated type, Single-SAT interpretation
trait Container {
    type Item;
    fn get(&self) -> Option<&Self::Item>;
}
```

Use associated types when a trait implementation has exactly one meaningful type for that role (Iterator → Item, Iterator → exactly one type). Use generic parameters when the same trait can be implemented multiple times for different type parameters (like `Add<Rhs>` — you can add many different things to the same type).

### Supertraits

```rust
trait Summary: Display + Sized {
    fn summarize(&self) -> String;
}
```

A trait with supertraits requires implementors to also implement the supertraits. Use this when your trait's default methods rely on other traits.

### Default Methods

```rust
trait Summary {
    fn summarize(&self) -> String;

    // default implementation — implementors can override
    fn summarize_with_author(&self, author: &str) -> String {
        format!("{} by {}", self.summarize(), author)
    }
}
```

## Generics

### Bounds and `where` Clauses

```rust
// inline bounds
fn max<T: PartialOrd>(a: T, b: T) -> T { /* ... */ }

// where clause — cleaner for multiple bounds or complex bounds
fn process<T, U>(a: T, b: U)
where
    T: Display + Clone,
    U: Into<T>,
    T: Debug,
{ /* ... */ }
```

Use `where` clauses when:
- You have more than 2-3 bounds.
- Bounds involve associated types or complex expressions like `T: Trait<Assoc = U>`.
- You want to separate bounds from the type parameter list for readability.

### Generic vs `impl Trait` in Return Position

```rust
// Generic return — caller specifies the type (usually via turbofish or inference)
fn parse<T: FromStr>(s: &str) -> Result<T, T::Err> { /* ... */ }

// opaque return — type is hidden, but the compiler knows it implements Trait
fn make iterable() -> impl Iterator<Item = i32> {
    vec![1, 2, 3].into_iter()
}
```

`impl Trait` in return position creates an **opaque type** — the concrete type exists but is hidden from the caller. The caller can only use the API guaranteed by the trait bounds. This is how you write "returns some type that implements Iterator" without exposing the concrete type.

**Caveat:** `impl Trait` in return position cannot be used in recursive functions (the opaque type would be infinite), and you can't return different concrete types from different branches:

```rust
// ERROR — different concrete types from branches
fn cond() -> impl Display {
    if condition() {
        "string"        // &str
    } else {
        42u32           // u32
    }
}
// Fix: return a common type (String, or Box<dyn Display>)
```

### Const Generics

```rust
// Array type with compile-time size parameter
struct FixedBuffer<T, const N: usize> {
    data: [T; N],
}

// Usage
let buf: FixedBuffer<u8, 1024> = FixedBuffer { data: [0; 1024] };
```

Const generics are useful for:
- Fixed-size arrays that you don't want to `Box` or `Vec`
- SIMD buffer types
- Abstracting over `[T; N]` for different N

### Generic Associated Types (GATs)

GATs let traits define associated types that are themselves generic:

```rust
trait LendingIterator {
    type Item<'a> where Self: 'a;
    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}
```

This is advanced; needed for iterators that lend items with a lifetime tied to the iterator borrow (the `streaming iterator` pattern). Stable since Rust 1.65.

## Lifetimes: Beyond the Basics

### Struct Lifetimes

A struct that holds references must be parameterized by the lifetime of those references:

```rust
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, pos: 0 }
    }

    fn rest(&self) -> &'a str {
        &self.input[self.pos..]
    }
}
```

The lifetime `'a` ties the struct's lifetime to the lifetime of the borrowed data. The compiler ensures `Parser` cannot outlive the `&'a str` it holds.

### Multiple Lifetime Parameters

When a function returns a reference and there are multiple input references, you may need to disambiguate which input the output borrows from:

```rust
// Without annotation, compiler doesn't know if output borrows from x or y
// fn longest(x: &str, y: &str) -> &str { ... }   // ERROR

fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

This says: the returned reference lives at least as long as the shorter of the two inputs (since both are `&'a str`, the output is tied to the intersection).

**When elision fails:** when a function has multiple input references and returns a reference, the compiler can't apply elision rule 2 (one input lifetime → all outputs) because there isn't one input lifetime. You must write the lifetime.

### `'static` Lifetime

`&'static str` means the reference lives for the entire program (e.g., string literals). `T: 'static` means the type doesn't contain any non-`'static` references.

```rust
let s: &'static str = "literal";   // string literals are 'static

fn require_static<T: 'static>(t: T) { /* ... */ }
```

Be careful: `T: 'static` is often used as "thread-safe to send to another thread" but `Send + Sync` is the correct bound for that. `'static` means "no borrows that could be invalidated."

### Lifetime Subtyping and Variance (conceptual)

Lifetimes form a subtyping relationship: `'long: 'short` means `'long` outlives `'short`. This is used by the compiler to allow:

```rust
fn shorten<'long, 'short>(x: &'long str) -> &'short str
where
    'long: 'short,
{
    x   // OK — 'long outlives 'short
}
```

Variance explains why:
- `&'a T` is covariant in both `'a` and `T` — you can use a longer-lived reference where a shorter one is expected.
- `&mut T` is invariant in `T` — you cannot substitute a `&mut String` where `&mut &str` is expected (mutation could write a `&str` into a slot expecting `&String`). This prevents unsoundness.

**Practical takeaway:** if you're fighting the borrow checker with a `&mut` reference that should "obviously" work, variance (invariance) is often the reason.

## Unsized Types and Coercions

### `?Sized` Bound

Most generic parameters have an implicit `Sized` bound. Use `?Sized` to allow unsized types:

```rust
// ERROR without ?Sized — str is not Sized
fn print<T: Display>(t: T) { println!("{}", t); }   // T: Sized implicit

// OK with ?Sized — allows &str, slices, trait objects
fn print_ref<T: Display + ?Sized>(t: &T) { println!("{}", t); }
```

Common unsized types:
- `str` (use `&str` or `Box<str>`)
- `[T]` (use `&[T]` or `Box<[T]>`)
- Trait objects `dyn Trait` (use `&dyn Trait`, `Box<dyn Trait>`, `Rc<dyn Trait>`)

### Coercions

Rust performs several coercions automatically:

```rust
// Deref coercion: &String → &str, &Vec<T> → &[T], &Rc<T> → &T
let s: String = "hello".into();
let r: &str = &s;        // &String coerces to &str via Deref

let v: Vec<i32> = vec![1, 2, 3];
let slice: &[i32] = &v;  // &Vec<i32> coerces to &[i32]

// Function pointer coercion: fn(x: i32) -> i32 → FnMut(i32) -> i32
fn add_one(x: i32) -> i32 { x + 1 }
let closure: impl FnMut(i32) -> i32 = add_one;  // fn coerces to closure type

// Unsizing coercion: [T; N] → [T] via Deref, Box<[T; N]> → Box<[T]>
let arr: [i32; 3] = [1, 2, 3];
let boxed: Box<[i32]> = Box::new(arr);  // unsized coercion
```

## Advanced Pattern Matching

### Match Guards

```rust
match x {
    n if n < 0 => println!("negative"),
    n if n > 100 => println!("big"),
    n => println!("normal: {n}"),
}
```

### `@` Bindings

```rust
match x {
    n @ 1..=5 => println!("small: {n}"),
    n @ 6..=10 => println!("medium: {n}"),
    _ => println!("other"),
}
```

### Or Patterns

```rust
match char {
    'a' | 'e' | 'i' | 'o' | 'u' => println!("vowel"),
    _ => println!("consonant or other"),
}
```

### If Let Chains (Rust 1.65+)

```rust
// before: nested if let
if let Some(a) = opt_a {
    if let Some(b) = opt_b {
        if a == b { /* ... */ }
    }
}

// after: if let chains
if let Some(a) = opt_a
    && let Some(b) = opt_b
    && a == b
{
    /* ... */
}
```

## Advanced Iterator Patterns

### Custom Iterator

```rust
struct Counter {
    current: i32,
    max: i32,
}

impl Iterator for Counter {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.max {
            None
        } else {
            self.current += 1;
            Some(self.current)
        }
    }
}
```

### Iterator from a closure (std since 1.78+ via `iter::from_fn`)

```rust
let mut state = 0;
let iter = std::iter::from_fn(|| {
    state += 1;
    if state > 5 { None } else { Some(state) }
});
```

### `FromIterator` and `collect()` targets

`collect()` works on any type implementing `FromIterator`. Common targets:
- `Vec<T>`
- `HashSet<T>`, `BTreeSet<T>`
- `String` (from `Iterator<Item = char>` or `Iterator<Item = &str>`)
- `HashMap<K, V>` (from `Iterator<Item = (K, V)>`)
- `Result<Vec<T>, E>` (fails on first `Err`)
- `Option<Vec<T>>` (fails on first `None` with `collect::<Option<Vec<_>>>()`)
- `ControlFlow` (early termination)

### `try_fold`, `try_for_each`, `reduce`, `try_reduce`

```rust
// try_fold — short-circuits on Err/None
let sum: Result<i32, ParseIntError> = iter.try_fold(0, |acc, s| {
    Ok(acc + s.parse()?)
});

// reduce — binary operator over iterator, no initial value
let sum = iter.reduce(|a, b| a + b);  // Option<i32>

// try_reduce — short-circuiting reduce
let max = iter.try_reduce(|a, b| if a > b { a } else { b });
```

## Advanced Error Handling

### Custom Error Types with `thiserror`

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error at position {pos}: {msg}")]
    Parse { pos: usize, msg: String },

    #[error("Validation failed: {reason}")]
    Validation { reason: String },

    // wrap another error type while adding context
    #[error("config error")]
    Config {
        #[source]
        source: std::io::Error,
        path: String,
    },
}
```

`#[from]` auto-implements `From<std::io::Error>` for `AppError`, enabling `?` propagation. `#[source]` marks the error's cause for `Error::source()`.

### `anyhow` for applications

```rust
use anyhow::{Context, Result, bail, ensure};

fn read_config(path: &str) -> Result<String> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config from {path}"))?;
    Ok(contents)
}

fn validate_age(age: i32) -> Result<()> {
    ensure!(age >= 0, "Age must be non-negative, got {age}");
    ensure!(age <= 150, "Age seems unrealistic: {age}");
    Ok(())
}

fn may_fail(b: bool) -> Result<()> {
    if !b {
        bail!("Condition not met");
    }
    Ok(())
}
```

## Advanced Module Patterns

### Re-exporting for a Clean Public API

```rust
// lib.rs
mod error;
mod parser;
mod ast;

// Re-export for ergonomic public API
pub use error::{Error, Result};
pub use parser::parse;
pub use ast::{Expr, Statement};

// Keep internal implementation private
// pub mod internal;   // only if you want to expose internals
```

This gives users `use my_crate::{parse, Expr, Error}` instead of `use my_crate::parser::parse; use my_crate::ast::Expr; use my_crate::error::Error;`.

### Public API Stability

- Things not marked `pub` are private to the crate.
- `pub(crate)` lets the whole crate see it but not outside crates.
- Adding `pub` to an existing private item is not a breaking change (expansion).
- Removing `pub` or changing a function signature is breaking.
- Changing trait bounds or associated types can be breaking.

## Advanced Testing

### Benchmarks with Criterion

```rust
// Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "my_bench"
harness = false
```

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_sum(c: &mut Criterion) {
    c.bench_function("sum vec", |b| {
        b.iter(|| {
            (0..1000).sum::<i32>()
        })
    });
}

criterion_group!(benches, bench_sum);
criterion_main!(benches);
```

### Fuzzing with cargo-fuzz (libFuzzer)

```bash
cargo install cargo-fuzz
cargo fuzz init
cargo fuzz add my_target
cargo fuzz run my_target
```

### Proptest for property-based testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_reverse_twice(s in ".*") {
        let rev_once = s.chars().rev().collect::<String>();
        let rev_twice = rev_once.chars().rev().collect::<String>();
        prop_assert_eq!(s, rev_twice);
    }
}
```

## Verification Checklist

- [ ] Can write a generic function with trait bounds and `where` clause
- [ ] Can choose between generic bound, `impl Trait` arg, `impl Trait` return, and `dyn Trait`
- [ ] Can define a trait with associated types and explain when to use them vs generic parameters
- [ ] Can define a struct with a lifetime parameter and explain what it ties
- [ ] Can write a function that needs explicit lifetime annotations (multiple input refs, returning a ref)
- [ ] Can explain why `&mut T` is invariant in `T`
- [ ] Can define a function accepting `?Sized` types via `&T`
- [ ] Can coerce `&String` to `&str`, `&Vec<T>` to `&[T]` explicitly and explain when it happens implicitly
- [ ] Can write a custom `Iterator` implementation
- [ ] Can use `collect()` into `Result<Vec<_>, _>`, `Option<Vec<_>>`, `HashSet`, `String`
- [ ] Can define a custom error enum with `thiserror` and use `?` to propagate
- [ ] Can use `anyhow` for application-level error handling with `ensure!`, `bail!`, `Context`
- [ ] Can write a proptest and a criterion benchmark

## Common Pitfalls

1. **`impl Trait` in return position from different branches returning different types.** The return type is a single opaque type — all branches must return the same concrete type (or a type that coerces to it). Use `Box<dyn Trait>` for heterogeneous returns.

2. **Lifetime too short on a struct holding references.** If a struct's lifetime parameter is too short, you can't store it where a longer lifetime is expected. The lifetime must cover all use sites.

3. **Choosing generic parameters over associated types for "single type per impl" roles.** This leads to awkward use: `impl Iterator<Item = i32> for MyIter` is clean; `impl<T> Iterator<T> for MyIter` forces users to write turbofish or suffers inference failures. Use associated types when there's one natural type.

4. **Using `dyn Trait` when a generic bound would do and performance matters.** Dynamic dispatch adds a vtable indirection per call. For hot paths with few concrete types, generics win. For heterogeneous collections, trait objects are the tool.

5. **Forgetting `T: 'static` is not "is Send"** `T: 'static` means T has no non-static references — it is safe to move to another thread but that's not the right question. Use `Send` for "can move to another thread" and `Sync` for "can share references across threads."

6. **Trying to store references in structs without lifetime parameters.** The compiler will reject it. Add a lifetime parameter to the struct. If the design genuinely needs self-referential structs, look at `ouroboros` or `self_cell` crates, not manual unsafe.

7. **Over-constraining generic bounds.** Writing `T: Clone + Debug + Display + PartialEq + Eq + Hash` when `T: Display` is all you need makes the function unusable for types that only implement `Display`. Constrain only what you use.

8. **Using `return` where an expression suffices.** Rust functions return the last expression. `return x;` works but `x` is more idiomatic in simple cases. Use `return` for early exit.

9. **Panicking in library code.** Libraries should propagate errors; applications decide whether to panic. A library panicking across a boundary is a bug in the library.

10. **Writing `unwrap()` in library functions.** Even `unwrap_or_default` silently hides failures. Return `Result` and let the caller decide.
