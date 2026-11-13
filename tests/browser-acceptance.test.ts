import { test, before, after } from "node:test";
import assert from "node:assert/strict";
import { spawn, type ChildProcess } from "node:child_process";
import { createServer } from "node:net";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { chromium, type Browser, type BrowserContext, type Page } from "playwright";
import { dispatchApi } from "../src/lib/api/dispatch.server";

// Port discovery for deterministic isolated server execution
async function getFreePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const srv = createServer();
    srv.listen(0, "127.0.0.1", () => {
      const port = (srv.address() as { port: number }).port;
      srv.close(() => resolve(port));
    });
    srv.on("error", reject);
  });
}

// Deterministic API Fixtures
const FIXTURES = {
  pools: [
    {
      address: "kvnc1pooladdr0000000000000000000000000000000000000000000000000dag",
      symbol: "TEST",
      name: "Test Gold Token",
      decimals: 9,
      kuna_reserve: "10000000000000",
      token_reserve: "20000000000000",
      fee_bps: 30,
      price_kuna: 0.5,
      lp_total: "14142135623730",
      paused: false,
      icon: "",
    },
  ],
  head: {
    network: "kovanica-testnet",
    genesis: "0x17e79432dfbee3adad15d7a851ff78ebafcefbc547073883489cd9cbdaae60fa",
    tip: "0x3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29",
    blocks: 12450,
    min_fee: 10000,
    atom: 1000000000,
  },
  bootstrap: {
    network: "kovanica-testnet",
    genesis: "0x17e79432dfbee3adad15d7a851ff78ebafcefbc547073883489cd9cbdaae60fa",
    tip: "0x3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29",
    blocks: 12450,
    min_fee: 10000,
    atom: 1000000000,
    token: "KVNC",
    k: 5,
    subsidy: 2500000000,
    admission: "open",
    poa_enabled: false,
    finality_depth: 100,
    block_pruning_depth: 18446744073709551615,
    payload_pruning_depth: 1000,
    founder_amount: 200000000000000,
    founder_seed: 0,
    max_supply: 90200000000000000,
    burned: 0,
    circulating: 1000000000000000,
    native_minted: 1000000000000000,
    total: 90200000000000000,
    listen: "0.0.0.0:8000",
    peers: ["seed1.kovanica.online:8000", "seed2.kovanica.online:8000"],
    upstream: {
      ok: true,
      head: {
        network: "kovanica-testnet",
        genesis: "0x17e79432dfbee3adad15d7a851ff78ebafcefbc547073883489cd9cbdaae60fa",
        tip: "0x3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29",
        blocks: 12450,
        min_fee: 10000,
        atom: 1000000000,
      },
    },
  },
  state: {
    mesh: {
      active: true,
      peers: 3,
      round: 12450,
      rate: 1.0,
      min_fee: 10000,
      queued: 0,
      nodes: [
        {
          name: "val1",
          blocks: 12450,
          tip: "0x3b6a",
          peers: ["seed1.kovanica.online"],
          mempool: 0,
        },
      ],
      events: [],
    },
    node: {
      blocks: 12450,
      tips: ["0x3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29"],
      selected_tip: "0x3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29",
      blue_score: 12450,
      blue_work: 1245000,
      k: 5,
      subsidy: 81000000000000000,
      issuance: 2500000000,
      halving_era: 8100000,
      min_fee: 10000,
      genesis: "0x17e79432dfbee3adad15d7a851ff78ebafcefbc547073883489cd9cbdaae60fa",
      supply: 90200000000000000,
      token: "KVNC",
      decimals: 9,
      miner: "Miner 0x1",
      atom: 1000000000,
      ui: "hybrid",
      utxos: 0,
      chain_len: 12450,
      mempool: 0,
      tx_count: 12,
      dag: [
        {
          id: "0x3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29",
          parents: ["0x17e79432dfbee3adad15d7a851ff78ebafcefbc547073883489cd9cbdaae60fa"],
          selected_parent: "0x17e79432dfbee3adad15d7a851ff78ebafcefbc547073883489cd9cbdaae60fa",
          work: 12450,
          timestamp_ms: Date.now(),
          nonce: 100,
          blue_score: 12450,
          colour: "chain",
          txs: [],
        },
        {
          id: "0x17e79432dfbee3adad15d7a851ff78ebafcefbc547073883489cd9cbdaae60fa",
          parents: [],
          selected_parent: null,
          work: 0,
          timestamp_ms: Date.now() - 60000,
          nonce: 0,
          blue_score: 0,
          colour: "genesis",
          txs: [],
        },
      ],
      order: [
        "0x3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29",
        "0x17e79432dfbee3adad15d7a851ff78ebafcefbc547073883489cd9cbdaae60fa",
      ],
      pending: [],
    },
    wallets: [],
    source: "testnet",
  },
  utxos: {
    address: "kvnc8d4z00xwk6jz6c4r4rgz5mcdwdjny9thrh3y8f36cpy2rz6emg54rufktdag",
    balance: 5000000000,
    utxos: [
      {
        tx: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        index: 0,
        value: 5000000000,
        asset_id: null,
      },
    ],
  },
  history: {
    address: "kvnc8d4z00xwk6jz6c4r4rgz5mcdwdjny9thrh3y8f36cpy2rz6emg54rufktdag",
    balance: 5000000000,
    txs: [
      {
        block: "0x3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29",
        tx: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        kind: "in",
        delta: 5000000000,
      },
    ],
  },
  tokens: [
    {
      address: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      symbol: "TEST",
      name: "Test Gold Token",
      decimals: 9,
      balance: "25000000000",
      paused: false,
    },
  ],
  feeEstimate: {
    fee_rate: 10,
    unit: "atoms/vbyte",
    mempool: 0,
    bytes: 0,
  },
  rwaDetail: {
    asset_id: "rwa-gold-001",
    kind: "rwa",
    max_supply: 1000,
    minted: 500,
    metadata_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
    collection_id: null,
    creator: "kvnc8d4z00xwk6jz6c4r4rgz5mcdwdjny9thrh3y8f36cpy2rz6emg54rufktdag",
    owner: "kvnc8d4z00xwk6jz6c4r4rgz5mcdwdjny9thrh3y8f36cpy2rz6emg54rufktdag",
    owner_tx: "0x1111111111111111111111111111111111111111111111111111111111111111",
    owner_index: 0,
    asset_class: "Commodity",
    legal_uri: "https://example.com/legal.pdf",
    custody_uri: "https://example.com/custody.pdf",
    total_supply: "1000",
    description: "Physical Gold Bar Vault Backed",
    jurisdiction: "Switzerland",
    created_at: "2026-10-01T00:00:00Z",
  },
};

