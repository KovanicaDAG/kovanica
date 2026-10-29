import { useEffect, useState } from "react";
import { Copy, Eye, EyeOff, Gem, Lock, Unlock, Usb } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { AddressQr } from "@/components/wallet/address-qr";
import { AssetPicker, type AssetOption } from "@/components/wallet/asset-picker";
import { ConnectHardwareModal } from "@/components/wallet/connect-hardware-modal";
import { HardwareSignModal } from "@/components/wallet/hardware-sign-modal";
import { api, useApiSource, isPublic } from "@/lib/api/client";
import { MIN_FEE, TOKEN, isNativeAsset, assetLabel } from "@/lib/api/contract";
import type { ApiHistory, ApiUtxos } from "@/lib/api/contract";
import type { ApiToken } from "@/lib/tokens/codec";
import { formatTokenAmount, parseTokenAmount } from "@/lib/tokens/codec";
import { prepareUrl, submitUrl, assetOptionsFromUtxos, balanceForAsset } from "@/lib/api/assets";
import {
  TOKEN_SEND_FEE,
  tokenPrepareUrl,
  tokenSubmitUrl,
} from "@/lib/api/token-send";
import { ATOM } from "@/lib/ledger/types";
import type { HardwareWalletRec, SoftwareWalletRec, WalletRec } from "@/lib/ledger/types";
import { fmtKvnc, parseKvnc } from "@/lib/ledger/format";
import { shortId } from "@/lib/ledger/hash";
import { useLedger } from "@/lib/ledger/store";
import {
  addressFromMnemonic,
  createMnemonic,
  importMnemonic,
  signSighash,
} from "@/lib/wallet/keys";
import { hexToKvnc, parseAddr } from "@/lib/wallet/address";
import { encryptMnemonic, decryptMnemonic } from "@/lib/wallet/vault";
import { disconnectActiveHardwareProvider } from "@/lib/wallet/hardware";
import { useHydrated } from "@/lib/use-hydrated";

function isLocked(w: WalletRec | null): w is SoftwareWalletRec & {
  encryptedMnemonic: NonNullable<SoftwareWalletRec["encryptedMnemonic"]>;
} {
  if (!w) return false;
  if (w.type === "hardware") return false;
  if (w.kind === "watch") return false;
  return !!w.encryptedMnemonic && !w.mnemonic;
}

function isPlaintext(w: WalletRec | null): w is SoftwareWalletRec & { mnemonic: string } {
  if (!w) return false;
  if (w.type === "hardware") return false;
  if (w.kind === "watch") return false;
  return !!w.mnemonic && !w.encryptedMnemonic;
}

