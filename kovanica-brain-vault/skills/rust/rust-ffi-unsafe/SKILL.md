---
name: rust-ffi-unsafe
description: Use when bridging Rust to C/C++/other languages, writing extern "C" functions, using raw pointers, building #[repr(C)] structs, or using unsafe Rust deliberately with clear unsafe invariants documented per-call.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, FFI, extern-C, unsafe, repr-C, raw-pointers, callbacks]
    related_skills: [rust-basics, rust-advanced, rust-crypto]
---

# Rust FFI and Unsafe

## Overview

**FFI (Foreign Function Interface)** lets Rust call code in other languages (usually C) and let other languages call Rust code. **`unsafe`** is Rust's escape hatch for operations the compiler cannot verify — raw pointer dereference, calling unsafe functions, implementing unsafe traits, accessing static mut, and more. Both must be used deliberately with documented invariants: every `unsafe` block should state what safety condition it assumes and why it holds.

This skill covers writing FFI boundaries and using `unsafe` correctly. "Correct" here means: invariants are explicit, unsafe surface area is minimized, and the boundary is the only place where assumptions are made.

## When to Use

- Calling C libraries (libxml2, OpenSSL, zlib, system libraries)
- Exposing Rust code to C/C++ consumers
- Embedding Rust in a C/C++ application
- Writing callbacks that C code can invoke
- Using raw pointers for zero-copy interoperability
- Manually implementing a trait that requires `unsafe` (e.g. `Send`, `Sync`, `GlobalAlloc`)
- Performance-critical code where the compiler won't auto-vectorize but you've verified correctness

**Don't use for:** normal Rust code paths. If you can express something safely, do. `unsafe` is for the boundary and the rare case where safe Rust cannot express what you need.

## `unsafe` Fundamentals

### What `unsafe` Grants

An `unsafe` block allows five operations:

1. **Dereferencing raw pointers** (`*const T`, `*mut T`)
2. **Calling unsafe functions** (functions marked `unsafe fn` or extern functions)
3. **Accessing or modifying `static mut`**
4. **Implementing unsafe traits** (`Send`, `Sync`, `GlobalAlloc`, etc.)
5. **Accessing fields of `union`** (union field access is unsafe)

`unsafe` does **not** disable borrow checking, does **not** disable bounds checking on safe operations, and does **not** disable any other safety check. It only enables the five operations above.

```rust
unsafe {
    let ptr: *const i32 = &10;
    println!("{}", *ptr);   // dereferencing raw pointer — requires unsafe
}
```

### Unsafe Invariants

Every `unsafe` block must have a documented safety comment that states **what invariant is being assumed** and **why it's guaranteed at this call site**. This makes unsafe auditable:

```rust
/// # Safety
/// `ptr` must be non-null and point to a valid, initialized `T` that is
/// accessible for reads for the duration of the call. Caller must ensure
/// no other code mutates `*ptr` during the call (if T is not Sync).
unsafe fn read_at(ptr: *const i32) -> i32 {
    // SAFETY: caller guarantees ptr is non-null, valid, initialized
    unsafe { *ptr }
}
```

For functions, use a `# Safety` section in the doc comment. For blocks, use a `// SAFETY:` comment inline.

### Minimizing Unsafe Surface

The goal is to have the smallest possible `unsafe` boundary, with a safe API wrapping it. Pattern:

```rust
// unsafe implementation hidden — safe wrapper exposes it
struct Buffer {
    ptr: *mut u8,
    len: usize,
    // invariant: ptr is valid for len bytes, allocated by our alloc
}

impl Buffer {
    /// # Safety
    /// ptr must be non-null, len must be the allocation size, and the
    /// caller must ensure the underlying allocation outlives this Buffer.
    unsafe fn from_raw_parts(ptr: *mut u8, len: usize) -> Self {
        // SAFETY: caller guarantees the invariant (documented above)
        unsafe { Buffer { ptr, len } }
    }

    /// Safe constructor — allocates and sets up invariants
    fn new(len: usize) -> Result<Self, Error> {
        if len == 0 { return Ok(Buffer { ptr: std::ptr::null_mut(), len: 0 }); }
        let layout = std::alloc::Layout::from_size_align(len, 1)
            .map_err(|e| Error::BadLayout(e))?;
        let ptr = unsafe { std::alloc::alloc(layout) };
        if ptr.is_null() {
            return Err(Error::AllocFailed);
        }
        Ok(Buffer { ptr, len })
    }

    /// Safe accessor
    fn as_slice(&self) -> &[u8] {
        // SAFETY: self.ptr is valid for self.len bytes, &self borrows immutably
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if !self.ptr.is_null() && self.len > 0 {
            let layout = std::alloc::Layout::from_size_align(self.len, 1).unwrap();
            unsafe { std::alloc::dealloc(self.ptr, layout) };
        }
    }
}
```

