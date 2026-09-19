---
name: rust-macros
description: Use when writing Rust macros: declarative macros (macro_rules!), procedural macros (derive, attribute, function), syn/quote for parsing, proc-macro2, and common macro patterns and pitfalls.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, macros, macro_rules, proc-macros, derive, attribute, syn, quote, proc-macro2]
    related_skills: [rust-basics, rust-advanced]
---

# Rust Macros

## Overview

Rust has two macro systems:
- **Declarative macros** (`macro_rules!`) — pattern-based, expanded at the syntactic level, good for repetitive code generation and DSL-like syntax.
- **Procedural macros** (`proc_macro`) — Rust functions that take token streams and return token streams, compiled as crates, more powerful but more complex.

Use macros when you need code generation that can't be expressed with generics/traits/functions, or when you're building a domain-specific syntax. Avoid macros when a function, trait, or generics would do — macros are harder to understand, harder to refactor, and don't produce great IDE support.

This skill covers the common macro patterns. For the full macro story, consult the Rust Reference and the syn/quote documentation.

## When to Use

- Generating repetitive boilerplate (e.g., `Serialize`/`Deserialize` on many types — but prefer serde's derive over writing your own)
- Defining domain-specific syntax (e.g., a test harness macro, a route definition macro)
- Creating a derive macro for a custom trait (e.g., `#[derive(MyTrait)]`)
- Attribute macros for custom annotations (e.g., `#[my_attr]`)
- Function-like macros for custom syntax (e.g., `sql!(...)`, `vec![]` — though `vec!` is a built-in macro)

**Don't use for:** simple code reuse that functions or traits can handle. Macros obscure type errors, make refactoring harder, and produce worse error messages.

## Declarative Macros (`macro_rules!`)

### Basic Pattern

```rust
macro_rules! say_hello {
    () => {
        println!("Hello!");
    };
    ($name:expr) => {
        println!("Hello, {}!", $name);
    };
}

say_hello!();              // "Hello!"
say_hello!("Alice");       // "Hello, Alice!"
```

Macro patterns match token trees. `$name:expr` matches an expression and binds it to `$name`.

### Repetition

```rust
// `vec!` is a real macro — conceptually:
macro_rules! my_vec {
    () => {
        std::vec::Vec::new()
    };
    ($($x:expr),* $(,)?) => {
        {
            let mut v = std::vec::Vec::new();
            $(
                v.push($x);
            )*
            v
        }
    };
}

let v = my_vec![1, 2, 3];   // [1, 2, 3]
let empty: Vec<i32> = my_vec![];   // []
```

`$($x:expr),*` — match zero or more expressions separated by commas. `$(...)*` — repeat the inner pattern for each match.

**Repetition modes:**
- `$(...),*` — comma-separated (with optional trailing comma via `$(,)?`)
- `$(...),+` — one or more, comma-separated
- `$(...)` — no separator (e.g., `$( $x:expr )*` — space-separated or no separator)

### Multiple Patterns (Macro Rules as Function Overloading)

```rust
macro_rules! parse {
    ($s:expr) => { parse_with_radix($s, 10) };
    ($s:expr, $radix:expr) => { parse_with_radix($s, $radix) };
}

parse!("42");          // radix 10
parse!("42", 16);      // radix 16
```

### `tt` (Token Tree) Matching

```rust
macro_rules! chain {
    ($first:expr) => {
        $first
    };
    ($first:expr, $second:expr $(, $rest:expr)*) => {
        chain!($second $(, $rest)*)
    };
}
```

`tt` matches any token tree (a single token or a grouped set). Useful for building macros that consume arbitrary tokens.

### 메타변수 종류

| 메타변수 | 매칭 |
|---|---|
| `$x:item` | item (fn, struct, mod, etc.) |
| `$x:block` | block `{}` |
| `$x:stmt` | statement |
| `$x:expr` | expression |
| `$x:pat` | pattern |
| `$x:ty` | type |
| `$x:ident` | identifier |
| `$x:path` | path (e.g., `std::vec::Vec`) |
| `$x:tt` | token tree (any single token or grouped) |
| `$x:literal` | literal |

## Procedural Macros

Procedural macros are crates of type `proc-macro`. They receive a `TokenStream`, process it, and return a `TokenStream`.

```toml
[lib]
proc-macro = true

[dependencies]
syn = "2"
quote = "1"
proc-macro2 = "1"
```

### Derive Macro

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(MyTrait)]
pub fn my_trait_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = &input.ident;

    let expanded = quote! {
        impl MyTrait for #name {
            fn do_something(&self) {
                println!("Doing something for {}", stringify!(#name));
            }
        }
    };

    TokenStream::from(expanded)
}