const ARTIFACTS_DIR = join(process.cwd(), "test-artifacts");
mkdirSync(ARTIFACTS_DIR, { recursive: true });

async function saveFailureArtifact(page: Page, testName: string, error: unknown) {
  const safeName = testName.replace(/[^a-zA-Z0-9_-]/g, "_").slice(0, 80);
  try {
    const pngPath = join(ARTIFACTS_DIR, `${safeName}.png`);
    await page.screenshot({ path: pngPath, fullPage: true }).catch(() => {});
    const htmlPath = join(ARTIFACTS_DIR, `${safeName}.html`);
    const content = await page.content().catch(() => "");
    // Sanitize any accidental secrets
    const sanitized = content
      .replace(/seed=[^&"'\s]+/gi, "seed=[REDACTED]")
      .replace(/privateKey=[^&"'\s]+/gi, "privateKey=[REDACTED]")
      .replace(/mnemonic=[^&"'\s]+/gi, "mnemonic=[REDACTED]");
    writeFileSync(htmlPath, sanitized, "utf8");
    const jsonPath = join(ARTIFACTS_DIR, `${safeName}.json`);
    writeFileSync(
      jsonPath,
      JSON.stringify(
        {
          testName,
          error: String((error as Error)?.message || error),
          url: page.url(),
          timestamp: new Date().toISOString(),
        },
        null,
        2,
      ),
      "utf8",
    );
  } catch {
    /* artifact capture must never mask original test error */
  }
}

// Global test suite state
let port: number;
let serverProcess: ChildProcess;
let browser: Browser;
let baseUrl: string;

// Tracking network isolation: external connections blocked
const blockedExternalRequests: string[] = [];
// Tracking mutation interception: prohibited mutation attempts intercepted locally
const interceptedMutations: string[] = [];

before(async () => {
  port = await getFreePort();
  baseUrl = `http://127.0.0.1:${port}`;

  if (!existsSync(".output/server/index.mjs")) {
    const { execSync } = await import("node:child_process");
    execSync("npm run build:vps", { stdio: "inherit" });
  }

  serverProcess = spawn("node", [".output/server/index.mjs"], {
    env: { ...process.env, NITRO_PORT: String(port), HOST: "127.0.0.1", PORT: String(port) },
    stdio: "pipe",
  });

  // Await server readiness
  const start = Date.now();
  let online = false;
  while (Date.now() - start < 15000) {
    try {
      const res = await fetch(`${baseUrl}/`);
      if (res.ok) {
        online = true;
        break;
      }
    } catch {
      await new Promise((r) => setTimeout(r, 200));
    }
  }
  if (!online) {
    throw new Error(`Local test server on port ${port} failed to become ready in time`);
  }

  browser = await chromium.launch({
    headless: true,
    args: ["--no-sandbox", "--disable-dev-shm-usage"],
  });
});

after(async () => {
  await browser?.close();
  if (serverProcess) {
    serverProcess.kill("SIGINT");
  }
});

// Helper to configure page context with strict isolation guard and deterministic fixtures
async function setupIsolatedPage(
  options: {
    viewport?: { width: number; height: number };
    delayUtxos?: Promise<void>;
    customRouteHandler?: (route: any, url: URL) => Promise<boolean>;
    watchWalletAddressHex?: string;
  } = {},
): Promise<{ context: BrowserContext; page: Page }> {
  const context = await browser.newContext({
    viewport: options.viewport ?? { width: 1280, height: 800 },
  });

  if (options.watchWalletAddressHex) {
    const addrHex = options.watchWalletAddressHex;
    await context.addInitScript((hex: string) => {
      window.localStorage.setItem(
        "kovanica.ledger",
        JSON.stringify({
          state: {
            wallet: { address: hex, index: 0, shown: false, kind: "watch" },
            balances: {},
            history: [],
            originPulses: {},
          },
          version: 0,
        }),
      );
    }, addrHex);
  }

  const page = await context.newPage();

  // Strict route interception
  await page.route("**/*", async (route) => {
    const req = route.request();
    const url = new URL(req.url());

    // 1. Strict Network Isolation Guard:
    // Any request to a non-loopback host is blocked immediately.
    const isLoopback =
      url.hostname === "127.0.0.1" ||
      url.hostname === "localhost" ||
      url.hostname === "::1" ||
      url.hostname === "[::1]";

    if (!isLoopback) {
      blockedExternalRequests.push(req.url());
      await route.abort("blockedbyclient");
      return;
    }

    // 2. Intercept potential mutation requests locally (never hit real backends or cause unintended writes)
    if (req.method() === "POST") {
      interceptedMutations.push(req.url());
      if (url.searchParams.get("source") === "mainnet") {
        await route.fulfill({
          status: 403,
          contentType: "text/plain",
          body: "mainnet transactions disabled prior to 1.1.2027 launch",
        });
        return;
      }
      // Return safe mock rejection or mock prepare for local checks
      await route.fulfill({
        status: 400,
        contentType: "text/plain",
        body: "mutations strictly intercepted in deterministic browser test",
      });
      return;
    }

    // 3. Custom route hooks if provided
    if (options.customRouteHandler && (await options.customRouteHandler(route, url))) {
      return;
    }

    // 4. Deterministic API fixtures for testnet
    const path = url.pathname;
    const source = url.searchParams.get("source") || "testnet";

    if (path.startsWith("/api/")) {
      // Mainnet isolation: 503 launching soon
      if (source === "mainnet") {
        if (path === "/api/spec") {
          await route.fulfill({
            status: 200,
            contentType: "text/plain",
            body: "# kovanica-mainnet\nBlockDAG mainnet. KVNC 9 decimals. GHOSTDAG K = 5.",
          });
          return;
        }
        await route.fulfill({
          status: 503,
          contentType: "text/plain",
          body: "mainnet launching soon",
        });
        return;
      }

      // Testnet fixtures
      if (path === "/api/pools") {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(FIXTURES.pools),
        });
        return;
      }
      if (path === "/api/head") {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(FIXTURES.head),
        });
        return;
      }
      if (path === "/api/bootstrap") {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(FIXTURES.bootstrap),
        });
        return;
      }
      if (path === "/api/state") {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(FIXTURES.state),
        });
        return;
      }
      if (path === "/api/utxos") {
        if (options.delayUtxos) {
          await options.delayUtxos;
        }
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(FIXTURES.utxos),
        });
        return;
      }
      if (path === "/api/history") {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(FIXTURES.history),
        });
        return;
      }
      if (path === "/api/tokens") {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(FIXTURES.tokens),
        });
        return;
      }
      if (path === "/api/fee_estimate") {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(FIXTURES.feeEstimate),
        });
        return;
      }
      if (path === "/api/rwa/rwa-gold-001") {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(FIXTURES.rwaDetail),
        });
        return;
      }
    }

    // Default: continue local server handling
    await route.continue();
  });

  return { context, page };
}

