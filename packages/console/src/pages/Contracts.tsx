import { useEffect, useState } from "react";
import {
  ContractKind,
  HtlcMethod,
  VaultMethod,
  MultisigMethod,
  TokenMethod,
  decodeContractEvent,
  KovanicaIndexerClient,
  type DecodedContractEvent,
  type IndexedEvent,
} from "@kovanica/api-client";

import { buildContractCall, invokeContract } from "../contracts";
import { useSession } from "../session";

const METHODS: Record<ContractKind, readonly string[]> = {
  [ContractKind.HTLC]: Object.values(HtlcMethod),
  [ContractKind.Vault]: Object.values(VaultMethod),
  [ContractKind.Multisig]: Object.values(MultisigMethod),
  [ContractKind.Token]: Object.values(TokenMethod),
};

const METHOD_HINTS: Record<ContractKind, Record<string, string>> = {
  [ContractKind.HTLC]: {
    [HtlcMethod.CREATE]: "claimer (hex bytes), amount, hash_lock (hex bytes), expiry",
    [HtlcMethod.CLAIM]: "swap id (hex bytes), preimage (hex bytes)",
    [HtlcMethod.REFUND]: "swap id (hex bytes)",
  },
  [ContractKind.Vault]: {
    [VaultMethod.CREATE]: "beneficiary (hex bytes), amount, vesting schedule",
    [VaultMethod.CLAIM]: "vault id (hex bytes)",
    [VaultMethod.CANCEL]: "vault id (hex bytes)",
  },
  [ContractKind.Multisig]: {
    [MultisigMethod.PROPOSE]: "target (hex bytes), value, data (hex bytes)",
    [MultisigMethod.CONFIRM]: "transaction id",
    [MultisigMethod.EXECUTE]: "transaction id",
  },
  [ContractKind.Token]: {
    [TokenMethod.TRANSFER]: "recipient (hex bytes), amount",
    [TokenMethod.APPROVE]: "spender (hex bytes), amount",
    [TokenMethod.TRANSFER_FROM]: "from (hex bytes), to (hex bytes), amount",
    [TokenMethod.MINT]: "recipient (hex bytes), amount",
    [TokenMethod.BURN]: "amount",
  },
};

const METHOD_ARG_COUNT: Record<ContractKind, Record<string, number>> = {
  [ContractKind.HTLC]: {
    [HtlcMethod.CREATE]: 4,
    [HtlcMethod.CLAIM]: 2,
    [HtlcMethod.REFUND]: 1,
  },
  [ContractKind.Vault]: {
    [VaultMethod.CREATE]: 3,
    [VaultMethod.CLAIM]: 1,
    [VaultMethod.CANCEL]: 1,
  },
  [ContractKind.Multisig]: {
    [MultisigMethod.PROPOSE]: 3,
    [MultisigMethod.CONFIRM]: 1,
    [MultisigMethod.EXECUTE]: 1,
  },
  [ContractKind.Token]: {
    [TokenMethod.TRANSFER]: 2,
    [TokenMethod.APPROVE]: 2,
    [TokenMethod.TRANSFER_FROM]: 3,
    [TokenMethod.MINT]: 2,
    [TokenMethod.BURN]: 1,
  },
};

