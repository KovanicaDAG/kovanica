import { useState } from "react";
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

export function Contracts() {
  const { client } = useSession();
  const [kind, setKind] = useState<ContractKind>(ContractKind.HTLC);
  const [method, setMethod] = useState<string>(HtlcMethod.CREATE);
  const [argsText, setArgsText] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<string | null>(null);

  const [indexerUrl, setIndexerUrl] = useState("");
  const [contractAddr, setContractAddr] = useState("");
  const [events, setEvents] = useState<DecodedContractEvent[]>([]);
  const [eventsError, setEventsError] = useState<string | null>(null);

  function onKindChange(next: ContractKind) {
    setKind(next);
    setMethod(METHODS[next][0]);
  }

  async function run() {
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      const args = argsText
        .split(",")
        .map((s) => s.trim())
        .filter((s) => s.length > 0)
        .map((s) => Number(s));
      if (args.some((n) => Number.isNaN(n))) throw new Error("args must be comma-separated numbers");
      const params = buildContractCall(kind, method, args);
      const res = await invokeContract(client, params);
      setResult(JSON.stringify(res, null, 2));
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
        <label>
          Args (comma-separated numbers)
          <input value={argsText} onChange={(e) => setArgsText(e.target.value)} placeholder="1, 2, 3" />
        </label>
        <button type="button" className="btn primary" onClick={() => void run()} disabled={busy}>
          {busy ? "Submitting…" : "Invoke"}
        </button>
      </div>

      {error ? <p className="error">{error}</p> : null}
      {result ? <pre className="mono">{result}</pre> : null}

      <div className="card">
        <h2>Decoded events</h2>
        <label>
          Indexer URL
          <input value={indexerUrl} onChange={(e) => setIndexerUrl(e.target.value)} placeholder="http://127.0.0.1:8080" />
        </label>
        <label>
          Contract address (hex bytes)
          <input value={contractAddr} onChange={(e) => setContractAddr(e.target.value)} placeholder="0707... (64 hex)" />
        </label>
        <button type="button" className="btn" onClick={() => void loadEvents()}>
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