export function WalletView() {
  const hydrated = useHydrated();
  const source = useApiSource();
  const live = isPublic(source);
  const walletStore = useLedger((s) => s.wallet);
  const wallet = hydrated ? walletStore : null;
  const setWallet = useLedger((s) => s.setWallet);
  const [busy, setBusy] = useState(false);
  const [phrase, setPhrase] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [to, setTo] = useState("");
  const [amount, setAmount] = useState("1");
  const [utxos, setUtxos] = useState<ApiUtxos | null>(null);
  const [hist, setHist] = useState<ApiHistory | null>(null);
  const [feeRates, setFeeRates] = useState<{ slow: number; normal: number; fast: number } | null>(
    null,
  );
  const [feeTier, setFeeTier] = useState<"slow" | "normal" | "fast">("normal");
  /** RFC-002: null = native KUNA */
  const [assetId, setAssetId] = useState<string | null>(null);
  /** KVP-102: contract address of the token selected in the send form. */
  const [tokenAddr, setTokenAddr] = useState<string | null>(null);
  /** KVP-102 tokens discovered on chain (all of them, not only ones this wallet holds). */
  const [tokens, setTokens] = useState<ApiToken[]>([]);

  // M-05: recovery phrase generation is two-phase — show the words, then
  // require an explicit "I wrote it down" confirmation before wallet creation.
  const [pendingMnemonic, setPendingMnemonic] = useState<string | null>(null);
  const [wroteItDown, setWroteItDown] = useState(false);

  const [showConnectModal, setShowConnectModal] = useState(false);
  const [signModalState, setSignModalState] = useState<{
    open: boolean;
    sighash: string;
    dest: string;
    atoms: number;
    amountKvnc: string;
    feeKvnc: string;
    /** Set for a KVP-102 token call; `atoms` is the u128 base-unit amount. */
    token?: { address: string; symbol: string; decimals: number; atoms: string };
  } | null>(null);

  const balance = utxos?.balance ?? 0;
  const assetBalance = utxos ? balanceForAsset(utxos.utxos, assetId) : 0;
  const selectedToken = tokenAddr
    ? (tokens.find((t) => t.address.toLowerCase() === tokenAddr.toLowerCase()) ?? null)
    : null;
  const tokenBalance = selectedToken ? BigInt(selectedToken.balance ?? "0") : 0n;
  const spendable = selectedToken
    ? Number(formatTokenAmount(tokenBalance, selectedToken.decimals))
    : isNativeAsset(assetId)
      ? balance
      : assetBalance;
  const fee = feeRates ? feeRates[feeTier] : MIN_FEE;

  // UTXO assets (RFC-002) + every KVP-102 contract token discovered on chain.
  // A contract token is not a UTXO, so it only shows up here because the token
  // scan found it — not because the wallet's UTXO set mentions it.
  const assetOptions: AssetOption[] = [
    ...assetOptionsFromUtxos(utxos?.utxos ?? []),
    ...tokens.map((t) => ({
      assetId: `token:${t.address}`,
      balance: Number(formatTokenAmount(BigInt(t.balance ?? "0"), t.decimals)),
      label: t.symbol,
      symbol: t.symbol,
      icon: t.icon,
    })),
  ];
  const pickerValue = selectedToken ? `token:${selectedToken.address}` : assetId;

  function onPickAsset(id: string | null) {
    if (id && id.startsWith("token:")) {
      setTokenAddr(id.slice("token:".length));
      setAssetId(null);
      return;
    }
    setTokenAddr(null);
    setAssetId(id);
  }

  // KVP-106: Check if selected asset is an NFT
  const selectedAssetOpt = assetOptions.find(
    (opt) =>
      (isNativeAsset(assetId) && isNativeAsset(opt.assetId)) ||
      (!isNativeAsset(assetId) && opt.assetId === assetId),
  );
  const isNftSelected = !selectedToken && selectedAssetOpt?.kind === "nft";

  async function refreshChain(address: string) {
    try {
      const [u, h, f] = await Promise.all([
        api<ApiUtxos>(`/api/utxos?address=${address}`),
        api<ApiHistory>(`/api/history?address=${address}`),
        api<{ fee_rate: number; unit: string; mempool: number; bytes: number }>(
          "/api/fee_estimate",
          "GET",
        ),
      ]);
      setUtxos(u);
      setHist(h);
      if (typeof f.fee_rate === "number") {
        const rate = f.fee_rate;
        setFeeRates({ slow: rate, normal: rate, fast: rate });
      }
      // Token discovery is a log scan; never let it hold up the balance.
      void api<ApiToken[]>(`/api/tokens?address=${address}`)
        .then(setTokens)
        .catch(() => setTokens([]));
    } catch {
      /* keep last */
    }
  }

  // One phrase = one wallet. Re-derive the address from the stored phrase and
  // repair wallets created by the old SLIP-0010 build (which showed three
  // "accounts" per phrase, each with a different address).
  useEffect(() => {
    if (!walletStore) return;
    if (walletStore.type === "hardware") return;
    if (walletStore.kind === "watch") return;
    if (!walletStore.mnemonic) return;
    void addressFromMnemonic(walletStore.mnemonic).then((address) => {
      if (address !== walletStore.address || walletStore.index !== 0) {
        setWallet({ ...walletStore, address, index: 0 });
      }
    });
  }, [walletStore, setWallet]);

  useEffect(() => {
    if (!wallet) {
      setUtxos(null);
      setHist(null);
      setTokens([]);
      return;
    }
    void (async () => {
      await refreshChain(wallet.address);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wallet?.address, source]);

  // M-05: phase 1 — generate the recovery phrase and show it for backup.
  async function onCreate() {
    if (!password) {
      toast.error("Choose a password to encrypt the seed");
      return;
    }
    setBusy(true);
    try {
      const mnemonic = await createMnemonic();
      setPendingMnemonic(mnemonic);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Could not generate recovery phrase");
    } finally {
      setBusy(false);
    }
  }

  // M-05: phase 2 — only after the user confirms they wrote the words down.
  async function onConfirmBackup() {
    if (!pendingMnemonic) return;
    if (!password) {
      toast.error("Choose a password to encrypt the seed");
      return;
    }
    setBusy(true);
    try {
      const address = await addressFromMnemonic(pendingMnemonic, 0);
      const encryptedMnemonic = await encryptMnemonic(pendingMnemonic, password);
      setWallet({ encryptedMnemonic, address, index: 0, shown: false, kind: "local" });
      setPassword("");
      setPendingMnemonic(null);
      setWroteItDown(false);
      toast.success("Encrypted wallet created — keep your recovery words offline");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Could not create wallet");
    } finally {
      setBusy(false);
    }
  }

  function onCancelPending() {
    setPendingMnemonic(null);
    setWroteItDown(false);
  }

  async function onImport(e: React.FormEvent) {
    e.preventDefault();
    if (!password) {
      toast.error("Choose a password to encrypt the seed");
      return;
    }
    setBusy(true);
    try {
      if (
        phrase.trim().startsWith("kvnc1") ||
        /^[0-9a-f]{64}$/i.test(phrase.trim()) ||
        /^[0-9a-f]{66}$/i.test(phrase.trim())
      ) {
        const dest = parseAddr(phrase.trim());
        if (!dest) throw new Error("Invalid watch-only address");
        setWallet({ address: dest, index: 0, shown: false, kind: "watch" });
        setPhrase("");
        setPassword("");
        toast.success("Watch-only wallet connected");
      } else {
        const mnemonic = await importMnemonic(phrase);
        const address = await addressFromMnemonic(mnemonic, 0);
        const encryptedMnemonic = await encryptMnemonic(mnemonic, password);
        setWallet({ encryptedMnemonic, address, index: 0, shown: false, kind: "local" });
        setPhrase("");
        setPassword("");
        toast.success("Imported and encrypted");
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Import failed");
    } finally {
      setBusy(false);
    }
  }

  async function onUnlock(e: React.FormEvent) {
    e.preventDefault();
    if (!wallet || !isLocked(wallet)) return;
    setBusy(true);
    try {
      const mnemonic = await decryptMnemonic(wallet.encryptedMnemonic, password);
      setWallet({ ...wallet, mnemonic });
      setPassword("");
      toast.success("Wallet unlocked");
    } catch {
      toast.error("Wrong password");
    } finally {
      setBusy(false);
    }
  }

  function onLock() {
    if (!wallet || wallet.type === "hardware" || wallet.kind === "watch") return;
    setWallet({ ...wallet, mnemonic: undefined });
    toast.message("Wallet locked");
  }

  async function onSecure() {
    if (!wallet || !isPlaintext(wallet)) return;
    if (!password) {
      toast.error("Choose a password to encrypt the seed");
      return;
    }
    setBusy(true);
    try {
      const encryptedMnemonic = await encryptMnemonic(wallet.mnemonic, password);
      setWallet({ ...wallet, mnemonic: undefined, encryptedMnemonic });
      setPassword("");
      toast.success("Seed encrypted — wallet will lock on reload");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Encryption failed");
    } finally {
      setBusy(false);
    }
  }

  async function onCopy() {
    if (!wallet) return;
    await navigator.clipboard.writeText(hexToKvnc(wallet.address));
    toast.success("Address copied");
  }

  function onDownload() {
    if (!wallet || wallet.type === "hardware") return;
    const blob = new Blob([`${wallet.mnemonic}\n`], { type: "text/plain" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = "kovanica-seed.txt";
    a.click();
    toast.message("Seed file downloaded — keep it offline");
  }

  async function onFaucet() {
    if (!wallet || live) return;
    try {
      await api(`/api/faucet?to=${wallet.address}&amount=${ATOM}&kind=faucet`, "POST");
      await refreshChain(wallet.address);
      toast.success("Faucet paid 1 KUNA");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Faucet failed");
    }
  }

  async function onSend(e: React.FormEvent) {
    e.preventDefault();
    if (!wallet) return;
    if (selectedToken) {
      await onSendToken(selectedToken);
      return;
    }
    const atoms = parseKvnc(amount);
    if (atoms === null) {
      toast.error("Enter a positive amount");
      return;
    }
    const dest = parseAddr(to);
    if (!dest) {
      toast.error("Need a kvnc…dag or 64-hex address");
      return;
    }
    const need = isNativeAsset(assetId) ? atoms + fee : atoms;
    if (spendable > 0 && need > spendable) {
      const maxSend = Math.max(0, spendable - (isNativeAsset(assetId) ? fee : 0)) / ATOM;
      toast.error(
        `Amount exceeds ${isNativeAsset(assetId) ? TOKEN : "asset"} balance. Send at most ${maxSend}.`,
      );
      return;
    }
    if (!isNativeAsset(assetId) && balance < fee) {
      toast.error(`Need at least ${fmtKvnc(fee)} ${TOKEN} for fee`);
      return;
    }
    setBusy(true);
    try {
      const prep = await api<{ sighash?: string; ok?: boolean }>(
        prepareUrl(wallet.address, dest, atoms, assetId),
        "POST",
      );
      if (!prep || !prep.sighash) {
        toast.error(typeof prep === "string" ? prep : "Failed to prepare transaction");
        setBusy(false);
        return;
      }
      if (wallet.type === "hardware") {
        setSignModalState({
          open: true,
          sighash: prep.sighash,
          dest,
          atoms,
          amountKvnc: amount,
          feeKvnc: fmtKvnc(fee),
        });
        setBusy(false);
        return;
      }
      if (!wallet.mnemonic) {
        toast.error("Wallet is locked or watch-only");
        setBusy(false);
        return;
      }
      const sig = await signSighash(wallet.mnemonic, wallet.index, prep.sighash);
      await submitTransaction(dest, atoms, sig);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Send failed");
      setBusy(false);
    }
  }

  async function submitTransaction(dest: string, atoms: number, sig: string) {
    if (!wallet) return;
    setBusy(true);
    try {
      const sub = await api<{ tx: string }>(
        submitUrl(wallet.address, dest, atoms, sig, assetId),
        "POST",
      );
      try {
        await api("/api/produce", "POST");
      } catch {
        /* ignore */
      }
      await refreshChain(wallet.address);
      toast.success(`Sent · ${shortId(sub.tx)}`);
      setTo("");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Send failed");
    } finally {
      setBusy(false);
    }
  }

  async function onSendToken(token: ApiToken) {
    if (!wallet) return;
    const atoms = parseTokenAmount(amount, token.decimals);
    if (atoms === null) {
      toast.error(`Enter a positive ${token.symbol} amount (max ${token.decimals} decimals)`);
      return;
    }
    const dest = parseAddr(to);
    if (!dest) {
      toast.error("Need a kvnc…dag or 64-hex address");
      return;
    }
    const held = BigInt(token.balance ?? "0");
    if (held < atoms) {
      toast.error(
        `Insufficient ${token.symbol}: balance ${formatTokenAmount(held, token.decimals)}`,
      );
      return;
    }
    if (balance < TOKEN_SEND_FEE) {
      toast.error(`Need at least ${fmtKvnc(TOKEN_SEND_FEE)} ${TOKEN} for the fee`);
      return;
    }
    setBusy(true);
    try {
      const prep = await api<{ sighash?: string; fee?: number }>(
        tokenPrepareUrl(wallet.address, token.address, dest, atoms),
        "POST",
      );
      if (!prep || !prep.sighash) {
        toast.error(typeof prep === "string" ? prep : "Failed to prepare token transfer");
        setBusy(false);
        return;
      }
      if (wallet.type === "hardware") {
        setSignModalState({
          open: true,
          sighash: prep.sighash,
          dest,
          atoms: 0,
          amountKvnc: `${formatTokenAmount(atoms, token.decimals)} ${token.symbol}`,
          feeKvnc: fmtKvnc(prep.fee ?? TOKEN_SEND_FEE),
          token: {
            address: token.address,
            symbol: token.symbol,
            decimals: token.decimals,
            atoms: atoms.toString(),
          },
        });
        setBusy(false);
        return;
      }
      if (!wallet.mnemonic) {
        toast.error("Wallet is locked or watch-only");
        setBusy(false);
        return;
      }
      const sig = await signSighash(wallet.mnemonic, wallet.index, prep.sighash);
      await submitToken(token, dest, atoms, sig);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Send failed");
      setBusy(false);
    }
  }

  async function submitToken(
    token: { address: string; symbol: string; decimals: number },
    dest: string,
    atoms: bigint,
    sig: string,
  ) {
    if (!wallet) return;
    setBusy(true);
    try {
      const sub = await api<{ tx: string }>(
        tokenSubmitUrl(wallet.address, token.address, dest, atoms, sig),
        "POST",
      );
      try {
        await api("/api/produce", "POST");
      } catch {
        /* ignore */
      }
      await refreshChain(wallet.address);
      toast.success(
        `Sent ${formatTokenAmount(atoms, token.decimals)} ${token.symbol} · ${shortId(sub.tx)}`,
      );
      setTo("");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Send failed");
    } finally {
      setBusy(false);
    }
  }

  async function onHardwareSignSuccess(sig: string) {
    if (!signModalState || !wallet) return;
    const { dest, atoms, token } = signModalState;
    setSignModalState(null);
    if (token) {
      await submitToken(token, dest, BigInt(token.atoms), sig);
      return;
    }
    await submitTransaction(dest, atoms, sig);
  }

  if (!wallet) {
    return (
      <div className="mx-auto flex w-full max-w-lg flex-col gap-6 px-4 py-8 md:px-6">
        <header>
          <p className="font-mono text-[10px] tracking-brand text-gold uppercase">KUNA</p>
          <h1 className="font-display text-3xl tracking-tight text-fg">Wallet</h1>
          <p className="mt-2 text-sm leading-relaxed text-muted">
            Secure your KUNA in this browser or connect a hardware wallet. Address is the Ed25519
            public key.
          </p>
        </header>
        <div className="flex flex-col gap-3">
          {pendingMnemonic ? (
            <div className="rounded-xl border border-border bg-surface p-4">
              <p className="text-xs font-medium uppercase tracking-wide text-gold">
                Write down your recovery phrase
              </p>
              <p className="mt-1 text-[11px] text-muted">
                Anyone with these words controls your KUNA — keep them offline and never share them.
              </p>
              <div className="mt-3 grid grid-cols-3 gap-2">
                {pendingMnemonic.split(" ").map((w, i) => (
                  <div
                    key={`${w}-${i}`}
                    className="rounded-md bg-bg px-2 py-1.5 font-mono text-[11px] text-fg"
                  >
                    <span className="mr-1 text-subtle">{i + 1}.</span>
                    {w}
                  </div>
                ))}
              </div>
              <label className="mt-4 flex cursor-pointer items-start gap-2 text-xs text-muted">
                <input
                  type="checkbox"
                  checked={wroteItDown}
                  onChange={(e) => setWroteItDown(e.target.checked)}
                  className="mt-0.5 size-4 shrink-0 accent-gold"
                />
                <span>
                  I wrote down these {pendingMnemonic.split(" ").length} words and will keep them
                  offline.
                </span>
              </label>
              <div className="mt-3 flex gap-2">
                <Button
                  type="button"
                  className="h-11 flex-1 bg-gold text-black hover:bg-gold/90"
                  disabled={busy || !wroteItDown}
                  onClick={() => void onConfirmBackup()}
                >
                  {busy ? "Creating…" : "Create wallet"}
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  className="h-11 px-4"
                  disabled={busy}
                  onClick={onCancelPending}
                >
                  Cancel
                </Button>
              </div>
            </div>
          ) : (
            <>
              <p className="text-xs text-muted">
                Your wallet is one 24-word phrase. The same phrase always restores the same
                address, here and in the Android wallets.
              </p>
              <Button
                type="button"
                className="h-12 bg-gold text-black hover:bg-gold/90"
                disabled={busy}
                onClick={() => void onCreate()}
              >
                {busy ? "Working…" : "Create encrypted wallet"}
              </Button>
            </>
          )}
          <Button
            type="button"
            variant="outline"
            className="h-12"
            disabled={busy}
            onClick={() => setShowConnectModal(true)}
          >
            <Usb className="size-4 text-teal" /> Connect Hardware Wallet
          </Button>
        </div>
        {busy ? null : (
          <div className="rounded-lg bg-surface p-3 text-xs text-muted">
            <p className="font-medium text-fg">Password protects your seed</p>
            <p className="mt-1">The mnemonic is encrypted with PBKDF2 + AES-GCM in this browser.</p>
          </div>
        )}
        <form onSubmit={(e) => void onImport(e)} className="flex flex-col gap-3">
          <label className="text-[10px] tracking-wide text-subtle uppercase">
            Import recovery phrase or Address
          </label>
          <textarea
            value={phrase}
            onChange={(e) => setPhrase(e.target.value)}
            placeholder="24 recovery words, or a kvnc1… address"
            rows={3}
            className="min-h-20 rounded-md border border-border bg-bg px-3 py-2 font-mono text-sm text-fg outline-none"
          />
          <div className="relative">
            <input
              type={showPassword ? "text" : "password"}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Encryption password"
              className="h-11 w-full rounded-md border border-border bg-bg px-3 pr-10 font-mono text-sm text-fg outline-none"
            />
            <button
              type="button"
              onClick={() => setShowPassword((s) => !s)}
              className="absolute inset-y-0 right-0 px-3 text-muted"
              aria-label="Toggle password"
            >
              {showPassword ? <EyeOff className="size-4" /> : <Eye className="size-4" />}
            </button>
          </div>
          <Button
            type="submit"
            variant="outline"
            className="h-12"
            disabled={busy || !phrase.trim()}
          >
            Import
          </Button>
        </form>
        <ConnectHardwareModal
          open={showConnectModal}
          onOpenChange={setShowConnectModal}
          onConnected={(rec) => setWallet(rec)}
        />
      </div>
    );
  }

  if (isLocked(wallet)) {
    return (
      <div className="mx-auto flex w-full max-w-lg flex-col gap-6 px-4 py-8 md:px-6">
        <header>
          <p className="font-mono text-[10px] tracking-brand text-gold uppercase">KUNA</p>
          <h1 className="font-display text-3xl tracking-tight text-fg">Wallet locked</h1>
          <p className="mt-2 text-sm text-muted">Enter your password to decrypt the seed.</p>
        </header>
        <form
          onSubmit={(e) => void onUnlock(e)}
          className="flex flex-col gap-3 rounded-xl border border-border bg-surface p-4"
        >
          <div className="flex items-center gap-2 text-gold">
            <Lock className="size-4" />
            <p className="text-xs font-medium uppercase tracking-wide">Encrypted seed</p>
          </div>
          <p className="break-all font-mono text-xs text-fg">{hexToKvnc(wallet.address)}</p>
          <div className="relative">
            <input
              type={showPassword ? "text" : "password"}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Password"
              className="h-11 w-full rounded-md border border-border bg-bg px-3 pr-10 font-mono text-sm text-fg outline-none"
            />
            <button
              type="button"
              onClick={() => setShowPassword((s) => !s)}
              className="absolute inset-y-0 right-0 px-3 text-muted"
              aria-label="Toggle"
            >
              {showPassword ? <EyeOff className="size-4" /> : <Eye className="size-4" />}
            </button>
          </div>
          <Button type="submit" className="h-12" disabled={busy || !password}>
            {busy ? "Unlocking…" : "Unlock"}
          </Button>
        </form>
        <Button
          type="button"
          variant="ghost"
          className="self-start"
          onClick={() => setWallet(null)}
        >
          Use a different wallet
        </Button>
      </div>
    );
  }

  const isHardware = wallet.type === "hardware";
  const history = hist?.txs ?? [];

  return (
    <div className="mx-auto flex w-full max-w-5xl flex-col gap-6 px-4 py-6 md:px-6 md:py-8">
      <header className="flex items-start justify-between gap-3">
        <div>
          <div className="flex items-center gap-2">
            <p className="font-mono text-[10px] tracking-wide text-subtle uppercase">Balance</p>
            {isHardware && (
              <span className="flex items-center gap-1 rounded-full border border-border bg-surface-2 px-2 py-0.5 font-mono text-[10px] text-teal">
                <Usb className="size-3 text-teal" />
                <span className="capitalize">{wallet.deviceType}</span>
              </span>
            )}
          </div>
          <p className="mt-1 font-display text-4xl tabular-nums tracking-tight text-fg">
            {fmtKvnc(balance)}
          </p>
          {selectedToken ? (
            <p className="mt-1 font-mono text-xs text-gold">
              {formatTokenAmount(tokenBalance, selectedToken.decimals)} {selectedToken.symbol}
            </p>
          ) : (
            !isNativeAsset(assetId) && (
              <p className="mt-1 font-mono text-xs text-gold">
                Selected asset: {fmtKvnc(assetBalance)} · {assetLabel(assetId)}
              </p>
            )
          )}
        </div>
        <div className="flex gap-1">
          {isPlaintext(wallet) && (
            <Button
              type="button"
              variant="ghost"
              size="icon"
              aria-label="Encrypt seed"
              onClick={() => void onSecure()}
            >
              <Lock className="size-4 text-gold" />
            </Button>
          )}
          {wallet.mnemonic && (
            <Button type="button" variant="ghost" size="icon" aria-label="Lock" onClick={onLock}>
              <Unlock className="size-4" />
            </Button>
          )}
        </div>
      </header>

      {/* Desktop: send column + history column side by side. Mobile: stacked. */}
      <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_360px]">
        <div className="flex min-w-0 flex-col gap-6">
          <div className="rounded-xl border border-border bg-surface p-4">
            <p className="break-all font-mono text-xs text-fg">{hexToKvnc(wallet.address)}</p>
            <div className="mt-3 flex flex-wrap gap-2">
              <Button type="button" variant="outline" size="sm" onClick={() => void onCopy()}>
                <Copy className="size-3.5" /> Copy
              </Button>
              <AddressQr value={hexToKvnc(wallet.address)} />
              {!live && (
                <Button type="button" variant="outline" size="sm" onClick={() => void onFaucet()}>
                  Faucet 1 KUNA
                </Button>
              )}
            </div>
          </div>

          <form
            onSubmit={(e) => void onSend(e)}
            className="flex flex-col gap-4 rounded-xl border border-border bg-surface p-4"
          >
            <label className="text-xs text-muted">
              To
              <input
                value={to}
                onChange={(e) => setTo(e.target.value)}
                placeholder="kvnc…dag or 64-hex"
                className="mt-1 h-11 w-full rounded-md border border-border bg-bg px-3 font-mono text-sm text-fg outline-none"
              />
            </label>

            <AssetPicker
              options={assetOptions}
              value={pickerValue}
              onChange={onPickAsset}
              disabled={busy}
            />

            {isNftSelected && (
              <div className="rounded-lg border border-purple/30 bg-purple/5 p-3 flex items-center gap-2">
                <Gem className="size-4 text-purple" />
                <div>
                  <p className="text-sm font-medium text-purple">NFT Transfer</p>
                  <p className="text-[11px] text-muted">
                    Entire token (1 unit) will be sent. Cannot be split.
                  </p>
                </div>
              </div>
            )}

            <label className="text-xs text-muted">
              Amount (
              {selectedToken
                ? selectedToken.symbol
                : isNativeAsset(assetId)
                  ? TOKEN
                  : isNftSelected
                    ? "NFT (fixed: 1)"
                    : "asset units"}
              )
              <div className="mt-1 flex gap-2">
                <input
                  value={amount}
                  onChange={(e) => {
                    if (!isNftSelected) setAmount(e.target.value);
                  }}
                  inputMode="decimal"
                  disabled={isNftSelected}
                  className="h-11 min-w-0 flex-1 rounded-md border border-border bg-bg px-3 font-mono text-sm text-fg outline-none disabled:bg-surface disabled:text-muted"
                  placeholder={isNftSelected ? "1 (fixed)" : ""}
                />
                {!isNftSelected && (
                  <Button
                    type="button"
                    variant="outline"
                    className="h-11 px-3"
                    disabled={busy || !utxos}
                    onClick={() => {
                      if (selectedToken) {
                        setAmount(
                          formatTokenAmount(tokenBalance, selectedToken.decimals) || "0",
                        );
                        return;
                      }
                      const feeDeduct = isNativeAsset(assetId) ? fee : 0;
                      const v = Math.max(0, spendable - feeDeduct) / ATOM;
                      setAmount(v.toFixed(8).replace(/\.?0+$/, "") || "0");
                    }}
                  >
                    Max
                  </Button>
                )}
              </div>
            </label>

            <div className="flex gap-2">
              {(["slow", "normal", "fast"] as const).map((tier) => (
                <Button
                  key={tier}
                  type="button"
                  variant={feeTier === tier ? "default" : "outline"}
                  size="sm"
                  onClick={() => setFeeTier(tier)}
                  className="flex-1 capitalize"
                >
                  {tier}
                </Button>
              ))}
            </div>
            <p className="text-[11px] text-muted text-center">
              Fee ≈ {fmtKvnc(selectedToken ? TOKEN_SEND_FEE : fee)} {TOKEN}
            </p>

            <Button type="submit" className="h-12" disabled={busy}>
              {busy
                ? "Sending…"
                : isHardware
                  ? `Confirm & send with ${wallet.deviceType}`
                  : live
                    ? "Sign & send on Testnet"
                    : "Send"}
            </Button>
          </form>
        </div>

        <div className="flex min-w-0 flex-col gap-6">
          <section>
            <p className="mb-2 text-[10px] tracking-wide text-subtle uppercase">Tokens</p>
            {tokens.length === 0 ? (
              <p className="text-sm text-muted">
                No KVP-102 tokens on {live ? "Testnet" : "this chain"} yet.
              </p>
            ) : (
              <ul className="divide-y divide-border rounded-xl border border-border">
                {tokens.slice(0, 8).map((token) => (
                  <li key={token.address} className="flex items-center gap-3 px-4 py-3">
                    {token.icon ? (
                      // eslint-disable-next-line @next/next/no-img-element
                      <img
                        src={token.icon}
                        alt=""
                        className="size-8 shrink-0 rounded-full border border-border object-cover"
                      />
                    ) : (
                      <span className="flex size-8 shrink-0 items-center justify-center rounded-full bg-surface-2 font-mono text-[11px] text-teal">
                        {token.symbol.slice(0, 2).toUpperCase()}
                      </span>
                    )}
                    <span className="min-w-0 flex-1">
                      <span className="block truncate text-sm text-fg">{token.symbol}</span>
                      <span className="block truncate text-[11px] text-subtle">{token.name}</span>
                    </span>
                    <span className="shrink-0 text-right font-mono text-xs tabular-nums text-muted">
                      {formatTokenAmount(BigInt(token.balance ?? "0"), token.decimals)}
                      {token.paused ? <span className="ml-1 text-[10px] text-gold">paused</span> : null}
                    </span>
                  </li>
                ))}
              </ul>
            )}
          </section>

          <section>
            <p className="mb-2 text-[10px] tracking-wide text-subtle uppercase">History</p>
            {history.length === 0 ? (
              <p className="text-sm text-muted">
                {live ? "No movements on Testnet yet." : "No movements yet. Use faucet or send."}
              </p>
            ) : (
              <ul className="divide-y divide-border rounded-xl border border-border">
                {history
                  .slice(-12)
                  .reverse()
                  .map((row) => (
                    <li
                      key={row.tx}
                      className="flex items-baseline justify-between gap-3 px-4 py-3"
                    >
                      <span className="min-w-0">
                        <span className="block text-sm capitalize text-fg">{row.kind}</span>
                        <span className="font-mono text-[11px] text-subtle">{shortId(row.tx)}</span>
                      </span>
                      <span className="shrink-0 font-mono text-xs tabular-nums text-muted">
                        {row.delta > 0 ? "+" : ""}
                        {fmtKvnc(Math.abs(row.delta))}
                        {!isNativeAsset(row.asset_id) ? (
                          <span className="ml-1 text-[10px] text-subtle">
                            {assetLabel(row.asset_id)}
                          </span>
                        ) : null}
                      </span>
                    </li>
                  ))}
              </ul>
            )}
          </section>
        </div>
      </div>

      {isHardware && signModalState && (
        <HardwareSignModal
          open={signModalState.open}
          deviceType={wallet.deviceType}
          accountIndex={wallet.index}
          path={wallet.path}
          recipient={signModalState.dest}
          amountKvnc={signModalState.amountKvnc}
          feeKvnc={signModalState.feeKvnc}
          sighash={signModalState.sighash}
          onSuccess={(sig) => void onHardwareSignSuccess(sig)}
          onCancel={() => setSignModalState(null)}
        />
      )}
      <ConnectHardwareModal
        open={showConnectModal}
        onOpenChange={setShowConnectModal}
        onConnected={(rec) => setWallet(rec)}
      />
    </div>
  );
}
