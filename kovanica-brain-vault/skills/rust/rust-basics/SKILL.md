---
name: rust-basics
description: Use when learning Rust from scratch — ownership, borrowing, cargo, std lib, structs, enums, pattern matching, and the compiler's error messages. Covers the 80/20 of day-one Rust.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, beginner, ownership, cargo, std]
    related_skills: [rust-advanced, rust-error-handling, rust-testing]
---

# Rust Basics

## Overview

Rust is a systems language with a strong static type system, no GC, and an ownership/borrowing model that the compiler enforces at compile time. The learning curve is front-loaded: the borrow checker fights you for the first few weeks, then becomes a productivity multiplier.

This skill covers the minimum viable Rust: what you need to read/write non-trivial code without constantly fighting the compiler.

## When to Use

- Starting Rust for the first time
- Coming from Python/JS/Go and hitting the borrow checker
- Need a refresher on ownership, lifetimes, or trait basics
- Setting up a new Rust project

**Don't use for:** async, macros, no_std/embedded, performance tuning, or advanced trait patterns — those have their own skills.

## Core Concepts

### 1. Project Structure

```
 Cargo.toml          # manifest: deps, features, profile
 src/
   main.rs           # binary entry point (cargo run)
   lib.rs            # library root (cargo build / used by others)
   modules/...       # mod.rs or foo.rs via `mod foo;`
```

```bash
cargo new my_project           # binary
cargo new --lib my_lib         # library
cargo add serde serde_json     # add deps (resolves to latest semver-compatible)
cargo fmt                       # format (rustfmt)
cargo clippy -- -D warnings    # lint (clippy, deny all warnings)
cargo doc --open                # generate + browse docs
```

### 2. Ownership & Borrowing

The three states of a value:

- **Owned** — you are the sole owner; `drop` runs when it goes out of scope.
- **Borrowed (&T)** — shared, read-only reference. Multiple allowed. Must not outlive the owner.
- **Mutably borrowed (&mut T)** — exclusive access. No other borrows (shared or mutable) coexist.

```rust
let s = String::from("hello");   // owned
let r = &s;                      // shared borrow
let mut v = vec![1, 2, 3];
let rv = &mut v;                 // mutable borrow — exclusive
// let _r2 = &v;                 // ERROR — already mutably borrowed
```

The compiler enforces **NLL (Non-Lexical Lifetimes)** since Rust 2021: borrows end after their last use, not at the scope boundary. This makes code like this fine:

```rust
let mut v = vec![1];
let r = &v;
println!("{r}");   // last use of r
v.push(2);         // OK — r is no longer live
```

### 3. Variables, Mutability, and Shadowing

```rust
let x = 5;           // immutable by default
let mut y = 5;       // mutable
y = 6;
let x = x + 1;       // shadowing — new binding, same name, different type OK
let x = x.to_string();
```

### 4. Types

- **Scalar:** `i32`, `u64`, `f64`, `bool`, `char`
- **Compound:** tuples `(i32, bool)`, arrays `[i32; 5]`
- **String-ish:** `String` (owned, heap), `&str` (borrowed slice of UTF-8), `Cow<str>` (either/or)
- **Collection:** `Vec<T>`, `HashMap<K,V>`, `BTreeMap<K,V>` (ordered)

Integer literals infer from context; use suffixes (`5u32`, `3.0f64`) for clarity.

### 5. Functions and Control Flow

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b   // no `return` needed for final expression
}

// early return for conditionals
fn parse_or_default(s: &str, def: i32) -> i32 {
    if let Ok(n) = s.parse() {
        return n;
    }
    def
}

// loop expressions can return values
let n = loop {
    if condition() { break 42; }
};
```

`if`/`if let`/`match` are expressions (produce values). `for item in iter` is the idiomatic loop.

### 6. Structs

```rust
// named fields
struct User {
    name: String,
    active: bool,
}

// tuple struct (newtype pattern)
struct UserId(u64);

// unit struct (marker / zero-sized type)
struct Marker;

// construction
let u = User { name: "Ana".into(), active: true };

// update syntax (like spread)
let u2 = User { active: false, ..u };

// derive common traits
#[derive(Debug, Clone, PartialEq)]
struct Point { x: f64, y: f64 }
```

`Debug` is the most useful derive for learning — `{:?}` in `println!`.

### 7. Enums and Pattern Matching

Enums in Rust can carry data — this is how `Option` and `Result` are built:

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

fn handle(m: Message) {
    match m {
        Message::Quit => println!("bye"),
        Message::Move { x, y } => println!("move to {x},{y}"),
        Message::Write(text) => println!("write: {text}"),
        Message::ChangeColor(r, g, b) => println!("color {r} {g} {b}"),
    }
}
```

`match` must be exhaustive — the compiler will warn on missed variants. Use `_ => ()` as a catch-all when appropriate.

### 8. Option and Result — the bread and butter