export function Contracts() {
  const { client, network } = useSession();
  const [kind, setKind] = useState<ContractKind>(ContractKind.HTLC);
  const [method, setMethod] = useState<string>(HtlcMethod.CREATE);
  const [argsText, setArgsText] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<string | null>(null);
  const [txHash, setTxHash] = useState<string | null>(null);
  const [txStatus, setTxStatus] = useState<string | null>(null);

  const [indexerUrl, setIndexerUrl] = useState<string | null>(network.indexerHttpUrl ?? null);
  const [contractAddr, setContractAddr] = useState("");
  const [events, setEvents] = useState<DecodedContractEvent[]>([]);
  const [eventsError, setEventsError] = useState<string | null>(null);

  useEffect(() => {
    setIndexerUrl(network.indexerHttpUrl ?? null);
  }, [network.id, network.indexerHttpUrl]);

  function onKindChange(next: ContractKind) {
    setKind(next);
    setMethod(METHODS[next][0]);
  }

  async function run() {
    setBusy(true);
    setError(null);
    setResult(null);
    setTxHash(null);
    setTxStatus(null);
    try {
      const args = argsText
        .split(",")
        .map((s) => s.trim())
        .filter((s) => s.length > 0)
        .map((s) => Number(s));
      if (args.some((n) => Number.isNaN(n))) throw new Error("args must be comma-separated numbers");
      const expected = METHOD_ARG_COUNT[kind]?.[method];
      if (expected != null && args.length !== expected) {
        throw new Error(`${method} expects ${expected} argument(s)`);
      }
      const params = buildContractCall(kind, method, args);
      const res = await invokeContract(client, params);
      setResult(JSON.stringify(res, null, 2));
      const r = (res && typeof res === "object" ? res : null) as
        | { transactionHash?: unknown; hash?: unknown; status?: unknown }
        | null;
      const h = r?.transactionHash ?? r?.hash;
      if (typeof h === "string") setTxHash(h);
      if (typeof r?.status === "string") setTxStatus(r.status);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Contract call failed");
    } finally {
      setBusy(false);
    }
  }

  async function loadEvents() {
    setEventsError(null);
    setEvents([]);
    if (!indexerUrl || !contractAddr) {
      setEventsError("Indexer URL and contract address are required");
      return;
    }
    try {
      const ic = new KovanicaIndexerClient(indexerUrl);
      const raw: IndexedEvent[] = await ic.getContractEvents(contractAddr);
      setEvents(raw.map((e) => decodeContractEvent(e)));
    } catch (err) {
      setEventsError(err instanceof Error ? err.message : "Failed to load events");
    }
  }

  useEffect(() => {
    if (!txHash) return;
    let cancelled = false;
    const poll = async () => {
      for (let i = 0; i < 5 && !cancelled; i += 1) {
        await new Promise((resolve) => setTimeout(resolve, 2000));
        if (cancelled) return;
        try {
          const receipt = await client.getTransactionReceipt(txHash);
          if (receipt) {
            setTxStatus(
              receipt.status ?? (receipt.success ? "committed" : "failed"),
            );
            return;
          }
        } catch {
          /* keep polling */
        }
      }
    };
    void poll();
    return () => {
      cancelled = true;
    };
  }, [txHash, client]);

  const eventsDisabled = !indexerUrl;

  return (
    <section>
      <h1>Contracts</h1>
      <p className="lede">Build and submit contract calls, then decode emitted events.</p>

      <div className="card">
        <label>
          Contract
          <select value={kind} onChange={(e) => onKindChange(e.target.value as ContractKind)}>
            <option value={ContractKind.HTLC}>HTLC</option>
            <option value={ContractKind.Vault}>Vault</option>
            <option value={ContractKind.Multisig}>Multisig</option>
            <option value={ContractKind.Token}>Token</option>
          </select>
        </label>
        <label>
          Method
          <select value={method} onChange={(e) => setMethod(e.target.value)}>
            {METHODS[kind].map((m) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </select>
        </label>
        <p className="muted">
          Args: {METHOD_HINTS[kind]?.[method] ?? "comma-separated numbers"}
        </p>
        <label>
          Args (comma-separated numbers)
          <input value={argsText} onChange={(e) => setArgsText(e.target.value)} placeholder="1, 2, 3" />
        </label>
        <button type="button" className="btn primary" onClick={() => void run()} disabled={busy}>
          {busy ? "Submitting…" : "Invoke"}
        </button>
      </div>

      {error ? <p className="error">{error}</p> : null}
      {txHash ? <p className="mono">Submitted {txHash}{txStatus ? ` (${txStatus})` : ""}</p> : null}
      {result ? <pre className="mono">{result}</pre> : null}

      <div className="card">
        <h2>Decoded events</h2>
        {eventsDisabled ? (
          <p className="muted">Indexer not configured for this network.</p>
        ) : null}
        <label>
          Indexer URL
          <input
            value={indexerUrl ?? ""}
            onChange={(e) => setIndexerUrl(e.target.value)}
            placeholder="http://127.0.0.1:8080"
            disabled={eventsDisabled}
          />
        </label>
        <label>
          Contract address (hex bytes)
          <input
            value={contractAddr}
            onChange={(e) => setContractAddr(e.target.value)}
            placeholder="0707... (64 hex)"
            disabled={eventsDisabled}
          />
        </label>
        <button type="button" className="btn" onClick={() => void loadEvents()} disabled={eventsDisabled}>
          Load events
        </button>
        {eventsError ? <p className="error">{eventsError}</p> : null}
        {events.length === 0 ? (
          <p className="muted">No events loaded.</p>
        ) : (
          <ul>
            {events.map((ev, i) => (
              <li key={i} className="mono">
                <div>address: {ev.addressHex}</div>
                <div>topic: {ev.topicHex}</div>
                <div>data: {ev.dataHex}</div>
                {ev.dataText ? <div>text: {ev.dataText}</div> : null}
                {ev.json ? <pre>{JSON.stringify(ev.json, null, 2)}</pre> : null}
              </li>
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}