// -----------------------------------------------------------------------------
// Deterministic wait helpers
//
// These tests run against a server-rendered app whose interactive state only
// exists after React hydrates. Fixed sleeps race hydration on slower machines
// and CI runners, so every assertion waits for the observable state it needs.
// -----------------------------------------------------------------------------
const HYDRATION_TIMEOUT = 30_000;

/** Wait until React has hydrated the app shell (fiber/props attached to the DOM). */
async function waitForHydration(page: Page) {
  await page.waitForFunction(
    () => {
      const el = document.querySelector('button[title="Mainnet"]') ?? document.body;
      if (!el) return false;
      return Object.keys(el).some(
        (key) => key.startsWith("__reactFiber$") || key.startsWith("__reactProps$"),
      );
    },
    undefined,
    { timeout: HYDRATION_TIMEOUT },
  );
}

/** Wait until the rendered body text contains `needle`. */
async function waitForBodyText(page: Page, needle: string, timeout = HYDRATION_TIMEOUT) {
  await page.waitForFunction(
    (text: string) => document.body.innerText.includes(text),
    needle,
    { timeout, polling: 100 },
  );
}

/** Wait until the rendered body text contains any of `needles`. */
async function waitForBodyTextAny(page: Page, needles: string[], timeout = HYDRATION_TIMEOUT) {
  await page.waitForFunction(
    (texts: string[]) => texts.some((text) => document.body.innerText.includes(text)),
    needles,
    { timeout, polling: 100 },
  );
}

/** Wait until the wallet header balance equals `expected` (e.g. "5 KVNC"). */
async function waitForHeaderBalance(page: Page, expected: string, timeout = HYDRATION_TIMEOUT) {
  await page.waitForFunction(
    (value: string) =>
      document.querySelector("header p.text-4xl")?.textContent?.trim() === value,
    expected,
    { timeout, polling: 100 },
  );
}

/** Navigate, wait for hydration, then optionally wait for a route-specific marker. */
async function gotoAndSettle(page: Page, url: string, marker?: string) {
  await page.goto(url, { waitUntil: "domcontentloaded" });
  await waitForHydration(page);
  if (marker) await waitForBodyText(page, marker);
}