```rust
// Option<T> — value may be absent
fn find(haystack: &str, needle: &str) -> Option<usize> {
    haystack.find(needle)   // std lib already returns Option
}

// handle with if let, match, or combinator
let pos = find("hello", "ll").unwrap_or(0);

// Result<T, E> — operation may fail
fn read_number() -> Result<i32, std::num::ParseIntError> {
    "42".parse()
}

// propagate errors with ? (in functions returning Result or Option)
use std::fs;
fn read_config(path: &str) -> Result<String, std::io::Error> {
    let contents = fs::read_to_string(path)?;  // early return Err on failure
    Ok(contents)
}
```

**Rule of thumb:** prefer `Result` for fallible operations, `Option` for "search returned nothing", never `unwrap()` in production code (use `expect("reason")` in tests and prototypes, `.unwrap_or_default()`, or proper error handling).

### 9. Collections

```rust
let mut v: Vec<i32> = Vec::new();
v.push(1);
v.push(2);
let third = v[2];            // indexing — panics on OOB
let opt = v.get(2);         // safe — returns Option<&T>
let last = v.pop();          // Option<T>
let len = v.len();

let mut map = HashMap::new();
map.insert("key".into(), 42);
if let Some(v) = map.get("key") { /* ... */ }
```

Prefer `vec![]` and `HashMap::from([...])` for literals.

### 10. Traits

Traits are Rust's version of interfaces + typeclasses. They define behavior, and you can implement them for your types or use derive macros for common ones.

```rust
// define a trait
trait Summary {
    fn summarize(&self) -> String;
}

// implement for a type
impl Summary for User {
    fn summarize(&self) -> String {
        format!("User: {}", self.name)
    }
}

// use as a bound
fn notify(item: &impl Summary) {
    println!("{}", item.summarize());
}

// generic form (same thing, more explicit)
fn notify<T: Summary>(item: &T) { /* ... */ }

// trait objects (dynamic dispatch)
fn notify_any(item: &dyn Summary) { /* ... */ }
```

Most common derives: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`. `Copy` requires all fields to be `Copy`.

### 11. Lifetimes (the scary part — simplified)

Lifetimes are the compiler's way of ensuring references don't outlive their data. The compiler infers most lifetimes; you write them when the compiler needs help disambiguating.

```rust
// The compiler needs to know which input reference the output borrows from
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

