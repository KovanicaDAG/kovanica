export const KVNC_DECIMALS = 9;
export const ATOMS_PER_KVNC = 1_000_000_000n;

/** Legacy aliases retained for backward compatibility. */
export const KUNA_DECIMALS = KVNC_DECIMALS;
export const ATOMA_PER_KUNA = ATOMS_PER_KVNC;

const DECIMALS = KVNC_DECIMALS;
const ATOM = ATOMS_PER_KVNC;

/** Parse a decimal KVNC string into base units (9 decimals). */
export function parseKvnc(input: string): bigint {
  const text = input.trim();
  if (!/^\d+(\.\d+)?$/.test(text)) {
    throw new Error("enter a KVNC amount");
  }
  const [whole, frac = ""] = text.split(".");
  if (frac.length > DECIMALS) {
    throw new Error("KVNC has 9 decimal places");
  }
  return BigInt(whole) * ATOM + BigInt(frac.padEnd(DECIMALS, "0"));
}

/** Format base units as a KVNC decimal, trimming trailing zeros. */
export function formatKvnc(atoms: bigint): string {
  if (atoms < 0n) {
    throw new Error("amount is negative");
  }
  const whole = atoms / ATOM;
  const frac = atoms % ATOM;
  if (frac === 0n) return whole.toString();
  const fracText = frac.toString().padStart(DECIMALS, "0").replace(/0+$/, "");
  return `${whole}.${fracText}`;
}

/** Legacy alias for {@link parseKvnc}. */
export const parseKuna = parseKvnc;

/** Legacy alias for {@link formatKvnc}. */
export const formatKuna = formatKvnc;
