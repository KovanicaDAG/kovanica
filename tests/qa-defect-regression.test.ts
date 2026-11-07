import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { ApiError } from "../src/lib/api/client";
import { dispatchApi } from "../src/lib/api/dispatch.server";

test("ApiError stores status code and extends Error", () => {
  const err404 = new ApiError("asset not found", 404);
  assert.equal(err404.name, "ApiError");
  assert.equal(err404.message, "asset not found");
  assert.equal(err404.status, 404);
  assert.ok(err404 instanceof Error);
  assert.ok(err404 instanceof ApiError);

  const err500 = new ApiError("internal server error", 500);
  assert.equal(err500.status, 500);
});

test("Route definitions for RWA, Collection, and NFT export notFoundComponent and errorComponent", async () => {
  const rwaCode = await readFile(
    new URL("../src/routes/explorer/rwa.$assetId.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(rwaCode.includes("notFoundComponent: RwaNotFoundPage"), "RWA route must define notFoundComponent");
  assert.ok(rwaCode.includes("errorComponent: RwaErrorPage"), "RWA route must define errorComponent");
  assert.ok(rwaCode.includes("throw notFound()"), "RWA loader must throw notFound() on missing asset");

  const collCode = await readFile(
    new URL("../src/routes/wallet/collection.$collectionId.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(collCode.includes("notFoundComponent: CollectionNotFoundPage"), "Collection route must define notFoundComponent");
  assert.ok(collCode.includes("errorComponent: CollectionErrorPage"), "Collection route must define errorComponent");
  assert.ok(collCode.includes("throw notFound()"), "Collection loader must throw notFound() on missing collection");

  const nftCode = await readFile(
    new URL("../src/routes/wallet/nft.$assetId.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(nftCode.includes("notFoundComponent: NftNotFoundPage"), "NFT route must define notFoundComponent");
  assert.ok(nftCode.includes("errorComponent: NftErrorPage"), "NFT route must define errorComponent");
  assert.ok(nftCode.includes("throw notFound()"), "NFT loader must throw notFound() on missing NFT");
});

test("Default router configuration provides defaultNotFoundComponent", async () => {
  const routerCode = await readFile(
    new URL("../src/router.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(routerCode.includes("defaultNotFoundComponent: AppNotFoundComponent"), "Router must register AppNotFoundComponent");
});

test("Mainnet dispatchApi returns 503 for telemetry endpoints prior to 1.1.2027", async () => {
  const headReq = new Request("http://localhost/api/head?source=mainnet");
  const headRes = await dispatchApi(headReq);
  assert.equal(headRes.status, 503);
  const headText = await headRes.text();
  assert.ok(headText.includes("mainnet launching soon"), `Expected 503 mainnet launching soon, got: ${headText}`);

  const stateReq = new Request("http://localhost/api/state?source=mainnet");
  const stateRes = await dispatchApi(stateReq);
  assert.equal(stateRes.status, 503);

  const bootReq = new Request("http://localhost/api/bootstrap?source=mainnet");
  const bootRes = await dispatchApi(bootReq);
  assert.equal(bootRes.status, 503);
});

test("Mainnet dispatchApi rejects mutation / POST requests with 403", async () => {
  const prepReq = new Request("http://localhost/api/prepare?source=mainnet", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({}),
  });
  const prepRes = await dispatchApi(prepReq);
  assert.equal(prepRes.status, 403);
  const prepText = await prepRes.text();
  assert.ok(prepText.includes("mainnet transactions disabled prior to 1.1.2027 launch"));

  const submitReq = new Request("http://localhost/api/submit?source=mainnet", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({}),
  });
  const submitRes = await dispatchApi(submitReq);
  assert.equal(submitRes.status, 403);
});

test("Mainnet dispatchApi returns mainnet spec containing kovanica-mainnet and KVNC", async () => {
  const specReq = new Request("http://localhost/api/spec?source=mainnet");
  const specRes = await dispatchApi(specReq);
  assert.equal(specRes.status, 200);
  const specText = await specRes.text();
  assert.ok(specText.includes("KVNC"), "Spec must contain KVNC");
  assert.ok(specText.includes("kovanica-mainnet"), "Spec must contain kovanica-mainnet");
  assert.ok(specText.includes("GHOSTDAG K = 5"), "Spec must contain GHOSTDAG K = 5");
});

test("Docs view contains no hardcoded 4650 block count or (Val 2) validator strings", async () => {
  const docsCode = await readFile(
    new URL("../src/components/docs/docs-view.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(!docsCode.includes("4650"), "docs-view.tsx must not contain hardcoded 4650 block count fallback");
  assert.ok(!docsCode.includes("(Val 2)"), "docs-view.tsx must not contain old (Val 2) validator label");
  assert.ok(docsCode.includes("Connecting to node telemetry…"), "docs-view.tsx must describe loading state honestly");
  assert.ok(docsCode.includes("Telemetry unavailable · Node offline"), "docs-view.tsx must describe offline state honestly");
});

test("dispatch.server.ts does not reference old 'Chain 4' genesis", async () => {
  const dispatchCode = await readFile(
    new URL("../src/lib/api/dispatch.server.ts", import.meta.url),
    "utf8",
  );
  assert.ok(!dispatchCode.includes('"Chain 4"'), "dispatch.server.ts must not reference old Chain 4 genesis");
});

test("Explorer and Network views handle mainnet source honestly without infinite loading", async () => {
  const explorerCode = await readFile(
    new URL("../src/components/explorer/explorer-view.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(explorerCode.includes("MainnetExplorerPlaceholder"), "explorer-view.tsx must provide a dedicated Mainnet placeholder");
  assert.ok(explorerCode.includes("1.1.2027"), "explorer-view.tsx placeholder must announce 1.1.2027 launch");

  const networkCode = await readFile(
    new URL("../src/components/network/network-view.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(networkCode.includes("MainnetNetworkView"), "network-view.tsx must provide a dedicated Mainnet placeholder");
  assert.ok(networkCode.includes("1.1.2027"), "network-view.tsx placeholder must announce 1.1.2027 launch");
});

test("Wallet view prevents transactions on mainnet source", async () => {
  const walletCode = await readFile(
    new URL("../src/components/wallet/wallet-view.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(walletCode.includes("Mainnet Prelaunch · Launch Scheduled 1.1.2027"), "Wallet must explain mainnet restrictions");
  assert.ok(walletCode.includes('toast.error("Mainnet transactions are disabled prior to 1.1.2027 launch")'), "Wallet toast must block send on mainnet");
  assert.ok(walletCode.includes("Mainnet disabled (launch 1.1.2027)"), "Submit button must display disabled status on mainnet");
});