Elision rules (compiler applies automatically when you don't write them):
1. Each input reference gets its own lifetime.
2. If there's exactly one input lifetime, it applies to all output lifetimes.
3. If there are multiple input lifetimes but one `&self` or `&mut self`, `self`'s lifetime applies to all outputs.

Write explicit lifetimes when:
- A function returns a reference and there are multiple input references.
- Structs hold references (every reference field needs a lifetime parameter).

```rust
struct Parser<'a> {
    input: &'a str,
}
```

### 12. Iterators

Rust iterators are lazy, zero-cost abstractions:

```rust
let v = vec![1, 2, 3, 4, 5];
let sum: i32 = v.iter()
    .filter(|x| *x % 2 == 0)
    .map(|x| x * 10)
    .sum();

// collect into a Vec
let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();

// enumerate, zip, flat_map, partition, find, any, all
let indexed: Vec<(usize, &i32)> = v.iter().enumerate().collect();
```

The `Iterator` trait has a huge set of methods — learn `map`, `filter`, `enumerate`, `zip`, `collect`, `find`, `any`, `all`, `fold`, `sum`, `product`, `take`, `skip`, `chain`, `flat_map` as the starter pack.

### 13. Modules and Visibility

```rust
// In lib.rs or a module file
mod networking {           // private by default
    fn internal_helper() { /* ... */ }

    pub fn connect() {     // public to crate
        internal_helper();
    }
}

// bring names into scope
use crate::networking::connect;
use std::collections::HashMap;

// re-export (publicly expose a submodule's item)
pub use networking::connect;
```

Visibility modifiers: `pub` (public), `pub(crate)` (visible to the whole crate), `pub(super)` (parent module), `pub(in path::to::module)` (specific module).

### 14. Cargo and Dependencies

```toml
[package]
name = "my-crate"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.11", features = ["json"] }

[dev-dependencies]
criterion = "0.5"
mockall = "0.12"

[features]
default = []
extra = ["dep:some_dep"]
```

- `cargo add <crate>` — add to `[dependencies]`
- `cargo add --dev <crate>` — add to `[dev-dependencies]`
- `cargo add --build <crate>` — add to `[build-dependencies]`
- `cargo update` — update lockfile to latest compatible
- `cargo update -p <crate> --precise <ver>` — pin a specific version
- `[patch.crates-io]` — override a dep with a local path or fork

### 15. Error Messages Are Helpful

The Rust compiler's error messages include:
- What went wrong (plain language)
- Why (the reasoning)
- How to fix it (suggested code)

**Read them.** When stuck with a borrow error, read the full message — it usually points exactly at the conflicting borrow. Use `cargo clippy` for idioms the compiler can't enforce.

### 16. Edition 2021 Defaults

Rust 2021 (the current default edition) brings:
- `IntoIterator` for `&mut Vec` and `&mut String` (no more `for x in &mut v` ambiguity)
- `Rust` 2021 namespace changes ( prelude changes: `From<&str> for String`, `TryFrom`/`TryInto` in prelude)
- Disjoint closures (a closure capturing a field doesn't borrow the whole struct)
- `#[safe_vocab]` changes for `extern "C"` (not relevant for most)

## Quick Reference

| Concept | Syntax | Notes |
|---|---|---|
| Mutable binding | `let mut x = ...` | default is immutable |
| Owned String | `String::from("...")` or `.to_string()` | heap-allocated, growable |
| Borrow | `&x`, `&mut x` | shared vs exclusive |
| Move | `let b = a;` (for non-Copy types) | ownership transfers |
| Clone | `a.clone()` | explicit deep copy |
| Result propagation | `expr?` | in `fn -> Result<_, _>` |
| Pattern match | `match x { A => ..., B => ... }` | exhaustive required |
| Option unwrap safe | `.unwrap_or(def)`, `.unwrap_or_default()` | prefer over `.unwrap()` |
| Iterator collect | `.iter().map(...).collect::<Vec<_>>()` | type inference usually suffices |
| Trait bound | `fn foo<T: Trait>(x: T)` | or `impl Trait` in arg position |
| Lifetime annotation | `<'a>`, `&'a str` | needed when returning refs from multiple inputs |

## Common Pitfalls

1. **Trying to share ownership with `Rc` too early.** Most problems are better solved with clear ownership + borrowing before reaching for `Rc`/`Arc`. Use smart pointers deliberately, not as a borrow-checker escape hatch.

2. **Fighting the borrow checker by cloning everything.** Cloning is a valid solution but be intentional about it — profile or reason about whether it's a bottleneck. Read the error first; the solution is often restructuring lifetimes, not cloning.

3. **Using `&String` instead of `&str`.** Functions should almost always take `&str`, not `&String`. A `&String` can be passed where `&str` is expected, but not vice versa. Same for `&Vec<T>` vs `&[T]`.

4. **Indexing collections with `[]`.** `v[i]` panics on out-of-bounds. Use `v.get(i)` when the index might be invalid and handle `None`.

5. **`unwrap()` in production code.** Fine in tests and prototypes with a clear invariant. In production code, use `expect("reason")`, `unwrap_or_default()`, or proper error propagation. The borrow checker kills panics harder at runtime.

6. **Forgetting that `String` is not `Copy`.** Assign-by-value moves ownership. If you need the original after a move, clone or borrow it first.

7. **Mixing `String` and `&str` in function signatures inconsistently.** Pick one convention and stick with it. The idiomatic rule: input parameters are `&str`, owned return values are `String`.

8. **Deriving `Copy` on types that contain `String` or `Vec`.** `Copy` is only possible when all fields are `Copy`. `Clone` is the right derive for most owned-data structs.

9. **Using `unwrap()` on `Result` from I/O, parsing, or networking in library code.** Library code should propagate errors. Application code can decide to panic or exit.

10. **Not reading compiler suggestions.** Rust compiler suggestions are usually exactly right. When it suggests a lifetime, try it before reasoning from scratch.

## Verification Checklist

- [ ] Can create a new cargo project (`cargo new`)
- [ ] Can add dependencies (`cargo add`)
- [ ] Can write a function with owned and borrowed parameters
- [ ] Can handle `Option` and `Result` without `unwrap()` in non-test code
- [ ] Can define a struct, derive `Debug`/`Clone`/`PartialEq`, and create instances
- [ ] Can define an enum with data variants and match on it exhaustively
- [ ] Can write a simple iterator chain (`iter().filter().map().collect()`)
- [ ] Can read and act on a borrow-checker error message
- [ ] Can use `?` for error propagation in a `Result`-returning function
- [ ] Knows when to use `&str` vs `String`, `&[T]` vs `&Vec<T>`
- [ ] Can run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`
- [ ] Can read `cargo doc --open` output for their own types

## One-Shot Recipes

### New project + format + lint + test

```bash
cargo new my_project --name my_project
cd my_project
cargo add serde serde_json   # example deps
cargo fmt
cargo clippy -- -D warnings
cargo test                    # runs doctests + tests
cargo doc --open              # browse docs
```

### Read a file into a String (Result-propagating)

```rust
use std::fs;
fn read_file(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}
```

### Parse a CLI argument as a number, fallback on failure

```rust
fn parse_port(s: &str) -> u16 {
    s.parse().unwrap_or_else(|e| {
        eprintln!("Invalid port '{s}': {e}");
        std::process::exit(1);
    })
}
```

### HashMap with owned keys

```rust
use std::collections::HashMap;
let mut scores = HashMap::new();
scores.insert("Alice".to_string(), 10);
let name = "Bob".to_string();
scores.insert(name, 5);  // name moved into map — can't use after
```

### Struct with a reference (needs lifetime)

```rust
struct Highlight<'a> {
    text: &'a str,
    color: u32,
}
```
