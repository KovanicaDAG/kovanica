/**
 * Wallet screen rules. Keys, addresses, and atom amounts stay in Rust.
 * This module only decides what the form may show.
 */

export const LOCK_AFTER_MS = 5 * 60 * 1000;

/** Tauri event emitted when Rust drops the open session. */
export const WALLET_LOCKED_EVENT = "wallet-locked";

/** Tauri event emitted before the idle lock, while the session is still open. */
export const WALLET_LOCK_WARNING_EVENT = "wallet-lock-warning";

export const LOCK_WARNING =
  "This wallet will lock soon. The recovery phrase will leave this screen. Finish writing it down. Click this screen if you are still here.";

export const MNEMONIC_WARNING =
  "Anyone who reads this phrase can spend the KUNA. Write it down. This screen does not copy it.";

export const HISTORY_HEADING = "History (reported by node)";

export const BALANCE_LABEL = "Balance (reported by node)";

export const EMPTY_HISTORY = "No transfers reported by the node.";

export interface QuoteView {
  recipient: string;
  amountAtoms: string;
  amountDisplay: string;
  feeAtoms: string;
  feeDisplay: string;
  chainId: string;
  nonce: string;
  feeRate: string;
  draftRecipient: string;
  draftAmount: string;
}

export interface ReviewRow {
  label: string;
  value: string;
}

/** True when the session should be treated as idle. */
export function idleLocked(lastActiveMs: number, nowMs: number): boolean {
  return nowMs - lastActiveMs >= LOCK_AFTER_MS;
}

/** Phrase and password values after a lock. The screen applies these at once. */
export function secretsCleared(): { phrase: string; password: string } {
  return { phrase: "", password: "" };
}

/** Import fields are wiped after the attempt, including a failed one. */
export function clearedImportFields(): { phrase: string; password: string } {
  return { phrase: "", password: "" };
}

/** Warning text while the phrase is still visible. An empty phrase shows nothing. */
export function phraseLockWarning(phrase: string, warned: boolean): string {
  if (phrase === "" || !warned) {
    return "";
  }
  return LOCK_WARNING;
}

/** The phrase stays on screen only while the session is still open. */
export function phraseOnScreen(phase: string, phrase: string): string {
  if (phase !== "open") {
    return "";
  }
  return phrase;
}

/** Review needs a recipient and an amount. Rust parses both. */
export function reviewEnabled(phase: string, recipient: string, amount: string): boolean {
  return phase === "open" && recipient.trim() !== "" && amount.trim() !== "";
}

/** Send stays off until the quote still matches the form and the chain. */
export function sendEnabled(input: {
  phase: string;
  recipient: string;
  amount: string;
  shownChainId: string | null;
  quote: QuoteView | null;
  confirmMainnet: boolean;
}): boolean {
  const quote = input.quote;
  if (input.phase !== "open" || quote === null) {
    return false;
  }
  if (quote.draftRecipient !== input.recipient || quote.draftAmount !== input.amount) {
    return false;
  }
  if (input.shownChainId === null || quote.chainId !== input.shownChainId) {
    return false;
  }
  if (quote.chainId === "1" && !input.confirmMainnet) {
    return false;
  }
  return true;
}

/** Rows shown before the user signs. */
export function reviewRows(quote: QuoteView): ReviewRow[] {
  return [
    { label: "Recipient", value: quote.recipient },
    {
      label: "Amount",
      value: `${quote.amountDisplay} KUNA (${quote.amountAtoms} atoms)`,
    },
    { label: "Fee", value: `${quote.feeDisplay} KUNA (${quote.feeAtoms} atoms)` },
    { label: "Chain id", value: quote.chainId },
    { label: "Nonce", value: quote.nonce },
  ];
}

/** Chain 1 is the only chain that needs the extra checkbox. */
export function needsMainnetConfirm(chainId: string): boolean {
  return chainId === "1";
}
