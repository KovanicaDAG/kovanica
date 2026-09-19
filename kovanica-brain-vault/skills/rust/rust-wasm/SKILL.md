---
name: rust-wasm
description: Use when compiling Rust to WebAssembly: wasm-bindgen for JS interop, wasm-pack for building and packaging, web-sys for browser APIs, js-sys for JS types, serde-wasm-bindgen for serialization, and common Wasm patterns.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, wasm, webassembly, wasm-bindgen, wasm-pack, web-sys, js-sys, serde-wasm-bindgen, browser]
    related_skills: [rust-basics, rust-async, rust-serialization]
---

# Rust WebAssembly

## Overview

Rust compiles to WebAssembly (Wasm) via `wasm32-unknown-unknown` target. The primary tooling is **wasm-bindgen** (Rust ↔ JS interop), **wasm-pack** (build + package + publish workflow), **web-sys** (browser API bindings), and **js-sys** (JS type bindings).

Wasm from Rust is used for:
- Computation-heavy tasks that benefit from Rust's performance in the browser
- Reusing Rust code in web applications
- CLI tools that run in the browser
- Shared logic between backend and frontend

This skill covers the common workflow and patterns. For heavy JS interop or DOM manipulation, the mental model is: Rust owns computation, JS owns the DOM.

## When to Use

- Porting a Rust library to run in the browser
- Writing a compute-heavy function (crypto, parsing, image processing) in Rust for the web
- Building a Wasm module to be consumed by JS (or another Wasm module)
- Sharing logic between Rust backend and web frontend

**Don't use for:** UI rendering (use JS/TypeScript frameworks for DOM), or when the computation is so light that the Wasm overhead outweighs the benefit.

## wasm-pack — The Build Tool

```bash
cargo install wasm-pack
```

`wasm-pack` builds Rust to Wasm, generates JS bindings via wasm-bindgen, and packages everything for consumption. It can target Node.js or the browser.

```bash
# Build for browser (default)
wasm-pack build --target web   # ES modules, for use in browser via <script type="module">
wasm-pack build --target bundler   # for use with webpack, Rollup, etc.
wasm-pack build --target nodejs    # for Node.js consumption

# Build with release optimization
wasm-pack build --release

# Output goes to pkg/ directory (by default)
```

**Targets:**
- `web` — raw ES module, load directly in browser with `<script type="module">`
- `bundler` — designed for webpack/Rollup consumption (may include extra glue)
- `nodejs` — for Node.js, uses `require()` / `import`

### Package Workflow

```bash
wasm-pack build --release --target web
# → pkg/ directory with .wasm, .js glue, package.json, README

# Publish to npm
wasm-pack publish

# Or just point npm at the pkg/ directory
# npm install ./pkg
```

## wasm-bindgen — Rust ↔ JS Interop

```toml
[dependencies]
wasm-bindgen = "0.2"

[lib]
crate-type = ["cdylib"]
```

### Exporting Rust Functions to JS

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[wasm_bindgen]
pub struct Point {
    x: f64,
    y: f64,
}

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    pub fn distance_from_origin(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

#[wasm_bindgen]
pub fn distance(p1: &Point, p2: &Point) -> f64 {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    (dx * dx + dy * dy).sqrt()
}
```

JS usage (after `wasm-pack build`):

```javascript
import init, { add, greet, Point, distance } from "./pkg/my_wasm.js";

async function main() {
    await init();   // initializes the Wasm module

    console.log(add(2, 3));          // 5
    console.log(greet("Alice"));     // "Hello, Alice!"

    const p1 = new Point(3, 4);
    console.log(p1.distance_from_origin());   // 5

    const p2 = new Point(0, 0);
    console.log(distance(p1, p2));   // 5
}

main();
```

### Importing JS Functions into Rust

```rust
use wasm_bindgen::prelude::*;

// Import JS function
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = fetch)]
    async fn fetch(url: &str) -> JsValue;
}

// Use imported function
pub fn rust_side() {
    log("Hello from Rust!");
}
```

### JS Types in Rust (js-sys)

```toml
[dependencies]
js-sys = "0.3"
wasm-bindgen = "0.2"
```

`js-sys` provides bindings for JS built-in types: `Date`, `RegExp`, `Array`, `Map`, `Set`, `Promise`, etc.

```rust
use wasm_bindgen::prelude::*;
use js_sys::{Date, Array, Uint8Array, Promise, JSON};

#[wasm_bindgen]
pub async fn get_json(url: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let resp_value = window.fetch_with_str(url).await?;
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();
    let json_value = resp.json().await?;
    Ok(json_value)
}
```

### Web APIs (web-sys)

```toml
[dependencies]
web-sys = { version = "0.3", features = [
    "console",
    "Window",
    "Document",
    "Element",
    "FetchObserver",
    "Request",
    "Response",
]}
```

`web-sys` provides bindings to browser APIs. Features must be enabled per API.

```rust
use wasm_bindgen::prelude::*;
use web_sys::{window, console};

