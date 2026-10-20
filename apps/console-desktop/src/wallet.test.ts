import { describe, expect, it } from "vitest";

import htmlSource from "../index.html?raw";
import mainSource from "./main.ts?raw";
import {
  BALANCE_LABEL,
  EMPTY_HISTORY,
  HISTORY_HEADING,
  LOCK_AFTER_MS,
  MNEMONIC_WARNING,
  clearedImportFields,
  idleLocked,
  needsMainnetConfirm,
  phraseOnScreen,
  reviewRows,
  sendEnabled,
  type QuoteView,
} from "./wallet";
import walletSource from "./wallet.ts?raw";
import viewSource from "./wallet-view.ts?raw";

const quote: QuoteView = {
  recipient: "kvnc-from-rust",
  amountAtoms: "1000000000",
  amountDisplay: "1",
  feeAtoms: "42",
  feeDisplay: "0.000000042",
  chainId: "4",
  nonce: "7",
  feeRate: "1",
  draftRecipient: "typed-recipient",
  draftAmount: "1",
};

function enabled(overrides: Partial<Parameters<typeof sendEnabled>[0]> = {}): boolean {
  return sendEnabled({
    phase: "open",
    recipient: quote.draftRecipient,
    amount: quote.draftAmount,
    shownChainId: "4",
    quote,
    confirmMainnet: false,
    ...overrides,
  });
}

describe("wallet form", () => {
  it("lists the recipient, amount, fee, chain, and nonce before sending", () => {
    expect(reviewRows(quote)).toEqual([
      { label: "Recipient", value: "kvnc-from-rust" },
      { label: "Amount", value: "1 KUNA (1000000000 atoms)" },
      { label: "Fee", value: "0.000000042 KUNA (42 atoms)" },
      { label: "Chain id", value: "4" },
      { label: "Nonce", value: "7" },
    ]);
  });

  it("sends only when the quote matches the form and the chain", () => {
    expect(enabled()).toBe(true);
    expect(enabled({ phase: "locked" })).toBe(false);
    expect(enabled({ quote: null })).toBe(false);
    expect(enabled({ recipient: "other" })).toBe(false);
    expect(enabled({ amount: "2" })).toBe(false);
    expect(enabled({ shownChainId: null })).toBe(false);
    expect(enabled({ shownChainId: "1" })).toBe(false);
    const mainnet = { ...quote, chainId: "1" };
    expect(enabled({ quote: mainnet, shownChainId: "1", confirmMainnet: false })).toBe(false);
    expect(enabled({ quote: mainnet, shownChainId: "1", confirmMainnet: true })).toBe(true);
    expect(needsMainnetConfirm("1")).toBe(true);
    expect(needsMainnetConfirm("4")).toBe(false);
  });

  it("clears the recovery phrase from the screen when the session locks", () => {
    expect(phraseOnScreen("open", "one two three")).toBe("one two three");
    expect(phraseOnScreen("locked", "one two three")).toBe("");
    expect(phraseOnScreen("absent", "one two three")).toBe("");
    expect(viewSource.includes("phrase = phraseOnScreen(phase, phrase)")).toBe(true);
    expect(viewSource.includes("mnemonicText.textContent = phrase")).toBe(true);
  });

  it("locks after five minutes and does not offer to copy the phrase", () => {
    expect(LOCK_AFTER_MS).toBe(300_000);
    expect(idleLocked(0, 299_999)).toBe(false);
    expect(idleLocked(0, 300_000)).toBe(true);
    expect(MNEMONIC_WARNING).toContain("Write it down");
    expect(MNEMONIC_WARNING.includes("clipboard")).toBe(false);
    for (const text of [walletSource, viewSource, htmlSource]) {
      expect(text.includes("copy-mnemonic")).toBe(false);
      expect(text.includes("clipboard")).toBe(false);
      expect(text.includes("copyMnemonic")).toBe(false);
    }
  });

  it("clears import fields after a failed import", () => {
    expect(clearedImportFields()).toEqual({ phrase: "", password: "" });
    const handler = viewSource.slice(viewSource.indexOf("wallet_import"));
    const finallyAt = handler.indexOf("finally");
    expect(finallyAt).toBeGreaterThan(0);
    expect(handler.indexOf("clearedImportFields()")).toBeGreaterThan(finallyAt);
    expect(handler.includes('importPhrase.value = ""')).toBe(false);
    expect(handler.includes('importPassword.value = ""')).toBe(false);
  });

  it("labels unproven figures and does not keep secrets in the page", () => {
    expect(HISTORY_HEADING).toBe("History (reported by node)");
    expect(BALANCE_LABEL).toBe("Balance (reported by node)");
    expect(EMPTY_HISTORY).toBe("No transfers reported by the node.");
    for (const text of [walletSource, viewSource, mainSource, htmlSource]) {
      expect(text.includes("localStorage")).toBe(false);
      expect(text.includes("innerHTML")).toBe(false);
    }
  });
});
