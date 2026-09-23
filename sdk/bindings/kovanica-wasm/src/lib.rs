//! WASM bindings for kovanica-sdk.
//!
//! Build with:
//! ```text
//! wasm-pack build bindings/kovanica-wasm --target web
//! ```
//!
//! This is a minimal surface for the first browser integration.
//! Expand as the TypeScript package matures.

use kovanica_sdk::keys::to_kvnc;
use kovanica_sdk::prelude::*;
use wasm_bindgen::prelude::*;

/// Initialize panic hook for better console errors (optional feature).
#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Generate a 12- or 24-word mnemonic phrase.
///
/// `words` must be 12 or 24.
#[wasm_bindgen]
pub fn generate_mnemonic(words: u32) -> Result<String, JsValue> {
    let wc = match words {
        12 => WordCount::Words12,
        24 => WordCount::Words24,
        _ => return Err(JsValue::from_str("words must be 12 or 24")),
    };
    let m = Mnemonic::generate(wc).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(m.phrase())
}

/// Derive the `kvnc…dag` address from a mnemonic phrase at account index
/// `index` using the frozen SLIP-0010 path `m/44'/917'/0'/0'/index'`
/// (empty passphrase).
#[wasm_bindgen]
pub fn address_from_mnemonic(phrase: &str, index: u32) -> Result<String, JsValue> {
    let m = Mnemonic::from_phrase(phrase).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let kp = Keypair::from_mnemonic_at(&m, "", index);
    Ok(to_kvnc(&kp.address()))
}

/// SDK version.
#[wasm_bindgen]
pub fn version() -> String {
    kovanica_sdk::VERSION.to_string()
}