#[wasm_bindgen]
pub fn log_to_console(msg: &str) {
    // Access the browser console
    if let Some(window) = window() {
        if let Some(console) = window.console() {
            console.log_1(&msg.into());
        }
    }
}

#[wasm_bindgen]
pub fn get_document_title() -> String {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|doc| doc.title())
        .unwrap_or_default()
}

#[wasm_bindgen]
pub fn set_body_background(color: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(body) = document.body() {
                body.style().set_property("background-color", color).unwrap();
            }
        }
    }
}
```

### Working with the DOM

```rust
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement};

#[wasm_bindgen]
pub fn add_click_handler(element_id: &str, callback: js_sys::Function) {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.get_element_by_id(element_id).unwrap();
    element.add_event_listener_with_callback("click", &callback).unwrap();
}
```

JS side:

```javascript
import init, { add_click_handler } from "./pkg/my_wasm.js";

async function main() {
    await init();

    const handler = (event) => {
        console.log("Clicked!", event);
    };

    add_click_handler("my-button", handler);
}

main();
```

### Closures (JS callbacks to Rust)

```rust
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use web_sys::{Window, Event, EventTarget};

#[wasm_bindgen]
pub fn register_callback() {
    let window = web_sys::window().unwrap();

    let closure = Closure::wrap(Box::new(|event: web_sys::Event| {
        web_sys::console::log_1(&"Event received!".into());
    }) as Box<dyn FnMut(web_sys::Event)>);

    window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref()).unwrap();

    // Keep the closure alive
    closure.forget();
}
```

**Important:** `Closure::forget()` leaks the closure (prevents Rust from dropping it while JS might call it). For one-shot callbacks, use `closure.into_js_value()` or `Closure::once`.

### Promises and Async

```rust
use wasm_bindgen::prelude::*;
use js_sys::Promise;
use web_sys::Window;

#[wasm_bindgen]
pub async fn fetch_data(url: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let promise = window.fetch_with_str(url)?;
    let resp_value = promise.await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;
    let json_promise = resp.json()?;
    let json_value = json_promise.await?;
    Ok(json_value)
}
```

## serde-wasm-bindgen — Serialization to JS

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde-wasm-bindgen = "0.6"
```

`serialize` Rust types to JS values (and vice versa) without JSON round-tripping.

```rust
use serde::{Serialize, Deserialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[wasm_bindgen]
pub fn get_user_js() -> js_sys::Object {
    let user = User {
        id: 1,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    };

    to_value(&user).unwrap()   // returns js_sys::Value (Object)
}
```

JS side:

```javascript
import init, { get_user_js } from "./pkg/my_wasm.js";

const user = get_user_js();
console.log(user.name);   // "Alice"
```

**Tradeoff:** `serde-wasm-bindgen` is faster and more direct than JSON.stringify/parse, but the JS value is less inspectable than a plain object. For debugging, JSON is easier.

## Building and Testing

### Testing Wasm

```bash
cargo test --target wasm32-unknown-unknown
```

You need the wasm target installed:

```bash
rustup target add wasm32-unknown-unknown
```

For browser-based testing, use a framework like `wasm-bindgen-test` or run tests in Node.js with `wasm-pack test --node`.

### Size Optimization

Wasm binary size matters for web loading. Strategies:

```toml
# Cargo.toml
[profile.release]
opt-level = "z"   # optimize for size
lto = true
codegen-units = 1
panic = "abort"   # no unwinding, smaller binary
```

```bash
wasm-pack build --release
# Further optimize with wasm-opt (from Binaryen)
wasm-opt -Oz pkg/my_wasm_bg.wasm -o pkg/my_wasm_bg.wasm
```

`wasm-opt` is part of the Binaryen toolchain.

### wasm-bindgen Crates

| Crate | Purpose |
|---|---|
| `wasm-bindgen` | Rust ↔ JS interop |
| `wasm-bindgen-futures` | Convert JS Promises to Rust futures |
| `js-sys` | JS built-in types (Date, Array, Map, etc.) |
| `web-sys` | Browser API bindings (DOM, Fetch, WebGL, etc.) |
| `serde-wasm-bindgen` | serde to/from JS values |

## Common Patterns

### Initializing the Wasm Module

Every wasm-bindgen module needs an `init()` async function to load the Wasm binary. In JS:

```javascript
import init, { /* exports */ } from "./pkg/my_wasm.js";

async function main() {
    await init();   // loads the .wasm file
    // now use the exports
}

main();
```

For inline Wasm (no network request for the .wasm):

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    // This runs automatically when the module loads (no await needed)
    web_sys::console::log_1(&"Module loaded!".into());
}
```

`#[wasm_bindgen(start)]` functions run on module load, before any JS calls the exports.

