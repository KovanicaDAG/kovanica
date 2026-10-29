import { useEffect, useState } from "react";

import { KovanicaIndexerClient, type IndexedEvent } from "@kovanica/api-client";
import { loadActivity, saveActivity, type ActivityItem } from "../activity";
import { useSession } from "../session";

export function Activity() {
  const { network, client, headTick } = useSession();
  const [items, setItems] = useState<ActivityItem[]>(() => loadActivity(network.id));
  const [indexerEvents, setIndexerEvents] = useState<IndexedEvent[]>([]);
  const [indexerError, setIndexerError] = useState<string | null>(null);

  useEffect(() => {
    setItems(loadActivity(network.id));
  }, [network.id]);

  useEffect(() => {
    const pending = items.filter((item) => item.status === "pending");
    if (pending.length === 0) return;
    let cancel = false;
    void (async () => {
      const next = await Promise.all(
        items.map(async (item) => {
          if (item.status !== "pending") return item;
          try {
            const receipt = await client.getTransactionReceipt(item.hash);
            return receipt ? { ...item, status: "committed" as const } : item;
          } catch {
            return item;
          }
        }),
      );
      if (cancel) return;
      if (next.some((item, index) => item.status !== items[index]?.status)) {
        saveActivity(network.id, next);
        setItems(next);
      }
    })();
    return () => {
      cancel = true;
    };
  }, [client, headTick, items, network.id]);

  useEffect(() => {
    const url = network.indexerHttpUrl;
    if (!url) {
      setIndexerEvents([]);
      setIndexerError(null);
      return;
    }
    let cancelled = false;
    const load = async () => {
      try {
        const indexer = new KovanicaIndexerClient(url);
        const logs = await indexer.getLogs({ limit: 20 });
        if (cancelled) return;
        setIndexerEvents(logs);
        setIndexerError(null);
      } catch (err) {
        if (cancelled) return;
        setIndexerError(err instanceof Error ? err.message : "Indexer unavailable");
      }
    };
    void load();
    const timer = window.setInterval(() => void load(), 15000);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [network.indexerHttpUrl]);

  return (
    <section>
      <h1>Activity</h1>
      <p className="lede">
        Transactions this browser submitted on {network.label}. The node does not list the mempool,
        so only these local submissions are shown.
      </p>
      {items.length === 0 ? (
        <p className="muted">Nothing submitted from this browser on {network.label} yet.</p>
      ) : (
        <table>
          <thead>
            <tr>
              <th>When</th>
              <th>Summary</th>
              <th>Status</th>
              <th>Hash</th>
            </tr>
          </thead>
          <tbody>
            {items.map((item) => (
              <tr key={item.hash}>
                <td>{new Date(item.submittedAt).toLocaleString()}</td>
                <td>
                  {item.summary}
                  <div className="muted">fee {item.fee} KUNA</div>
                </td>
                <td>{item.status === "committed" ? "In a block" : "Pending"}</td>
                <td className="mono">{item.hash}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {network.indexerHttpUrl ? (
        <section style={{ marginTop: "24px" }}>
          <h2>Recent contract events (indexer)</h2>
          {indexerError ? <p className="error">{indexerError}</p> : null}
          {indexerEvents.length === 0 ? (
            <p className="muted">No indexed events.</p>
          ) : (
            <table>
              <thead>
                <tr>
                  <th>Height</th>
                  <th>Address</th>
                  <th>Topic</th>
                  <th>TX</th>
                </tr>
              </thead>
              <tbody>
                {indexerEvents.map((ev) => (
                  <tr key={ev.id}>
                    <td className="mono">{ev.height}</td>
                    <td className="mono">{ev.address.map((b) => b.toString(16).padStart(2, "0")).join("").slice(0, 12)}…</td>
                    <td className="mono">
                      {(() => {
                        try {
                          return new TextDecoder().decode(new Uint8Array(ev.topic));
                        } catch {
                          return ev.topic.map((b) => b.toString(16).padStart(2, "0")).join("");
                        }
                      })()}
                    </td>
                    <td className="mono">{ev.tx_hash.map((b) => b.toString(16).padStart(2, "0")).join("").slice(0, 12)}…</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </section>
      ) : null}
    </section>
  );
}
