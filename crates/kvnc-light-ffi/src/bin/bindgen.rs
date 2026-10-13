//! Generate Kotlin or Swift bindings from the built `kvnc_light_ffi` library.
//!
//! `cargo run -p kvnc-light-ffi --features bindgen --bin kvnc-light-bindgen -- generate --library target/debug/libkvnc_light_ffi.so --language kotlin --out-dir apps/android/app/src/main/java`

fn main() {
    uniffi::uniffi_bindgen_main();
}
