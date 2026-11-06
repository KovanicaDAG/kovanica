/**
 * Pool -> Wallet handoff parameter validation and capability gating.
 *
 * Rules:
 * - Treat all URL/deep-link parameters as untrusted.
 * - Validate types, token identifiers, units, amount bounds and chain identity.
 * - Strictly forbid secrets in URLs.
 * - Prefill review screen only (when supported); never sign, submit, unlock or switch networks silently.
 * - If the wallet lacks the execution flow, do NOT fabricate swap support or build an unrelated
 *   native transfer. Keep action disabled with a clear explanation and report capability gap as blocker.
 */
import { isAddr, parseAddr } from "@/lib/wallet/address";

export type SwapDirection = "buy" | "sell";

export interface PoolHandoffParams {
  action?: string;
  network?: string;
  token?: string;
  direction?: SwapDirection;
  amount?: string;
}

export interface ValidationSuccess {
  ok: true;
  params: {
    token: string;
    direction: SwapDirection;
    amount: string;
    network: string;
  };
  isSupported: false;
  capabilityGap: string;
}

export interface ValidationError {
  ok: false;
  reason:
    | "MISSING_TOKEN"
    | "INVALID_TOKEN"
    | "INVALID_DIRECTION"
    | "INVALID_AMOUNT"
    | "NETWORK_MISMATCH"
    | "FORBIDDEN_SECRET"
    | "UNKNOWN_ACTION";
  message: string;
}

export type HandoffValidationResult = ValidationSuccess | ValidationError;

const MAX_KVNC_AMOUNT = 90_200_000; // Total supply bound

/**
 * Validates untrusted query/deep-link parameters for Pool -> Wallet handoff.
 */
export function validatePoolHandoff(
  input: Record<string, unknown> | URLSearchParams | null | undefined,
  currentNetwork: string,
): HandoffValidationResult {
  if (!input) {
    return {
      ok: false,
      reason: "MISSING_TOKEN",
      message: "No handoff parameters provided",
    };
  }

  const get = (key: string): string | null => {
    if (input instanceof URLSearchParams) {
      return input.get(key);
    }
    const val = input[key];
    return typeof val === "string" ? val : null;
  };

  // Reject forbidden secret parameters
  for (const forbidden of ["mnemonic", "seed", "privateKey", "secret", "password", "key"]) {
    if (get(forbidden) !== null) {
      return {
        ok: false,
        reason: "FORBIDDEN_SECRET",
        message: `Forbidden parameter '${forbidden}' detected in URL parameters. Secrets must never be passed via URL.`,
      };
    }
  }

  const action = get("action");
  if (action && action !== "swap" && action !== "pool") {
    return {
      ok: false,
      reason: "UNKNOWN_ACTION",
      message: `Unsupported handoff action: '${action}'.`,
    };
  }

  // Network verification
  const network = get("network");
  if (network) {
    const normReq = network.trim().toLowerCase();
    const normCur = currentNetwork.trim().toLowerCase();
    if (normReq !== normCur) {
      return {
        ok: false,
        reason: "NETWORK_MISMATCH",
        message: `Network mismatch: requested network '${network}' does not match active network '${currentNetwork}'. Silent network switching is forbidden.`,
      };
    }
  }

  // Token verification
  const token = get("token");
  if (!token || typeof token !== "string" || !token.trim()) {
    return {
      ok: false,
      reason: "MISSING_TOKEN",
      message: "Missing token parameter in handoff request.",
    };
  }
  const cleanToken = token.trim();
  const is64Hex = /^[0-9a-f]{64}$/i.test(cleanToken);
  const isValidAddress = isAddr(cleanToken) || parseAddr(cleanToken) !== null;
  if (!is64Hex && !isValidAddress) {
    return {
      ok: false,
      reason: "INVALID_TOKEN",
      message: "Invalid token identifier: must be a 64-hex contract address or canonical kvnc...dag address.",
    };
  }

  // Direction verification
  const directionRaw = get("direction")?.toLowerCase();
  if (directionRaw !== "buy" && directionRaw !== "sell") {
    return {
      ok: false,
      reason: "INVALID_DIRECTION",
      message: "Direction must be either 'buy' or 'sell'.",
    };
  }
  const direction: SwapDirection = directionRaw;

  // Amount verification
  const amountStr = get("amount");
  if (!amountStr || typeof amountStr !== "string") {
    return {
      ok: false,
      reason: "INVALID_AMOUNT",
      message: "Missing amount parameter.",
    };
  }
  const parsedNum = Number(amountStr);
  if (!Number.isFinite(parsedNum) || parsedNum <= 0 || parsedNum > MAX_KVNC_AMOUNT) {
    return {
      ok: false,
      reason: "INVALID_AMOUNT",
      message: `Invalid amount: must be a positive finite number <= ${MAX_KVNC_AMOUNT}.`,
    };
  }

  // Check execution capability
  // The wallet does NOT support token_buy / token_sell AMM contract execution.
  // Explicitly return isSupported = false with the blocker capability gap.
  return {
    ok: true,
    params: {
      token: cleanToken,
      direction,
      amount: amountStr,
      network: network ?? currentNetwork,
    },
    isSupported: false,
    capabilityGap:
      "Wallet execution gap: AMM pool swap calls ('token_buy' / 'token_sell') are not implemented in the wallet or RPC dispatch. Pool remains quote-only.",
  };
}