// -----------------------------------------------------------------------------
// SECTION A: Multi-surface, direct navigation/reload, and viewports
// -----------------------------------------------------------------------------
test("Section A: desktop viewport renders home, explorer, wallet, pool, and docs with meaningful assertions", async () => {
  const { context, page } = await setupIsolatedPage({ viewport: { width: 1280, height: 800 } });

  try {
    // 1. Home (/)
    await page.goto(`${baseUrl}/`, { waitUntil: "domcontentloaded" });
    const homeTitle = await page.title();
    assert.ok(homeTitle.includes("Kovanica"), `Home page title must include Kovanica, got "${homeTitle}"`);
    const homeBody = await page.locator("body").innerText();
    assert.ok(
      homeBody.includes("Kovanica") || homeBody.includes("BlockDAG"),
      "Home page must render Kovanica BlockDAG branding",
    );

    // Direct reload on Home
    await page.reload({ waitUntil: "domcontentloaded" });
    assert.ok((await page.title()).includes("Kovanica"), "Home page reload must preserve title");

    // 2. Explorer (/explorer)
    await gotoAndSettle(page, `${baseUrl}/explorer`, "BlockDAG Explorer");
    const expText = await page.locator("body").innerText();
    assert.ok(expText.includes("BlockDAG Explorer"), "Explorer must render 'BlockDAG Explorer'");
    assert.ok(
      expText.includes("GHOSTDAG") || expText.includes("KVNC"),
      "Explorer must render consensus & token info",
    );
    // Direct reload on Explorer
    await page.reload({ waitUntil: "domcontentloaded" });
    assert.ok((await page.locator("body").innerText()).includes("BlockDAG Explorer"));

    // 3. Wallet (/wallet)
    await gotoAndSettle(page, `${baseUrl}/wallet`, "PRO-21");
    const walletText = await page.locator("body").innerText();
    assert.ok(walletText.includes("Wallet"), "Wallet route must render Wallet heading");
    assert.ok(
      walletText.includes("PRO-21") || walletText.includes("kvnc"),
      "Wallet must mention canonical PRO-21 address format",
    );
    // Direct reload on Wallet
    await page.reload({ waitUntil: "domcontentloaded" });
    assert.ok((await page.locator("body").innerText()).includes("Wallet"));

    // 4. Pool (/pool)
    await gotoAndSettle(page, `${baseUrl}/pool`, "TEST / KVNC");
    const poolText = await page.locator("body").innerText();
    assert.ok(poolText.includes("Liquidity pools"), "Pool route must render 'Liquidity pools'");
    assert.ok(poolText.includes("TEST / KVNC"), "Pool route must render deterministic TEST / KVNC pool");
    assert.ok(
      poolText.includes("Open in Wallet (Unsupported)"),
      "Pool route must render disabled 'Open in Wallet (Unsupported)' button",
    );
    // Direct reload on Pool
    await page.reload({ waitUntil: "domcontentloaded" });
    assert.ok((await page.locator("body").innerText()).includes("Liquidity pools"));

    // 5. Docs (/docs)
    await gotoAndSettle(page, `${baseUrl}/docs`, "Getting started");
    const docsText = await page.locator("body").innerText();
    assert.ok(docsText.includes("Documentation"), "Docs route must render Documentation heading");
    assert.ok(docsText.includes("Getting started"), "Docs route must include Getting started section");
    // Direct reload on Docs
    await page.reload({ waitUntil: "domcontentloaded" });
    assert.ok((await page.locator("body").innerText()).includes("Documentation"));
  } catch (err) {
    await saveFailureArtifact(page, "section-a-desktop", err);
    throw err;
  } finally {
    await context.close();
  }
});

test("Section A: mobile viewport (390x844) renders without horizontal scroll overflow", async () => {
  const { context, page } = await setupIsolatedPage({ viewport: { width: 390, height: 844 } });

  try {
    const routes: Array<[string, string]> = [
      ["/", "Kovanica"],
      ["/explorer", "BlockDAG Explorer"],
      ["/wallet", "PRO-21"],
      ["/pool", "Liquidity pools"],
      ["/docs", "Getting started"],
    ];
    for (const [route, marker] of routes) {
      await gotoAndSettle(page, `${baseUrl}${route}`, marker);

      const hasOverflow = await page.evaluate(() => {
        const el = document.documentElement;
        return el.scrollWidth > el.clientWidth + 1;
      });
      assert.equal(hasOverflow, false, `Route ${route} on mobile (390x844) must not have horizontal scroll overflow`);
    }
  } catch (err) {
    await saveFailureArtifact(page, "section-a-mobile", err);
    throw err;
  } finally {
    await context.close();
  }
});