### Exposing a Rust Error to JS

```rust
use wasm_bindgen::prelude::*;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Not found")]
    NotFound,
    #[error("Invalid input: {0}")]
    Invalid(String),
}

#[wasm_bindgen]
pub fn risky_operation(input: &str) -> Result<String, JsValue> {
    if input.is_empty() {
        return Err(Error::Invalid("empty".to_string()).into());
    }
    Ok(format!("Processed: {}", input))
}
```

Convert Rust errors to `JsValue` (which JS sees as an exception). JS can catch it with try/catch.

### Using Rust Data Structures in JS

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Counter {
    count: u32,
}

#[wasm_bindgen]
impl Counter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Counter {
        Counter { count: 0 }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn reset(&mut self) {
        self.count = 0;
    }
}
```

JS:

```javascript
import init, { Counter } from "./pkg/my_wasm.js";

const counter = new Counter();
counter.increment();
console.log(counter.get_count());   // 1
```

### Sharing State Between Rust and JS

Keep state in Rust and expose methods to read/modify it. JS doesn't directly access Rust struct fields — it goes through the wasm-bindgen-generated methods.

```rust
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SharedState {
    value: Mutex<i32>,
}

#[wasm_bindgen]
impl SharedState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SharedState {
        SharedState { value: Mutex::new(0) }
    }

    pub fn get(&self) -> i32 {
        *self.value.lock().unwrap()
    }

    pub fn set(&self, value: i32) {
        *self.value.lock().unwrap() = value;
    }
}
```

For `no_std` Wasm (embedded or minimal), note that `std` is not available — use `spinlock` or `atomic` based alternatives.

## wasm-pack Output Structure

After `wasm-pack build --target web`:

```
pkg/
  my_wasm.wasm          # the Wasm binary
  my_wasm.js            # JS glue (wasm-bindgen generated)
  my_wasm.d.ts          # TypeScript types
  package.json          # for npm
  README.md
  etc.
```

The JS glue handles:
- Loading the Wasm binary
- Converting Rust types to JS types and vice versa
- Exporting Rust functions/structs as JS

## Verification Checklist

- [ ] Can install wasm-pack and add `wasm32-unknown-unknown` target
- [ ] Can set up a library crate with `crate-type = ["cdylib"]` and `wasm-bindgen` dependency
- [ ] Can export a Rust function and struct to JS with `#[wasm_bindgen]`
- [ ] Can import a JS function into Rust via `extern "C"` block
- [ ] Can use `web-sys` to access browser APIs (console, window, document)
- [ ] Can use `js-sys` for JS types (Date, Array, Promise)
- [ ] Can build with `wasm-pack build --target web/bundler/nodejs`
- [ ] Understands the `init()` async function requirement in JS
- [ ] Can use `serde-wasm-bindgen` to serialize Rust structs to JS objects
- [ ] Can register a JS callback in Rust with `Closure`
- [ ] Can optimize Wasm size with release profile + `wasm-opt`
- [ ] Understands that Wasm carries the `init()` overhead and that cold start matters for small modules

## Common Pitfalls

1. **Forgetting `await init()` in JS.** The Wasm module isn't usable until `init()` completes. Calling exports before `init()` returns undefined or throws.

2. **Using `std::fs` / `std::net` in Wasm.** Wasm in the browser can't access the filesystem or network directly — it goes through browser APIs (web-sys). For Node.js, some std features work but not all.

3. **Not enabling web-sys features.** `web-sys` uses feature flags — you must enable each API you use (e.g., `features = ["Window", "Document"]`). Forgetting a feature leads to compile errors about missing bindings.

4. **Using panic! in Wasm.** A panic in Wasm terminates the module (or the thread, depending on the environment). In the browser, this can crash the page. Handle errors gracefully and convert to `Result` or `JsValue`.

5. **Forgetting to `forget()` closures that JS may call later.** If a closure is dropped while JS still holds a reference, calling it is UB. Use `Closure::forget()` for long-lived callbacks, or `Closure::once` for one-shot.

6. **Assuming Wasm is fast for everything.** Wasm has cold-start overhead (loading + instantiation). For small, fast operations, the overhead may dominate. Benchmark.

7. **Not testing in the target environment.** Wasm that works in Node.js may not work in the browser (and vice versa). Test in the actual target.

8. **Using `String` / `&str` across the boundary without understanding the cost.** Each crossing of the Rust/JS boundary has a cost (serialization, memory allocation). For high-frequency calls, batch operations or share memory.

9. **Ignoring the Wasm binary size.** Large Wasm binaries slow page loads. Use `wasm-opt` and check the size.

10. **Building with wrong target.** `wasm-pack build --target web` generates ES module output. If you need CommonJS for Node.js, use `--target nodejs`. Mismatched target leads to import errors.