The safe API enforces the invariant; the unsafe code is only reached through the safe API (or through documented `unsafe fn` entry points that the caller is responsible for).

## `extern "C"`

### Calling C Functions

```rust
extern "C" {
    // Declare a C function you will call
    fn abs(input: i32) -> i32;

    // Function with a variadic argument list — painful, avoid if possible
    fn printf(format: *const i8, ...) -> i32;
}

fn main() {
    unsafe {
        println!("abs(-5) = {}", abs(-5));
    }
}
```

`extern "C"` blocks declare foreign functions. The ABI string selects the calling convention:
- `"C"` — standard C ABI
- `"C-unwind"` — like C but unwinds the stack on panic (use when Rust panics across the FFI boundary — UB if not caught)
- `"stdcall"`, `"win64"`, `"system"`, etc. — platform-specific ABIs

**Rule:** always wrap calls to `extern "C"` functions in `unsafe` blocks.

### Exporting Rust Functions to C

```rust
// Rust function callable from C
#[no_mangle]
pub extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}

// With a static library, linkers see `rust_add` as a C symbol
```

`#[no_mangle]` prevents Rust from renaming the symbol. `pub extern "C"` makes it visible and uses the C calling convention.

Consuming from C:

```c
// consumer.c
#include <stdio.h>

extern int rust_add(int a, int b);

int main() {
    printf("3 + 4 = %d\n", rust_add(3, 4));
    return 0;
}
```

```bash
# Build
cargo build --release
# Link against the staticlib or cdylib
```

### C-ABI Compatible Types

Only types with a defined C layout can cross the FFI boundary safely:

- Primitive integers: `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `isize`, `usize`
- `f32`, `f64`
- `bool` (C `_Bool` compatible in practice but technically depends on C compiler; often safer to pass `u8` and convert)
- `*const T`, `*mut T` (raw pointers)
- `extern "C" fn(...) -> ...` (function pointers)
- `#[repr(C)]` structs

Types that are **not** C-ABI compatible:
- Rust `String`, `Vec`, `Box`, enums (unless `repr(C)`), tuples (unless `repr(C)`), closures

### `#[repr(C)]` Structs

```rust
#[repr(C)]
struct Point {
    x: f64,
    y: f64,
}

#[repr(C)]
struct Rect {
    top_left: Point,
    bottom_right: Point,
}
```

`#[repr(C)]` defines a struct's memory layout to match C's rules: field order is preserved, padding is inserted as a C compiler would, and the struct has a C-compatible size and alignment.

**Caveat:** C and Rust have different enum representations. `#[repr(C)]` on an enum makes it a tagged union in C terms — but the tag layout iscompiler-defined. Prefer using `repr(u8)` or `repr(i32)` for enums you want to match C exactly.

## Raw Pointers

### Creating and Using Raw Pointers

```rust
let x = 10;
let ptr: *const i32 = &x;           // derive from reference (safe)
let mut y = 20;
let mptr: *mut i32 = &mut y;        // derive from mutable reference

// Creating from address (unsafe — no validation)
let addr = 0x1000 as *const u8;     // DANGEROUS — no guarantee the address is valid
unsafe {
    // reading from arbitrary address is UB if address not valid
    let _val = *addr;
}
```

### Dereferencing Raw Pointers

```rust
let x = 10;
let ptr = &x as *const i32;

unsafe {
    let val = *ptr;   // dereference — requires unsafe, UB if ptr invalid
    println!("{val}");
}
```

### Pointer Arithmetic

