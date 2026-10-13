import { blake3 } from "@noble/hashes/blake3";

import { bytesToHex, hexToBytes } from "./bytes.js";

const PREFIX = "kvnc";
const SUFFIX = "dag";

/**
 * Canonical address: `kvnc` + hex(32-byte public key || 4-byte BLAKE3 checksum) + `dag`.
 * The checksum is the first 4 bytes of unkeyed BLAKE3 over the public key.
 */
export function encodeAddress(publicKey: Uint8Array): string {
  if (publicKey.length !== 32) {
    throw new Error("address payload must be 32 bytes");
  }
  const checksum = blake3(publicKey).slice(0, 4);
  const body = new Uint8Array(36);
  body.set(publicKey, 0);
  body.set(checksum, 32);
  return `${PREFIX}${bytesToHex(body)}${SUFFIX}`;
}

/** Decode a canonical `kvnc…dag` address and verify its checksum. */
export function decodeAddress(value: string): Uint8Array {
  const trimmed = value.trim();
  if (!trimmed.startsWith(PREFIX) || !trimmed.endsWith(SUFFIX)) {
    throw new Error("address must start with kvnc and end with dag");
  }
  const body = hexToBytes(trimmed.slice(PREFIX.length, -SUFFIX.length));
  if (body.length !== 36) {
    throw new Error("address payload must be 32 bytes plus a 4-byte checksum");
  }
  const payload = body.slice(0, 32);
  const checksum = body.slice(32);
  const expected = blake3(payload).slice(0, 4);
  if (!equalBytes(checksum, expected)) {
    throw new Error("address checksum mismatch");
  }
  return payload;
}

/**
 * Accept a canonical address or a bare / `0x` 32-byte hex string.
 * Hex input skips the checksum, matching `Address::from_str` in kvnc-types.
 */
export function parseAddress(value: string): Uint8Array {
  const trimmed = value.trim();
  if (trimmed.startsWith(PREFIX) || trimmed.endsWith(SUFFIX)) {
    return decodeAddress(trimmed);
  }
  const raw = hexToBytes(trimmed);
  if (raw.length !== 32) {
    throw new Error("address must be 32 bytes");
  }
  return raw;
}

function equalBytes(left: Uint8Array, right: Uint8Array): boolean {
  if (left.length !== right.length) return false;
  let diff = 0;
  for (let i = 0; i < left.length; i += 1) {
    diff |= left[i] ^ right[i];
  }
  return diff === 0;
}
