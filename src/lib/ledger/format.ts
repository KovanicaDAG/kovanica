import { ATOM, DECIMALS } from "./types";

export function fmtKvnc(atoms: number | bigint): string {
  const v = Number(atoms) / ATOM;
  const s = v.toFixed(DECIMALS).replace(/\.?0+$/, "");
  return `${s || "0"} KVNC`;
}

export function parseKvnc(raw: string): number | null {
  const n = Number(String(raw).trim().replace(",", "."));
  if (!Number.isFinite(n) || n <= 0) return null;
  return Math.round(n * ATOM);
}