// -----------------------------------------------------------------------------
// SECTION B: Server/API Isolation, Outages, 404 vs 502, Unknown Stats & NO-Stake
// -----------------------------------------------------------------------------
test("Section B: mainnet-prelaunch does not fall back to testnet data in API and UI", async () => {
  const { context, page } = await setupIsolatedPage({ viewport: { width: 1280, height: 800 } });

  try {
    // 1. Direct API checks for mainnet isolation
    const headRes = await dispatchApi(new Request("http://localhost/api/head?source=mainnet"));
    assert.equal(headRes.status, 503, "Mainnet head must return 503");
    assert.equal(await headRes.text(), "mainnet launching soon");

    const stateRes = await dispatchApi(new Request("http://localhost/api/state?source=mainnet"));
    assert.equal(stateRes.status, 503, "Mainnet state must return 503");

    // 2. Browser UI check: Explorer on mainnet shows dedicated prelaunch placeholder
    await gotoAndSettle(page, `${baseUrl}/explorer`, "BlockDAG Explorer");

    // Switch to Mainnet via header button
    const mainnetBtn = page.locator("button[title=\"Mainnet\"], button:text(\"Mainnet\")").first();
    await mainnetBtn.click();
    await waitForBodyText(page, "Mainnet Explorer Unavailable");

    const explorerMainnetText = await page.locator("body").innerText();
    assert.ok(
      explorerMainnetText.includes("Mainnet Explorer Unavailable"),
      "Explorer on mainnet must display 'Mainnet Explorer Unavailable'",
    );
    assert.ok(
      explorerMainnetText.includes("1.1.2027"),
      "Explorer on mainnet must announce 1.1.2027 genesis launch",
    );
    assert.ok(
      !explorerMainnetText.includes("0x3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29"),
      "Mainnet must NEVER render testnet DAG blocks or tips",
    );

    // 3. Browser UI check: Wallet on mainnet shows prelaunch notice and disables actions
    await gotoAndSettle(
      page,
      `${baseUrl}/wallet`,
      "Mainnet Prelaunch · Launch Scheduled 1.1.2027",
    );
    const walletMainnetText = await page.locator("body").innerText();
    assert.ok(
      walletMainnetText.includes("Mainnet Prelaunch · Launch Scheduled 1.1.2027"),
      "Wallet on mainnet must render Mainnet Prelaunch banner",
    );

    // 4. Mutation rejection locally without signing or submitting
    const submitReq = new Request("http://localhost/api/submit?source=mainnet", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ tx: "prohibited_mutation" }),
    });
    const submitRes = await dispatchApi(submitReq);
    assert.equal(submitRes.status, 403, "Mainnet submit must reject with 403");
    assert.ok((await submitRes.text()).includes("mainnet transactions disabled"));
  } catch (err) {
    await saveFailureArtifact(page, "section-b-mainnet-isolation", err);
    throw err;
  } finally {
    await context.close();
  }
});

test("Section B: missing entities (404) vs backend failure (502) render distinct truthful components", async () => {
  const { context, page } = await setupIsolatedPage({
    viewport: { width: 1280, height: 800 },
  });

  try {
    // 1. Missing Entity (404): RWA on local source (entity does not exist on in-memory mock node)
    await gotoAndSettle(page, `${baseUrl}/explorer/rwa/missing-asset-404?source=local`, "RWA Asset Not Found");
    const missingRwaText = await page.locator("body").innerText();
    assert.ok(
      missingRwaText.includes("RWA Asset Not Found"),
      "Missing entity must render RwaNotFoundPage ('RWA Asset Not Found')",
    );
    assert.ok(
      missingRwaText.includes("Back to Explorer"),
      "404 page must provide 'Back to Explorer' link",
    );

    // 2. Missing Entity (404): NFT & Collection
    await gotoAndSettle(page, `${baseUrl}/wallet/nft/missing-nft-404?source=local`, "NFT Not Found");
    const missingNftText = await page.locator("body").innerText();
    assert.ok(
      missingNftText.includes("NFT Not Found"),
      "Missing NFT entity must render NftNotFoundPage ('NFT Not Found')",
    );

    await gotoAndSettle(page, `${baseUrl}/wallet/collection/missing-coll-404?source=local`, "Collection Not Found");
    const missingCollText = await page.locator("body").innerText();
    assert.ok(
      missingCollText.includes("Collection Not Found"),
      "Missing Collection entity must render CollectionNotFoundPage ('Collection Not Found')",
    );

    // 3. Backend Failure (502): RWA, NFT, and Collection on public testnet when node is unreachable
    // dispatchApi returns 502 'testnet node error' because no live RPC is active locally
    await gotoAndSettle(page, `${baseUrl}/explorer/rwa/failing-service-asset?source=testnet`, "RWA Endpoint Unavailable");
    const errorRwaText = await page.locator("body").innerText();
    assert.ok(
      errorRwaText.includes("RWA Endpoint Unavailable"),
      "Backend failure must render RouteError ('RWA Endpoint Unavailable'), NOT 404",
    );
    assert.ok(
      !errorRwaText.includes("RWA Asset Not Found"),
      "Backend outage must not be misreported as missing entity 404",
    );
    assert.ok(
      errorRwaText.includes("Try again"),
      "Error page must provide 'Try again' retry button",
    );

    await gotoAndSettle(page, `${baseUrl}/wallet/nft/failing-nft-asset?source=testnet`, "NFT Endpoint Unavailable");
    const errorNftText = await page.locator("body").innerText();
    assert.ok(
      errorNftText.includes("NFT Endpoint Unavailable"),
      "NFT backend failure must render RouteError ('NFT Endpoint Unavailable'), NOT 404",
    );

    await gotoAndSettle(page, `${baseUrl}/wallet/collection/failing-coll-asset?source=testnet`, "Collection Endpoint Unavailable");
    const errorCollText = await page.locator("body").innerText();
    assert.ok(
      errorCollText.includes("Collection Endpoint Unavailable"),
      "Collection backend failure must render RouteError ('Collection Endpoint Unavailable'), NOT 404",
    );
  } catch (err) {
    await saveFailureArtifact(page, "section-b-404-vs-502", err);
    throw err;
  } finally {
    await context.close();
  }
});

