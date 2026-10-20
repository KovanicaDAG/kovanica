/* tslint:disable */
/* eslint-disable */

/**
 * Derive `kvnc…dag` from the frozen SLIP-0010 path `m/44'/3007'/0'/0'/index'`.
 */
export function address_from_mnemonic(phrase: string, index: number): string;

/**
 * `kvnc…dag` P2PK address for a raw 32-byte public key (64 hex chars).
 */
export function address_from_public_key(pubkey_hex: string): string;

/**
 * Canonical 66-hex rendering of any accepted address form: `kvnc…dag`,
 * 64-hex legacy, or 66-hex versioned.
 */
export function address_to_hex(address: string): string;

/**
 * WASM entry: same as [`build_signed_transfer_inner`], with `JsValue` errors
 * for browser callers.
 */
export function build_signed_transfer(phrase: string, index: number, network: string, utxos_json: string, outputs_json: string, fee_atoms: string, change_address: string): string;

/**
 * Deterministic English mnemonic from caller-supplied entropy (16 or 32 bytes).
 */
export function entropy_to_mnemonic(entropy: Uint8Array): string;

/**
 * Generate a 12- or 24-word phrase.
 *
 * `words` must be 12 or 24.
 */
export function generate_mnemonic(words: number): string;

/**
 * BIP-39 validity: 12 or 24 English words with a correct checksum.
 */
export function mnemonic_is_valid(phrase: string): boolean;

/**
 * 64-byte BIP-39 seed (PBKDF2-HMAC-SHA512, 2048 iterations). Optional
 * passphrase acts as the BIP-39 "25th word".
 */
export function mnemonic_to_seed(phrase: string, passphrase: string): Uint8Array;

/**
 * Words in `phrase` that are missing from the BIP-39 English wordlist, in
 * order. Lets a caller show a precise "typo" hint before reporting a checksum
 * failure.
 */
export function mnemonic_unknown_words(phrase: string): string[];

/**
 * Normalize a recovery phrase: lowercase, collapse every run of whitespace
 * (spaces, newlines, tabs) to a single space, trim.
 */
export function normalize_mnemonic(phrase: string): string;

/**
 * Public key (32 bytes, 64 hex chars) at account `index`.
 */
export function public_key_from_mnemonic(phrase: string, passphrase: string, index: number): string;

/**
 * Public key (32 bytes, 64 hex chars) for a raw 32-byte Ed25519 secret
 * (64 hex chars). Wallets that already hold key material use this instead of
 * re-running the full derivation.
 */
export function public_key_from_secret_bytes(secret_hex: string): string;

/**
 * 32-byte Ed25519 signing seed at account `index` on the frozen path
 * `m/44'/3007'/0'/0'/index'` (all segments hardened).
 */
export function seed_from_mnemonic(phrase: string, passphrase: string, index: number): Uint8Array;

/**
 * Sign a sighash (hex) with the key at account `index`; returns the 64-byte
 * Ed25519 signature as 128 hex chars.
 */
export function sign_sighash(phrase: string, passphrase: string, index: number, sighash_hex: string): string;

/**
 * Sign a sighash (hex) with a raw 32-byte Ed25519 seed (64 hex chars).
 */
export function sign_sighash_with_seed_hex(seed_hex: string, sighash_hex: string): string;

/**
 * Signing key (32 bytes, 64 hex chars) at account `index`, derived from a
 * 64-byte BIP-39 seed (128 hex chars) with the frozen hardened path. Lets a
 * wallet that already holds seed material keep it and still derive per-index
 * keys through the core instead of re-implementing SLIP-0010.
 */
export function signing_key_from_seed_hex(bip39_seed_hex: string, index: number): string;

/**
 * Frozen SLIP-0010 derivation path template (`m/44'/3007'/0'/0'/i'`).
 */
export function slip10_derivation_path(): string;

/**
 * SLIP-44-style coin type for Kovanica (3007).
 */
export function slip44_coin_type(): number;

/**
 * Initialize panic hook for better console errors (optional feature).
 */
export function start(): void;

/**
 * Strict Ed25519 verification with the node's rules (malleable signatures are
 * rejected). Returns `false` for a well-formed but wrong signature; malformed
 * input throws.
 */
export function verify_signature(pubkey_hex: string, message_hex: string, signature_hex: string): boolean;

/**
 * SDK version.
 */
export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly address_from_mnemonic: (a: number, b: number, c: number) => [number, number, number, number];
    readonly address_from_public_key: (a: number, b: number) => [number, number, number, number];
    readonly address_to_hex: (a: number, b: number) => [number, number, number, number];
    readonly build_signed_transfer: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number) => [number, number, number, number];
    readonly entropy_to_mnemonic: (a: number, b: number) => [number, number, number, number];
    readonly generate_mnemonic: (a: number) => [number, number, number, number];
    readonly mnemonic_is_valid: (a: number, b: number) => number;
    readonly mnemonic_to_seed: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly mnemonic_unknown_words: (a: number, b: number) => [number, number];
    readonly normalize_mnemonic: (a: number, b: number) => [number, number];
    readonly public_key_from_mnemonic: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly public_key_from_secret_bytes: (a: number, b: number) => [number, number, number, number];
    readonly seed_from_mnemonic: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly sign_sighash: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly sign_sighash_with_seed_hex: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly signing_key_from_seed_hex: (a: number, b: number, c: number) => [number, number, number, number];
    readonly slip10_derivation_path: () => [number, number];
    readonly slip44_coin_type: () => number;
    readonly start: () => void;
    readonly verify_signature: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly version: () => [number, number];
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __externref_drop_slice: (a: number, b: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
