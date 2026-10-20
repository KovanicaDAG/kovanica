import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const text = await readFile(new URL("../.env.example", import.meta.url), "utf8");
const entries = text
  .split("\n")
  .map((l) => l.trim())
  .filter((l) => l && !l.startsWith("#"))
  .map((l) => {
    const i = l.indexOf("=");
    return [l.slice(0, i), l.slice(i + 1)] as const;
  });

test(".env.example contains no non-loopback IPv4 literals", () => {
  for (const [key, value] of entries) {
    for (const ip of value.match(/\b\d{1,3}(?:\.\d{1,3}){3}\b/g) ?? []) {
      assert.ok(ip.startsWith("127."), `${key} contains a non-loopback IP`);
    }
  }
});

test(".env.example leaves every secret blank", () => {
  for (const [key, value] of entries) {
    if (/SECRET|TOKEN|PASSWORD|DATABASE_URL|PRIVATE/.test(key)) {
      assert.equal(value, "", `${key} must be blank in .env.example`);
    }
  }
});

test(".env.example documents the KOVANICA_* config", () => {
  const keys = new Set(entries.map(([k]) => k));
  for (const k of [
    "KOVANICA_PUBLIC_SEED_PEERS",
    "KOVANICA_PUBLIC_P2P_PORT",
    "KOVANICA_PUBLIC_POOL_STRATUM_URL",
    "KOVANICA_NODE_API_URL",
  ]) {
    assert.ok(keys.has(k), `${k} missing`);
  }
});
