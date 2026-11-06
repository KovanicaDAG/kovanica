/**
 * Client helpers for sending KVP-102 tokens.
 *
 * A token balance lives in the token contract, not in a UTXO, so a transfer is
 * a `Call` transaction to `token_transfer` — not the native `/api/prepare`
 * flow. The server builds the signing hash (it owns the nonce read), the
 * browser signs it with the wallet's Ed25519 key, and the server submits the
 * bincode-encoded transaction.
 */

/** Flat fee the node charges for a token call, in KVNC atoms. Mirrors the server. */
export const TOKEN_SEND_FEE = 10_000;

/** `0x`-prefixed contract address → the 64-hex form the API expects. */
export function stripTokenPrefix(address: string): string {
  return address.trim().replace(/^0x/i, "");
}

function query(
  from: string,
  token: string,
  to: string,
  amount: bigint,
): string {
  return (
    `from=${encodeURIComponent(from)}` +
    `&token=${encodeURIComponent(stripTokenPrefix(token))}` +
    `&to=${encodeURIComponent(to)}` +
    `&amount=${amount.toString()}`
  );
}

/** `/api/token_prepare` — returns `{ sighash }` for the wallet to sign. */
export function tokenPrepareUrl(
  from: string,
  token: string,
  to: string,
  amount: bigint,
): string {
  return `/api/token_prepare?${query(from, token, to, amount)}`;
}

/** `/api/token_submit` — submits the signed call and returns `{ tx }`. */
export function tokenSubmitUrl(
  from: string,
  token: string,
  to: string,
  amount: bigint,
  sig: string,
): string {
  return `/api/token_submit?${query(from, token, to, amount)}&sig=${encodeURIComponent(sig)}`;
}
