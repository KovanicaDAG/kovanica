import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dispatchApi } from "../src/lib/api/dispatch.server";
import { HttpError, withSource } from "../src/lib/api/client";

test("dispatchApi preserves network isolation: mainnet requests return 503 launching soon, never testnet data", async () => {
  const reqHead = new Request("http://localhost/api/head?source=mainnet");
  const resHead = await dispatchApi(reqHead);
  assert.equal(resHead.status, 503);
  assert.equal(await resHead.text(), "mainnet launching soon");

  const reqState = new Request("http://localhost/api/state?source=mainnet");
  const resState = await dispatchApi(reqState);
  assert.equal(resState.status, 503);
  assert.equal(await resState.text(), "mainnet launching soon");

  const reqBoot = new Request("http://localhost/api/bootstrap?source=mainnet");
  const resBoot = await dispatchApi(reqBoot);
  assert.equal(resBoot.status, 503);
  assert.equal(await resBoot.text(), "mainnet launching soon");

  const reqSubmit = new Request("http://localhost/api/submit?source=mainnet", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ tx: "test" }),
  });
  const resSubmit = await dispatchApi(reqSubmit);
  assert.equal(resSubmit.status, 403);
});

test("HttpError captures HTTP status and message accurately", () => {
  const err404 = new HttpError(404, "not found");
  assert.equal(err404.status, 404);
  assert.equal(err404.message, "not found");
  assert.ok(err404 instanceof Error);

  const err502 = new HttpError(502, "bad gateway");
  assert.equal(err502.status, 502);
  assert.equal(err502.message, "bad gateway");
});

test("withSource resolves absolute URL with host during SSR (typeof window === 'undefined')", () => {
  const url = withSource("/api/head", "testnet");
  assert.ok(url.startsWith("http://127.0.0.1:"), `Expected absolute loopback URL during SSR, got ${url}`);
  assert.ok(url.includes("/api/head"), `Expected path /api/head, got ${url}`);
  assert.ok(url.includes("source=testnet"), `Expected source=testnet query param, got ${url}`);
});

test("route files handle missing entities with notFound and provide notFoundComponent & errorComponent", async () => {
  const rwaCode = await readFile(
    new URL("../src/routes/explorer/rwa.$assetId.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(rwaCode.includes("notFoundComponent"), "RWA detail route must have notFoundComponent");
  assert.ok(rwaCode.includes("errorComponent"), "RWA detail route must have errorComponent");
  assert.ok(rwaCode.includes("throw notFound()"), "RWA loader must throw notFound() on 404");
  assert.ok(!rwaCode.includes('const source = "local";'), "RWA loader must not hardcode local source");

  const collectionCode = await readFile(
    new URL("../src/routes/wallet/collection.$collectionId.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(collectionCode.includes("notFoundComponent"), "Collection detail route must have notFoundComponent");
  assert.ok(collectionCode.includes("errorComponent"), "Collection detail route must have errorComponent");
  assert.ok(collectionCode.includes("throw notFound()"), "Collection loader must throw notFound() on 404");
  assert.ok(!collectionCode.includes('const source = "local";'), "Collection loader must not hardcode local source");

  const nftCode = await readFile(
    new URL("../src/routes/wallet/nft.$assetId.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(nftCode.includes("notFoundComponent"), "NFT detail route must have notFoundComponent");
  assert.ok(nftCode.includes("errorComponent"), "NFT detail route must have errorComponent");
  assert.ok(nftCode.includes("throw notFound()"), "NFT loader must throw notFound() on 404");
  assert.ok(!nftCode.includes('const source = "local";'), "NFT loader must not hardcode local source");
});

test("docs view contains no unlabelled hardcoded fallback or validator details", async () => {
  const docsCode = await readFile(
    new URL("../src/components/docs/docs-view.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(!docsCode.includes("4650"), "Docs view must not contain hardcoded 4650 blocks fallback");
  assert.ok(!docsCode.includes("(Val 2)"), "Docs view must not contain old '(Val 2)' validator detail");
  assert.ok(!docsCode.includes('boot?.network ?? "Chain 4"'), "Docs view must not hardcode Chain 4 fallback");
  assert.ok(docsCode.includes("useApiSource"), "Docs view must observe active network source");
});

test("explorer and network views handle mainnet pre-launch honestly without endless loading", async () => {
  const explorerCode = await readFile(
    new URL("../src/components/explorer/explorer-view.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(explorerCode.includes('source === "mainnet"'), "ExplorerView must handle mainnet pre-launch");
  assert.ok(explorerCode.includes("Mainnet Explorer Unavailable"), "ExplorerView must inform user that mainnet node is not open");

  const networkCode = await readFile(
    new URL("../src/components/network/network-view.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(networkCode.includes('source === "mainnet"'), "NetworkView must handle mainnet pre-launch");
  assert.ok(networkCode.includes("Mainnet Pre-Launch"), "NetworkView must inform user of pre-launch status");
});