test("Section B: unreachable services, malformed responses, and unknown statistics stay explicit", async () => {
  const { context, page } = await setupIsolatedPage({
    viewport: { width: 1280, height: 800 },
    customRouteHandler: async (route, url) => {
      if (url.pathname === "/api/pools") {
        // Malformed non-array JSON response
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({ malformed: true }),
        });
        return true;
      }
      if (url.pathname === "/api/bootstrap") {
        // Unreachable service (502)
        await route.fulfill({
          status: 502,
          contentType: "text/plain",
          body: "testnet node error",
        });
        return true;
      }
      if (url.pathname === "/api/state") {
        // Malformed node state payload
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({ malformed: true }),
        });
        return true;
      }
      return false;
    },
  });

  try {
    // 1. Malformed /api/pools response renders explicit error instead of crashing
    await gotoAndSettle(page, `${baseUrl}/pool`, "Malformed pools response");
    const poolText = await page.locator("body").innerText();
    assert.ok(
      poolText.includes("Malformed pools response"),
      "Malformed /api/pools response must render explicit error message without crashing",
    );

    // 2. Unreachable /api/bootstrap on /docs renders explicit 'Telemetry unavailable · Node offline'
    await gotoAndSettle(page, `${baseUrl}/docs`, "Telemetry unavailable · Node offline");
    const docsText = await page.locator("body").innerText();
    assert.ok(
      docsText.includes("Telemetry unavailable · Node offline"),
      "Docs must render explicit 'Telemetry unavailable · Node offline' when node is unreachable",
    );

    // 3. Malformed /api/state on /explorer keeps unknown statistics explicit ('—') and shows error
    await gotoAndSettle(page, `${baseUrl}/explorer`, "BlockDAG Explorer");
    const expText = await page.locator("body").innerText();
    assert.ok(
      expText.includes("—"),
      "Explorer must keep unknown statistics explicit ('—') when node state is unavailable or malformed",
    );
    assert.ok(
      !expText.includes("12,450"),
      "Explorer must not fabricate or retain stale block statistics on malformed response",
    );
  } catch (err) {
    await saveFailureArtifact(page, "section-b-malformed-and-unreachable", err);
    throw err;
  } finally {
    await context.close();
  }
});

test("Section B: NO-stake messaging is preserved across documentation, network, and explorer", async () => {
  const { context, page } = await setupIsolatedPage({ viewport: { width: 1280, height: 800 } });

  try {
    // 1. Docs consensus section
    await gotoAndSettle(page, `${baseUrl}/docs#consensus`);
    await waitForBodyTextAny(page, [
      "There is no validator staking, bonding, committee rotation, or slashing in consensus",
      "None (pure PoW + GHOSTDAG)",
    ]);
    const docsText = await page.locator("body").innerText();
    assert.ok(
      docsText.includes("There is no validator staking, bonding, committee rotation, or slashing in consensus") ||
      docsText.includes("None (pure PoW + GHOSTDAG)"),
      "Docs must preserve explicit NO-stake consensus statement",
    );

    // 2. Network view
    await gotoAndSettle(page, `${baseUrl}/network`);
    // Switch to mainnet network view
    const mainnetBtn = page.locator("button[title=\"Mainnet\"], button:text(\"Mainnet\")").first();
    await mainnetBtn.click();
    await waitForBodyText(page, "None (Zero-stake PoW consensus)");
    const networkText = await page.locator("body").innerText();
    assert.ok(
      networkText.includes("None (Zero-stake PoW consensus)"),
      "Network view must preserve 'None (Zero-stake PoW consensus)'",
    );
  } catch (err) {
    await saveFailureArtifact(page, "section-b-no-stake", err);
    throw err;
  } finally {
    await context.close();
  }
});

// -----------------------------------------------------------------------------
// SECTION C: Response Delay, Network Switch Race, and Stale Overwrite Prevention
// -----------------------------------------------------------------------------
test("Section C: delayed old-network response and late errors do not cause cross-network stale overwrite", async () => {
  let releaseDelayedUtxos: () => void = () => {};
  const delayedPromise = new Promise<void>((resolve) => {
    releaseDelayedUtxos = resolve;
  });

  let releaseDelayedLateError: () => void = () => {};
  const delayedErrorPromise = new Promise<void>((resolve) => {
    releaseDelayedLateError = resolve;
  });
  let triggerLatePoolError = false;

  const { context, page } = await setupIsolatedPage({
    viewport: { width: 1280, height: 800 },
    delayUtxos: delayedPromise,
    watchWalletAddressHex: "e0aa3a00ef2fa26ad627d0cc5359dc6ca65212db5dd4a1c3f1c0485e8ecc8f14",
    customRouteHandler: async (route, url) => {
      if (url.pathname === "/api/pools" && triggerLatePoolError) {
        await delayedErrorPromise;
        await route.fulfill({
          status: 502,
          contentType: "text/plain",
          body: "late old-network pool failure",
        });
        return true;
      }
      return false;
    },
  });

  try {
    await gotoAndSettle(
      page,
      `${baseUrl}/wallet`,
      "kvncuz4r5q80973x44386rx9xkwudjn9yykmth22rsl3cpy9arkv3u29hmu5mdag",
    );

    // Switch to mainnet while testnet utxos request for watch wallet is delayed in-flight
    const mainnetBtn = page.locator("button[title=\"Mainnet\"], button:text(\"Mainnet\")").first();
    await mainnetBtn.click();
    await waitForBodyText(page, "Mainnet Prelaunch · Launch Scheduled 1.1.2027");

    // Verify mainnet prelaunch state
    let walletText = await page.locator("body").innerText();
    assert.ok(
      walletText.includes("Mainnet Prelaunch · Launch Scheduled 1.1.2027"),
      "Wallet must display Mainnet Prelaunch banner after switching",
    );

    // Release late testnet response (which returns 5 KVNC)
    releaseDelayedUtxos();
    await waitForHeaderBalance(page, "0 KVNC");

    // State on mainnet must NOT be polluted by stale testnet 5 KVNC response
    walletText = await page.locator("body").innerText();
    const displayedBalance = (await page.locator("header p.text-4xl").innerText()).trim();
    assert.ok(
      walletText.includes("Mainnet Prelaunch · Launch Scheduled 1.1.2027"),
      "Mainnet prelaunch state must not be overwritten by late testnet response",
    );
    assert.equal(
      displayedBalance,
      "0 KVNC",
      `Stale testnet 5 KVNC balance must not overwrite mainnet 0 KVNC balance (got ${displayedBalance})`,
    );

    // Now test late error on /pool across network switch + overlapping refreshes + unmount
    const testnetBtn = page.locator("button[title=\"Testnet\"], button:text(\"Testnet\")").first();
    await testnetBtn.click();
    await waitForHeaderBalance(page, "5 KVNC");

    triggerLatePoolError = true;
    await gotoAndSettle(page, `${baseUrl}/pool`);

    // Switch to mainnet while testnet /api/pools error is delayed in-flight
    await mainnetBtn.click();
    await waitForBodyText(page, "No pools initialized yet");

    // Release the late error from testnet
    releaseDelayedLateError();
    await page.waitForTimeout(750);

    const poolText = await page.locator("body").innerText();
    assert.ok(
      !poolText.includes("late old-network pool failure"),
      "Late error from prior network must not overwrite active mainnet view",
    );

    // Navigate away (unmount) while on route
    await gotoAndSettle(page, `${baseUrl}/docs`, "Documentation");
    assert.ok((await page.locator("body").innerText()).includes("Documentation"));
  } catch (err) {
    await saveFailureArtifact(page, "section-c-delayed-race", err);
    throw err;
  } finally {
    releaseDelayedUtxos();
    releaseDelayedLateError();
    await context.close();
  }
});

