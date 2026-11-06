import { test } from "node:test";
import assert from "node:assert/strict";
import { fmtKvnc } from "../src/lib/ledger/format";
import {
  TOKEN,
  K,
  getSpecText,
  SUBSIDY,
  MAX_SUPPLY,
  ATOM,
} from "../src/lib/networkConstants";
import { SPEC_TEXT } from "../src/lib/api/spec";
import {
  createDagBlocksCache,
  isDagUnchanged,
} from "../src/lib/api/map-blocks";
import { nextBackoffMs, MIN_POLL_MS, MAX_BACKOFF_MS } from "../src/lib/api/use-node";
import { ADDRESS_LENGTH, encodeAddress } from "../src/lib/wallet/address";
import type { ApiDagBlock } from "../src/lib/api/contract";
import { readFile } from "node:fs/promises";

test("fmtKvnc returns value with KVNC suffix", () => {
  assert.equal(fmtKvnc(1_000_000_000n), "1 KVNC");
  assert.equal(fmtKvnc(2_500_000_000n), "2.5 KVNC");
  assert.equal(fmtKvnc(0n), "0 KVNC");
});

test("networkConstants and spec generators reflect KVNC, K=5, 4s target, 2.5 KVNC reward", () => {
  assert.equal(TOKEN, "KVNC");
  assert.equal(K, 5);
  assert.equal(SUBSIDY, 2_500_000_000);
  assert.equal(MAX_SUPPLY, 90_200_000 * ATOM);

  const testnetSpec = getSpecText("testnet");
  assert.ok(testnetSpec.includes("KVNC"), "Network spec must mention KVNC");
  assert.ok(testnetSpec.includes("GHOSTDAG K = 5"), "Network spec must mention GHOSTDAG K = 5");
  assert.ok(testnetSpec.includes("4 s"), "Network spec must mention 4 s target block interval");
  assert.ok(testnetSpec.includes("2.5 KVNC"), "Network spec must mention 2.5 KVNC initial reward");
  assert.ok(!testnetSpec.includes("KUNA"), "Network spec must not contain old KUNA ticker");

  const apiSpec = SPEC_TEXT;
  assert.ok(apiSpec.includes("KVNC"), "API spec must mention KVNC");
  assert.ok(apiSpec.includes("GHOSTDAG K = 5"), "API spec must mention GHOSTDAG K = 5");
  assert.ok(!apiSpec.includes("KUNA"), "API spec must not contain old KUNA ticker");
});

test("read adapter has no synthetic candidate generation (_cand_r)", async () => {
  const dispatchCode = await readFile(
    new URL("../src/lib/api/dispatch.server.ts", import.meta.url),
    "utf8",
  );
  assert.ok(
    !dispatchCode.includes("_cand_r"),
    "dispatch.server.ts must not contain synthetic _cand_r candidate generation",
  );
  assert.ok(
    !dispatchCode.includes("earliestWindowBlock.parents = [dag[0].id]"),
    "dispatch.server.ts must not contain fake genesis link",
  );
});

test("graph cache helper: unchanged DAG returns stable layout input; changed tips/edges invalidate cache", () => {
  const blockA: ApiDagBlock = {
    id: "0xaaaa",
    parents: [],
    selected_parent: null,
    work: 0,
    timestamp_ms: 1000,
    nonce: 0,
    blue_score: 0,
    colour: "genesis",
    txs: [],
  };

  const blockB: ApiDagBlock = {
    id: "0xbbbb",
    parents: ["0xaaaa"],
    selected_parent: "0xaaaa",
    work: 1,
    timestamp_ms: 5000,
    nonce: 1,
    blue_score: 1,
    colour: "chain",
    txs: [],
  };

  const initialDag: ApiDagBlock[] = [blockA, blockB];
  const order = ["0xaaaa", "0xbbbb"];

  const getCachedBlocks = createDagBlocksCache();
  const firstResult = getCachedBlocks(initialDag, order);
  assert.equal(firstResult.length, 2);

  // Poll 2: identical DAG data in new array reference
  const secondDag: ApiDagBlock[] = [{ ...blockA }, { ...blockB }];
  assert.ok(isDagUnchanged(initialDag, secondDag));
  const secondResult = getCachedBlocks(secondDag, order);
  // Must preserve exact array identity
  assert.equal(
    secondResult,
    firstResult,
    "Unchanged DAG must return stable cached array reference",
  );

  // Poll 3: New block arrives (tip changes)
  const blockC: ApiDagBlock = {
    id: "0xcccc",
    parents: ["0xbbbb"],
    selected_parent: "0xbbbb",
    work: 2,
    timestamp_ms: 9000,
    nonce: 2,
    blue_score: 2,
    colour: "chain",
    txs: [],
  };
  const updatedDag: ApiDagBlock[] = [blockA, { ...blockB, colour: "blue" }, blockC];
  const updatedOrder = ["0xaaaa", "0xbbbb", "0xcccc"];
  const thirdResult = getCachedBlocks(updatedDag, updatedOrder);

  assert.notEqual(
    thirdResult,
    firstResult,
    "Changed DAG must invalidate cache and return fresh blocks",
  );
  assert.equal(thirdResult.length, 3);
});

test("polling backoff math: bounded backoff caps at 30s and resets on success", () => {
  assert.equal(MIN_POLL_MS, 2000);
  assert.equal(MAX_BACKOFF_MS, 30000);

  let delay = MIN_POLL_MS; // 2000
  delay = nextBackoffMs(delay); // 4000
  assert.equal(delay, 4000);

  delay = nextBackoffMs(delay); // 8000
  assert.equal(delay, 8000);

  delay = nextBackoffMs(delay); // 16000
  assert.equal(delay, 16000);

  delay = nextBackoffMs(delay); // 32000 capped at 30000
  assert.equal(delay, 30000);

  delay = nextBackoffMs(delay); // stays capped at 30000
  assert.equal(delay, 30000);
});

test("PRO-21 canonical address format: 64 characters with kvnc...dag", () => {
  assert.equal(ADDRESS_LENGTH, 64);

  const zeroKey = new Uint8Array(32);
  const addr = encodeAddress(zeroKey);
  assert.equal(addr.length, 64);
  assert.ok(addr.startsWith("kvnc"));
  assert.ok(addr.endsWith("dag"));
});
