import { test } from "node:test";
import assert from "node:assert/strict";
import { validatePoolHandoff } from "../src/lib/wallet/handoff";

const VALID_TOKEN_HEX = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
// Pinned PRO-21 canonical vector from tests/fixtures/vectors.json
const VALID_PRO21_ADDR = "kvnc8d4z00xwk6jz6c4r4rgz5mcdwdjny9thrh3y8f36cpy2rz6emg54rufktdag";

test("PoolHandoff: valid parameters return ok with execution capability gap", () => {
  const params = {
    action: "swap",
    token: VALID_TOKEN_HEX,
    direction: "buy",
    amount: "10.5",
    network: "testnet",
  };

  const result = validatePoolHandoff(params, "testnet");
  assert.equal(result.ok, true);
  if (result.ok) {
    assert.equal(result.params.token, VALID_TOKEN_HEX);
    assert.equal(result.params.direction, "buy");
    assert.equal(result.params.amount, "10.5");
    assert.equal(result.params.network, "testnet");
    assert.equal(result.isSupported, false, "Must report execution capability gap");
    assert.ok(result.capabilityGap.includes("not implemented"));
  }
});

test("PoolHandoff: accepts canonical PRO-21 address format", () => {
  const params = {
    token: VALID_PRO21_ADDR,
    direction: "sell",
    amount: "1.0",
    network: "testnet",
  };

  const result = validatePoolHandoff(params, "testnet");
  assert.equal(result.ok, true);
  if (result.ok) {
    assert.equal(result.params.token, VALID_PRO21_ADDR);
    assert.equal(result.params.direction, "sell");
    assert.equal(result.isSupported, false);
  }
});

test("PoolHandoff: rejects missing or invalid token identifier", () => {
  const missing = validatePoolHandoff({ direction: "buy", amount: "5" }, "testnet");
  assert.equal(missing.ok, false);
  if (!missing.ok) assert.equal(missing.reason, "MISSING_TOKEN");

  const invalid = validatePoolHandoff(
    { token: "not-a-valid-token-format", direction: "buy", amount: "5" },
    "testnet",
  );
  assert.equal(invalid.ok, false);
  if (!invalid.ok) assert.equal(invalid.reason, "INVALID_TOKEN");
});

test("PoolHandoff: rejects invalid swap direction", () => {
  const result = validatePoolHandoff(
    { token: VALID_TOKEN_HEX, direction: "transfer", amount: "10" },
    "testnet",
  );
  assert.equal(result.ok, false);
  if (!result.ok) assert.equal(result.reason, "INVALID_DIRECTION");
});

test("PoolHandoff: rejects invalid or out-of-bounds amount", () => {
  const zero = validatePoolHandoff(
    { token: VALID_TOKEN_HEX, direction: "buy", amount: "0" },
    "testnet",
  );
  assert.equal(zero.ok, false);
  if (!zero.ok) assert.equal(zero.reason, "INVALID_AMOUNT");

  const negative = validatePoolHandoff(
    { token: VALID_TOKEN_HEX, direction: "buy", amount: "-5.0" },
    "testnet",
  );
  assert.equal(negative.ok, false);
  if (!negative.ok) assert.equal(negative.reason, "INVALID_AMOUNT");

  const nanAmount = validatePoolHandoff(
    { token: VALID_TOKEN_HEX, direction: "buy", amount: "not_a_number" },
    "testnet",
  );
  assert.equal(nanAmount.ok, false);
  if (!nanAmount.ok) assert.equal(nanAmount.reason, "INVALID_AMOUNT");

  const overflow = validatePoolHandoff(
    { token: VALID_TOKEN_HEX, direction: "buy", amount: "99999999999" },
    "testnet",
  );
  assert.equal(overflow.ok, false);
  if (!overflow.ok) assert.equal(overflow.reason, "INVALID_AMOUNT");
});

test("PoolHandoff: rejects network mismatch to prevent silent switching", () => {
  const result = validatePoolHandoff(
    { token: VALID_TOKEN_HEX, direction: "buy", amount: "1.0", network: "mainnet" },
    "testnet",
  );
  assert.equal(result.ok, false);
  if (!result.ok) {
    assert.equal(result.reason, "NETWORK_MISMATCH");
    assert.ok(result.message.includes("Silent network switching is forbidden"));
  }
});

test("PoolHandoff: rejects forbidden secret parameters in URL", () => {
  const withMnemonic = validatePoolHandoff(
    {
      token: VALID_TOKEN_HEX,
      direction: "buy",
      amount: "1.0",
      mnemonic: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    },
    "testnet",
  );
  assert.equal(withMnemonic.ok, false);
  if (!withMnemonic.ok) {
    assert.equal(withMnemonic.reason, "FORBIDDEN_SECRET");
    assert.ok(withMnemonic.message.includes("Secrets must never be passed via URL"));
  }

  const withKey = validatePoolHandoff(
    {
      token: VALID_TOKEN_HEX,
      direction: "buy",
      amount: "1.0",
      privateKey: "0123456789abcdef",
    },
    "testnet",
  );
  assert.equal(withKey.ok, false);
  if (!withKey.ok) assert.equal(withKey.reason, "FORBIDDEN_SECRET");
});

test("PoolHandoff: URLSearchParams support works seamlessly", () => {
  const qs = new URLSearchParams();
  qs.set("token", VALID_TOKEN_HEX);
  qs.set("direction", "buy");
  qs.set("amount", "2.5");
  qs.set("network", "testnet");

  const result = validatePoolHandoff(qs, "testnet");
  assert.equal(result.ok, true);
  if (result.ok) {
    assert.equal(result.params.amount, "2.5");
    assert.equal(result.isSupported, false);
  }
});