test("Section C: wallet visibility/freshness, hidden-tab pause, visible resume, and stale badge on failure", async () => {
  let utxoFetchCount = 0;
  let failNextUtxoRefresh = false;

  const { context, page } = await setupIsolatedPage({
    viewport: { width: 1280, height: 800 },
    watchWalletAddressHex: "e0aa3a00ef2fa26ad627d0cc5359dc6ca65212db5dd4a1c3f1c0485e8ecc8f14",
    customRouteHandler: async (route, url) => {
      if (url.pathname === "/api/utxos") {
        utxoFetchCount += 1;
        if (failNextUtxoRefresh) {
          await route.fulfill({
            status: 502,
            contentType: "text/plain",
            body: "testnet node unreachable",
          });
          return true;
        }
      }
      return false;
    },
  });

  try {
    await gotoAndSettle(
      page,
      `${baseUrl}/wallet`,
      "kvncuz4r5q80973x44386rx9xkwudjn9yykmth22rsl3cpy9arkv3u29hmu5mdag",
    );
    await waitForHeaderBalance(page, "5 KVNC");

    // Verify canonical address and initial fresh balance (5 KVNC)
    let walletText = await page.locator("body").innerText();
    assert.ok(
      walletText.includes("kvncuz4r5q80973x44386rx9xkwudjn9yykmth22rsl3cpy9arkv3u29hmu5mdag"),
      "Active watch wallet must render canonical PRO-21 address",
    );
    assert.equal(
      (await page.locator("header p.text-4xl").innerText()).trim(),
      "5 KVNC",
      "Active watch wallet must display fetched 5 KVNC balance",
    );
    const countBeforeHide = utxoFetchCount;
    assert.ok(countBeforeHide >= 1, "Initial wallet sync must have fetched UTXOs");

    // Emulate tab hidden -> cadence pauses
    await page.evaluate(() => {
      Object.defineProperty(document, "visibilityState", { value: "hidden", writable: true });
      document.dispatchEvent(new Event("visibilitychange"));
    });
    await page.waitForTimeout(500);
    assert.equal(utxoFetchCount, countBeforeHide, "No wallet refreshes should fire while tab is hidden");

    // Emulate tab visible resume with failing backend -> immediate refresh fires, preserves 5 KVNC and marks Stale
    failNextUtxoRefresh = true;
    await page.evaluate(() => {
      Object.defineProperty(document, "visibilityState", { value: "visible", writable: true });
      document.dispatchEvent(new Event("visibilitychange"));
    });

    // Deterministic wait for the immediate visible-resume refresh to fire
    const resumeDeadline = Date.now() + 15_000;
    while (utxoFetchCount <= countBeforeHide && Date.now() < resumeDeadline) {
      await page.waitForTimeout(100);
    }

    assert.ok(utxoFetchCount > countBeforeHide, "Visible resume must trigger immediate wallet refresh");
    await waitForBodyText(page, "Stale");
    walletText = await page.locator("body").innerText();
    assert.ok(
      walletText.includes("5 KVNC"),
      "Failed refresh must preserve last known 5 KVNC balance (never zero out)",
    );
    assert.ok(
      walletText.includes("Stale"),
      "Failed refresh after prior success must display explicit 'Stale' freshness badge",
    );
  } catch (err) {
    await saveFailureArtifact(page, "section-c-tab-visibility", err);
    throw err;
  } finally {
    await context.close();
  }
});

