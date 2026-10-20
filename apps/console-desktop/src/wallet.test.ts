import { describe, expect, it } from "vitest";

import htmlSource from "../index.html?raw";
import mainSource from "./main.ts?raw";
import {
  BALANCE_LABEL,
  EMPTY_HISTORY,
  HISTORY_HEADING,
  LOCK_AFTER_MS,
  LOCK_WARNING,
  MNEMONIC_WARNING,
  WALLET_LOCK_WARNING_EVENT,
  WALLET_LOCKED_EVENT,
  clearedImportFields,
  idleLocked,
  needsMainnetConfirm,
  phraseLockWarning,
  phraseOnScreen,
  reviewRows,
  secretsCleared,
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

  it("warns before the phrase leaves the screen", () => {
    expect(WALLET_LOCK_WARNING_EVENT).toBe("wallet-lock-warning");
    expect(phraseLockWarning("one two three", true)).toBe(LOCK_WARNING);
    expect(phraseLockWarning("one two three", false)).toBe("");
    expect(phraseLockWarning("", true)).toBe("");
    expect(LOCK_WARNING.includes("recovery phrase")).toBe(true);
    expect(htmlSource.includes('id="mnemonic-lock-warning"')).toBe(true);
    const warnAt = viewSource.indexOf("listen(WALLET_LOCK_WARNING_EVENT");
    const lockAt = viewSource.indexOf("listen(WALLET_LOCKED_EVENT");
    expect(warnAt).toBeGreaterThan(0);
    expect(lockAt).toBeGreaterThan(warnAt);
    const handler = viewSource.slice(warnAt, lockAt);
    expect(handler.includes("paintLockWarning(true)")).toBe(true);
    expect(handler.includes('mnemonicText.textContent = ""')).toBe(false);
    expect(handler.includes('phrase = ""')).toBe(false);
    const activity = viewSource.slice(
      viewSource.indexOf("const noteActivity"),
      viewSource.indexOf("document.addEventListener"),
    );
    expect(activity.includes("mnemonicLockWarning.hidden")).toBe(true);
    expect(activity.includes("wallet_touch")).toBe(true);
    expect(activity.includes("paintLockWarning(false)")).toBe(true);
  });

  it("clears secrets when the lock event arrives, before the next status poll", () => {
    expect(WALLET_LOCKED_EVENT).toBe("wallet-locked");
    expect(secretsCleared()).toEqual({ phrase: "", password: "" });
    const listenAt = viewSource.indexOf("listen(WALLET_LOCKED_EVENT");
    expect(listenAt).toBeGreaterThan(0);
    const handler = viewSource.slice(listenAt, viewSource.indexOf("setInterval"));
    const clearAt = handler.indexOf("clearSecretFields()");
    const refreshAt = handler.indexOf("refreshStatus");
    expect(clearAt).toBeGreaterThan(0);
    expect(refreshAt).toBeGreaterThan(clearAt);
    const clearer = viewSource.slice(viewSource.indexOf("function clearSecretFields"), listenAt);
    expect(clearer.includes("secretsCleared()")).toBe(true);
    expect(clearer.includes("mnemonicText.textContent = cleared.phrase")).toBe(true);
    expect(clearer.includes("importPhrase.value = cleared.phrase")).toBe(true);
    expect(clearer.includes("newPassword.value = cleared.password")).toBe(true);
    expect(clearer.includes("unlockPassword.value = cleared.password")).toBe(true);
    expect(clearer.includes("mnemonicPanel.hidden = true")).toBe(true);
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