// Usage:
// #[derive(MyTrait)]
// struct Foo;
```

**Derive macro pattern:**
1. Parse the input with `syn::parse_macro_input!`
2. Extract the struct/enum name and fields
3. Generate code with `quote!`
4. Return the generated `TokenStream`

### Attribute Macro

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, TokenStream};

#[proc_macro_attribute]
pub fn my_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // _attr — the arguments to the attribute (e.g., #[my_attr(x = 1)])
    // item — the item the attribute is attached to

    let item = parse_macro_input!(item as syn::Item);

    quote! {
        #item
        // additional code generated by the macro
    }.into()
}
```

Attribute macros can modify or wrap the annotated item. Common uses: injecting code, wrapping function bodies, generating additional items.

### Function-like Macro

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ExprArray};

#[proc_macro]
pub fn my_vec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ExprArray);

    // input is an expression array (like vec![1, 2, 3])
    // generate code based on the contents
    let elements = &input.elems;

    quote! {
        {
            let mut v = std::vec::Vec::new();
            #(#v.push(#elements);)*
            v
        }
    }.into()
}
```

### The `syn` Crate — Parsing Token Streams

`syn` provides types for parsing Rust syntax from token streams:

```rust
use syn::{DeriveInput, Fields, FieldsNamed, FieldsUnnamed};

// Parse a struct
fn parse_struct(input: DeriveInput) {
    match input.data {
        syn::Data::Struct(data) => {
            match data.fields {
                Fields::Named(FieldsNamed { fields, .. }) => {
                    for field in fields {
                        let name = &field.ident;
                        // ...
                    }
                }
                Fields::Unnamed(FieldsUnnamed { fields, .. }) => {
                    for field in fields {
                        // tuple struct field
                    }
                }
                Fields::Unit => {
                    // unit struct
                }
            }
        }
        syn::Data::Enum(data) => {
            // enum variants
        }
        syn::Data::Union(data) => {
            // union
        }
    }
}
```

### The `quote` Crate — Generating Code

`quote!` generates `TokenStream` from a DSL:

```rust
use quote::{quote, ToTokens};

let name = syn::Ident::new("Foo", Span::call_site());

// Simple interpolation
let tokens = quote! {
    struct #name {
        x: i32,
    }
};

// Repetition (like macro_rules)
let fields = vec![quote! { x: i32 }, quote! { y: i32 }];

let tokens = quote! {
    struct Point {
        #( #fields )*
    }
};
// expands to:
// struct Point {
//     x: i32
//     y: i32
// }

// Interpolation with `ToTokens` trait
let expr = quote! { 1 + 2 };
quote! {
    let x = #expr;
}
```

### `proc-macro2` — Token Stream Abstraction

`proc-macro2` is a compatible abstraction over `proc_macro::TokenStream`. It's used by `syn` and `quote`, and allows macros to be tested without a procedural macro context (since `proc_macro` only works in proc-macro crates).

```rust
use proc_macro2::TokenStream;

// In a proc-macro crate, proc_macro::TokenStream is available.
// In tests or non-proc-macro code, use proc_macro2's version.
```

## Common Macro Patterns

### Builder Pattern Macro

```rust
// Rather than writing a macro for builders, prefer the builder pattern with
// methods. Macros for builders tend to be more complex than they're worth.
```

### Test Harness Macro

```rust
#[macro_export]
macro_rules! test_case {
    ($name:ident, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let result = process($input);
            assert_eq!(result, $expected);
        }
    };
}

// Usage:
test_case!(parse_empty, "", Ok(Value::Empty));
test_case!(parse_number, "42", Ok(Value::Number(42)));
```

### SQL Query Macro (Conceptual)

```rust
#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    // Parse a SQL string literal, validate it, generate typed bindings
    // This is complex — use a crate like sqlx for compile-time checked SQL
}
```

In practice, don't roll your own SQL macro — use `sqlx::query!` which does this with compile-time verification against a live database.

## Macro Pitfalls

### Hygiene

Rust macros are hygienic: identifiers introduced by a macro don't capture or get captured by the surrounding scope. This prevents accidental name collisions but also means you can't accidentally reference a variable in the caller's scope.

```rust
macro_rules! make_fn {
    () => {
        fn foo() {
            let x = 5;
            println!("{}", x);
        }
    };
}