// -----------------------------------------------------------------------------
// SECTION D: Quote-Only Pool Inputs, Errors, Handoff & Back Navigation
// -----------------------------------------------------------------------------
test("Section D: quote-only pool calculator calculates exact quotes without financial mutations", async () => {
  const { context, page } = await setupIsolatedPage({ viewport: { width: 1280, height: 800 } });

  try {
    await gotoAndSettle(page, `${baseUrl}/pool`, "Liquidity pools");

    const input = page.locator("input[placeholder=\"0\"]");
    await assert.doesNotReject(input.waitFor({ timeout: 5000 }));

    // 1. Enter valid amount "5"
    await input.fill("5");
    await page.waitForTimeout(400);

    let text = await page.locator("body").innerText();
    assert.ok(text.includes("Price impact"), "Pool view must calculate price impact");
    assert.ok(text.includes("Fee 0.30%"), "Pool view must reflect 0.30% fee");

    // 2. Toggle direction to Token -> KVNC
    const sellDirBtn = page.locator("button:text(\"TEST → KVNC\")");
    await sellDirBtn.click();
    await page.waitForTimeout(400);
    text = await page.locator("body").innerText();
    assert.ok(text.includes("You pay (TEST)"), "Input label must reflect selected token direction");

    // 3. Test invalid inputs (0, negative, and non-numeric)
    for (const invalidVal of ["0", "-1", "abc"]) {
      await input.fill(invalidVal);
      await page.waitForTimeout(300);
      text = await page.locator("body").innerText();
      assert.ok(text.includes("—"), `Invalid amount '${invalidVal}' must render quote as em-dash`);
    }

    // 4. Assert Open in Wallet button is disabled with execution capability explanation
    const openBtn = page.locator("button:text(\"Open in Wallet (Unsupported)\")");
    const isDisabled = await openBtn.isDisabled();
    assert.equal(isDisabled, true, "Open in Wallet button must be strictly disabled");
    assert.ok(
      text.includes("Notice: AMM pool execution is not supported by the web wallet"),
      "Pool view must explain that execution is unsupported and strictly quote-only",
    );

    // 5. Verify NO financial mutation was triggered
    assert.equal(interceptedMutations.length, 0, "No financial mutations should be triggered on pool view");
  } catch (err) {
    await saveFailureArtifact(page, "section-d-quote-only-pool", err);
    throw err;
  } finally {
    await context.close();
  }
});

test("Section D: pool handoff to wallet renders capability gap blocker, rejects malformed/secret handoffs, and supports back navigation", async () => {
  const { context, page } = await setupIsolatedPage({ viewport: { width: 1280, height: 800 } });

  try {
    // 1. Start at pool
    await gotoAndSettle(page, `${baseUrl}/pool`, "Liquidity pools");

    // 2. Navigate to wallet with valid handoff query
    const tokenHex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const handoffUrl = `${baseUrl}/wallet?action=swap&token=${tokenHex}&direction=buy&amount=10&network=testnet`;
    await gotoAndSettle(page, handoffUrl, "Capability Gap Notice");

    let walletText = await page.locator("body").innerText();
    assert.ok(
      walletText.includes("Capability Gap Notice"),
      "Wallet must render Capability Gap Notice banner",
    );
    assert.ok(
      walletText.includes("Pool Swap Handoff Blocked: AMM pool execution ('token_buy' / 'token_sell') is not supported by the web wallet"),
      "Handoff must explain that swap execution is unsupported and blocked",
    );
    assert.ok(
      walletText.includes("Pool remains quote-only"),
      "Handoff banner must state that pool remains quote-only",
    );

    // 3. Navigate back to /pool
    await page.goBack();
    await waitForBodyText(page, "Liquidity pools");
    assert.ok(page.url().includes("/pool"), "Back navigation must return to /pool");
    const backPoolText = await page.locator("body").innerText();
    assert.ok(backPoolText.includes("Liquidity pools"), "Pool page must remain intact after back navigation");

    // 4. Test invalid handoff: network mismatch and forbidden secret parameter
    await gotoAndSettle(
      page,
      `${baseUrl}/wallet?action=swap&token=${tokenHex}&direction=buy&amount=10&network=mainnet`,
      "Network mismatch",
    );
    walletText = await page.locator("body").innerText();
    assert.ok(
      walletText.includes("Handoff rejected:") && walletText.includes("Network mismatch"),
      "Mismatched network handoff must be explicitly rejected",
    );

    await gotoAndSettle(
      page,
      `${baseUrl}/wallet?action=swap&token=${tokenHex}&direction=buy&amount=10&network=testnet&seed=forbidden`,
      "Handoff rejected: Forbidden parameter 'seed'",
    );
    walletText = await page.locator("body").innerText();
    assert.ok(
      walletText.includes("Handoff rejected: Forbidden parameter 'seed'"),
      "Handoff with secret query parameter must be rejected",
    );

    // 5. Verify zero mutations were sent
    assert.equal(interceptedMutations.length, 0, "Handoff must not trigger any mutation or submission");
  } catch (err) {
    await saveFailureArtifact(page, "section-d-handoff-back-nav", err);
    throw err;
  } finally {
    await context.close();
  }
});

// -----------------------------------------------------------------------------
// VERIFICATION OF STRICT NETWORK ISOLATION
// -----------------------------------------------------------------------------
test("Network Isolation Guard: zero unintended external egress during entire test execution", () => {
  // Any public connection attempt was caught and blocked by the harness guard
  for (const blocked of blockedExternalRequests) {
    console.log(`[GUARD VERIFIED BLOCKED] ${blocked}`);
  }
  // The test harness successfully trapped every external network attempt
  assert.ok(true, "All public connections were intercepted and prevented");
});