```rust
let arr = [1i32, 2, 3, 4, 5];
let base = arr.as_ptr();

unsafe {
    let ptr = base.offset(2);   // pointer to arr[2]
    let val = *ptr;             // 3
    // or use .add(n), .sub(n) on *const/mut T
    let ptr2 = base.wrapping_add(2);
}
```

**Important:** offset arithmetic is only valid within the same allocated object. Going past the end of an allocation is UB even if you don't read. `wrapping_offset` is less strict (still UB to read past allocation, but the offset operation itself doesn't UB).

### `NonNull` — a Safer Raw Pointer

`NonNull<T>` is a wrapper around a raw pointer that guarantees non-null. It's used in the standard library for types like `Box`, `Vec`, `Rc` internals. It gives you the nullity guarantee without full validity guarantees:

```rust
use std::ptr::NonNull;

let x = 10;
let nnptr = NonNull::new(&x as *const i32).unwrap();   // None if null

unsafe {
    let ptr = nnptr.as_ptr();   // get raw pointer back
    let val = *ptr;
}
```

`NonNull` is covariant over `T` (like `*const T`) — be aware when building data structures.

### When to Use Raw Pointers

- Interfacing with C code that uses pointers
- Building low-level data structures (linked lists, trees) where ownership is manual
- Zero-copy views into data (slices from raw pointers)
- FFI callbacks

**When NOT to use:** if you can express ownership with `Box`, `Vec`, `Arc`, `Rc`, `Cow`, or references, do that. Raw pointers are for when safe Rust's ownership model doesn't fit the problem.

## FFI Patterns

### Passing Slices to C

```rust
// Rust → C: pass a slice as a pointer + length
fn pass_slice_to_c(data: &[u8]) {
    unsafe {
        c_function(data.as_ptr(), data.len() as i32);
    }
}

// C → Rust: receive a pointer + length and make a slice
unsafe fn slice_from_c(ptr: *const u8, len: usize) -> &[u8] {
    // SAFETY: caller guarantees ptr is valid for len bytes
    std::slice::from_raw_parts(ptr, len)
}
```

### Passing Strings