make_fn!();
// `x` inside foo is local to foo — the macro doesn't capture any `x` from outside
```

If you need to reference something from the caller's scope, you must pass it as a macro argument (declarative) or explicitly generate code that references the caller's names (procedural — but hygiene still applies).

### Debugging Macros

Macro errors are notoriously hard to debug. Use:

```bash
cargo expand   # shows the expanded code (cargo-expand tool)
```

```bash
cargo install cargo-expand
cargo expand my_module   # shows expanded code for my_module
```

For declarative macros, pay attention to the pattern matching — if no pattern matches, you get a "no rule expected" error.

For procedural macros, the error is often "cannot find type" or "unexpected token" after expansion. `cargo expand` shows what the macro generated.

### Error Messages from Procedural Macros

If a procedural macro generates invalid code, the compiler error points to the generated code, not the macro invocation. This is confusing. Use `compile_error!` or `syn::Error` to emit errors at the macro site:

```rust
use syn::Error;

fn validate(input: DeriveInput) -> Result<TokenStream, Error> {
    if input.data.r#struct.fields.len() > 10 {
        return Err(Error::new(input.ident.span(), "Too many fields"));
    }
    // ...
    Ok(TokenStream::from(expanded))
}
```

### Testing Procedural Macros

```rust
#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    // Test the macro logic without the proc_macro context
    fn expand_foo() -> TokenStream {
        quote! {
            impl MyTrait for Foo {
                fn do_something(&self) {}
            }
        }
    }

    #[test]
    fn test_expand() {
        let tokens = expand_foo();
        // compile the tokens and check?
        // For full testing, use trybuild (compiles code with the macro and checks error messages)
    }
}
```

For full integration testing of proc macros:

```toml
[dev-dependencies]
trybuild = "1"
```

```rust
#[test]
fn test_my_trait() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/my_trait_pass.rs");
    t.fail("tests/ui/my_trait_fail.rs");
}
```

trybuild compiles the test files and checks that the pass file compiles and the fail file produces the expected error.

## Verification Checklist

- [ ] Can write a declarative macro with multiple patterns and repetition
- [ ] Can write a derive procedural macro using syn + quote
- [ ] Can write an attribute procedural macro
- [ ] Can write a function-like procedural macro
- [ ] Can parse a struct with named fields using syn
- [ ] Can generate code with quote! and interpolation
- [ ] Can use `cargo expand` to debug macro expansion
- [ ] Can emit errors from a procedural macro with `syn::Error`
- [ ] Can test proc macros with trybuild for UI tests
- [ ] Understands macro hygiene basics

## Common Pitfalls

1. **Using macros when generics/traits would do.** A macro that generates a trait impl is harder to use than a trait bound. Prefer traits.

2. **Writing macros without understanding hygiene.** Macros can't see the caller's local variables. Pass them as arguments.

3. **Not testing macro expansion.** Macros generate code — test that the generated code compiles and behaves correctly.

4. **Generating code that doesn't compile.** A macro that generates invalid Rust is invisible until expansion. Use `cargo expand` and trybuild.

5. **Macro error messages are unhelpful.** If the macro fails to match, the error is "no rule expected the input." Make patterns specific and test with varied inputs.

6. **Using `tt` matching too greedily.** `tt` matches anything — a macro that consumes `tt` can accidentally consume tokens meant for another part of the macro. Be specific with fragment types.

7. **Not handling trailing commas.** Macro patterns for comma-separated lists should allow an optional trailing comma: `$(,)?`.

8. **Expanding to code that captures the wrong identifiers.** If a macro generates a variable `x`, and the caller also has an `x`, hygiene prevents collision — but if the macro explicitly references the caller's `x` via `$x`, it's captured correctly. Understand the difference.

9. **Re-inventing derive macros that serde already provides.** Before writing a derive macro, check if serde, derive_more, or another crate already does it.

10. **Writing proc macros without `proc-macro2` for testing.** `proc_macro` only works in proc-macro crates. Use `proc_macro2` to test macro logic in regular test code.