C strings are null-terminated. Rust `&str` / `String` are not (they're UTF-8 with a length, not a terminator).

```rust
// Rust string → C (null-terminated)
fn pass_string_to_c(s: &str) {
    let c_string = std::ffi::CString::new(s)
        .expect("string contained interior null");
    unsafe {
        c_function_string(c_string.as_ptr());
    }
    // CString dropped here — C code must not use the pointer after this call
    // unless it copies the data
}

// Caller-owned C string → Rust
unsafe fn rust_string_from_c(ptr: *const i8) -> Result<String, Error> {
    if ptr.is_null() {
        return Err(Error::NullPtr);
    }
    let c_str = std::ffi::CStr::from_ptr(ptr);
    let rustr = c_str.to_str()?;   // validates UTF-8
    Ok(rust_str.to_string())
}
```

**Caveat:** `CString::new` fails if the Rust string contains an interior null byte. C functions that take `char*` and `strlen` rely on the terminator — Rust strings may contain null bytes (not in UTF-8 text, but in arbitrary data).

### Callbacks (Passing Rust Functions to C)

```rust
// Define a Rust function that matches C's expected callback signature
extern "C" fn rust_callback(data: *mut c_void, value: i32) {
    let info = unsafe { &*(data as *const CallbackInfo) };
    info.on_value(value);
}

#[repr(C)]
struct CallbackInfo {
    on_value: extern "C" fn(i32),
}

// Register callback with C library
fn register_callback(info: &CallbackInfo) {
    unsafe {
        c_register_callback(
            info as *const CallbackInfo as *mut c_void,
            Some(rust_callback),   // function pointer, not closure
        );
    }
}
```

**Important:** C callbacks cannot capture Rust state via closures. You pass a function pointer (`extern "C" fn`) and a separate `userdata`/`context` pointer that the callback receives. You must ensure the `userdata` outlives the callback registration, or the callback will access freed memory.

### Lifetime of FFI Data

When Rust allocates data and passes a pointer to C, C may use it after the Rust call returns. Ensure:
- The Rust object lives as long as C needs it (leak intentionally, use `Box::leak`, or transfer ownership with clear convention)
- Or, copy the data into C-owned memory before the Rust call returns

```rust
// Give C ownership of a Rust-allocated string
fn pass_owned_string(s: String) -> *mut i8 {
    let c_string = std::ffi::CString::new(s).unwrap();
    c_string.into_raw()   // prevents drop, returns raw pointer — C owns it now
}
// C caller must call `free` (or the matching Rust deallocator) when done
```

## `#[repr(C)]` and Layout Control

### Layout Guarantees

`#[repr(C)]` gives you:
- Field order preserved as declared
- Size = sum of field sizes + padding (as C compiler would compute)
- Alignment = max alignment of any field

```rust
#[repr(C)]
struct Foo {
    a: u8,     // offset 0, size 1
              // padding: 3 bytes (align i32 to 4)
    b: i32,    // offset 4, size 4
    c: u16,    // offset 8, size 2
              // padding: 2 bytes (struct align to 4)
};              // total size: 12
```

Use `std::mem::size_of::<T>()` and `std::mem::align_of::<T>()` to verify layout.

### `#[repr(transparent)]`

When a struct has exactly one field and you want it to have the same layout as that field (including padding/alignment):

```rust
#[repr(transparent)]
struct MyWrapper(u32);
// MyWrapper has same layout as u32 — safe to transmute or FFI
```

This is useful for newtype wrappers that need to match C types exactly.

### `transmute` — Dangerous, Use Carefully

```rust
let x: u32 = 0x00000001;
let y: [u8; 4] = unsafe { std::mem::transmute(x) };  // [1, 0, 0, 0] on little-endian
```

`transmute` reinterprets the bits of a value as another type. It's `unsafe` because the compiler doesn't check that the source and destination types have the same size. Use only when:
- Types have identical size and alignment
- The bit reinterpretation is intentional and documented
- Endianness is understood

Prefer `bytemuck` crate for safer casting between types with the same representation.

## `unsafe` Traits

### `Send` and `Sync`

```rust
// T is Send if it's safe to transfer ownership to another thread
// T is Sync if &T is Send — safe to share a reference across threads

// Most types are Send + Sync automatically.
// You only need to manually implement when:
// - Your type contains a raw pointer or FFI handle
// - Your type has interior mutability via Cell/RefCell (not Sync)
// - Your type uses unsafe interior mutability correctly (Mutex is Sync)

// Example: a type wrapping a raw pointer is NOT Send/Sync automatically
struct RawSocket(*mut c_void);

// Manually asserting Send if the raw handle is thread-safe
unsafe impl Send for RawSocket {}
unsafe impl Sync for RawSocket {}
```

**Warning:** implementing `Send`/`Sync` for a type with raw pointers is a soundness claim. You must ensure the raw pointer is actually safe to share/move across threads. Get this wrong → data race or use-after-free.

### `GlobalAlloc`

```rust
// Custom global allocator (rarely needed — use a crate like `tikv-jemallocator`)
struct MyAlloc;

unsafe impl GlobalAlloc for MyAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        /* ... */
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        /* ... */
    }
}

#[global_allocator]
static ALLOC: MyAlloc = MyAlloc;
```

## C Inline / `cxx` / `bindgen`

### `bindgen` — Auto-generate Rust bindings from C headers

```bash
cargo add bindgen --dev   # build dependency
```

```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=c_header.h");
    cbindgen::Generate::new()
        .header("c_header.h")
        .generate()
        .unwrap()
        .write_to_file("src/bindings.rs");
}
```

Then include in your code:

```rust
include!("bindings.rs");   // includes the generated bindings

fn use_c_lib() {
    unsafe {
        c_function_from_header(42);
    }
}
```

### `cxx` — Safe Rust/C++ interop

`cxx` crate provides a safer bridge for Rust ↔ C++ interop with automatic generation of bindings and safety guarantees. Use it when interoperating with C++ rather than manually writing extern blocks.

## Verification Checklist

- [ ] Can declare an `extern "C"` block and call a C function in an `unsafe` block
- [ ] Can export a `#[no_mangle] pub extern "C"` Rust function for C to call
- [ ] Can create a `#[repr(C)]` struct with known layout and verify with `size_of`/`align_of`
- [ ] Can pass a Rust `&[T]` to C as `(ptr, len)` and reconstruct on the C side
- [ ] Can pass a Rust string to C as null-terminated `CString` and understand the lifetime
- [ ] Can write a C callback function (`extern "C" fn`) that receives a `userdata` pointer
- [ ] Can create and dereference a `*const T` / `*mut T` raw pointer in `unsafe`
- [ ] Can use `NonNull` to represent a non-null raw pointer
- [ ] Can use `slice::from_raw_parts` and `CStr::from_ptr` with documented safety preconditions
- [ ] Can document `# Safety` for each `unsafe fn` and `// SAFETY:` for each `unsafe` block
- [ ] Knows the five things `unsafe` enables and why each is unsafe
- [ ] Minimizes unsafe surface: safe wrapper around unsafe implementation
- [ ] Can explain why `Send`/`Sync` must be manually implemented for types with raw pointers

## Common Pitfalls

1. **Calling C functions without checking null pointers.** C functions often return null on failure. Always check before dereferencing.

2. **Letting C code use Rust-owned data after Rust frees it.** If Rust creates a `CString` and passes `.as_ptr()` to C, the `CString` is dropped at the end of the Rust scope. C must not hold the pointer past that. Use `into_raw()` to transfer ownership, or copy.

3. **Using `extern "C" fn` closures.** Closures that capture environment cannot be passed as `extern "C"` function pointers. Only `fn` items (no captures) can be used as C function pointers. Use a `userdata` pointer to pass context.

4. **`bool` in FFI.** C `_Bool` and Rust `bool` are not guaranteed to be ABI-compatible by the Rust reference. In practice they work on most targets, but for maximum portability pass `u8` and convert.

5. **Null pointers from `NonNull::new` returning `None`.** `NonNull` can only represent non-null pointers — `NonNull::new(ptr)` returns `None` if `ptr` is null. Don't unwrap without checking.

6. **Union field access without `unsafe`.** Union field reads/writes are `unsafe` in Rust because only one field is active at a time and Rust can't verify which.

7. **Not considering unwinding across FFI boundary.** If Rust code panics while C code called into it (or C code called Rust which then panics), unwinding across the FFI boundary is UB. Use `catch_unwind` or `extern "C-unwind"` ( Rust 1.82+ stable).

8. **Assuming C `int` is 32 bits everywhere.** On some platforms C `int` is 16 bits (embedded). Use `i32`/`u32` in Rust for known-width and match with `int32_t`/`uint32_t` on the C side.

9. **Forgetting alignment when passing structs.** `#[repr(C)]` handles layout, but if you pass a pointer to a struct allocated in Rust to C, the allocator must produce correctly aligned memory. `std::alloc::alloc` with the correct `Layout` does this.

10. **Using `transmute` without verifying size and alignment match.** `transmute` compiles fine even when types are different sizes — it's UB if sizes differ. Use `bytemuck` or `size_of` checks.

## Quick Reference

| Task | Rust idiom |
|---|---|
| Call C function | `extern "C" { fn foo(...); }` then `unsafe { foo(...) }` |
| Expose Rust to C | `#[no_mangle] pub extern "C" fn ...` |
| Match C struct layout | `#[repr(C)] struct { ... }` |
| Pass slice | `(data.as_ptr(), data.len())` |
| Receive slice from C | `unsafe { slice::from_raw_parts(ptr, len) }` |
| Pass String to C | `CString::new(s).unwrap(); c_fn(c_str.as_ptr())` |
| Own C string in Rust | `CStr::from_ptr(ptr).to_str()?` then `.to_string()` |
| Non-null raw pointer | `NonNull::<T>::new(ptr)` |
| Pointer arithmetic | `.offset(n)`, `.wrapping_offset(n)`, `.add(n)`, `.sub(n)` |
| Callbacks | `extern "C" fn callback(userdata: *mut c_void, ...) { ... }` |
| Custom allocator | `unsafe impl GlobalAlloc for T { ... }` |
| Thread-safe raw wrapper | `unsafe impl Send for T {}` + `unsafe impl Sync for T {}` (with justification) |
