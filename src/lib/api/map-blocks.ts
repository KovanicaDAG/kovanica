import type { Block } from "@/lib/ledger/types";
import type { ApiDagBlock } from "./contract";

export function dagToBlocks(dag: ApiDagBlock[], order: string[]): Block[] {
  const heightOf = new Map(order.map((id, i) => [id, i]));
  return dag.map((d) => ({
    id: d.id,
    parents: d.parents,
    selectedParent: d.selected_parent,
    work: d.work,
    timestamp: d.timestamp_ms,
    nonce: d.nonce,
    blueScore: d.blue_score,
    colour: d.colour,
    height: typeof d.work === "number" ? d.work : (heightOf.get(d.id) ?? 0),
    txs: d.txs.map((t) => ({
      id: t.id,
      coinbase: t.coinbase,
      to: t.outputs[0]?.owner,
      amount: t.outputs.reduce((n, o) => n + o.value, 0),
    })),
  }));
}

/**
 * Compare two DAG arrays by block IDs, parents, colours, and work heights.
 */
export function isDagUnchanged(prevDag: ApiDagBlock[], nextDag: ApiDagBlock[]): boolean {
  if (prevDag === nextDag) return true;
  if (!prevDag || !nextDag) return false;
  if (prevDag.length !== nextDag.length) return false;
  for (let i = 0; i < prevDag.length; i++) {
    const a = prevDag[i];
    const b = nextDag[i];
    if (a.id !== b.id || a.colour !== b.colour || a.work !== b.work) return false;
    if (a.parents.length !== b.parents.length) return false;
    for (let j = 0; j < a.parents.length; j++) {
      if (a.parents[j] !== b.parents[j]) return false;
    }
  }
  return true;
}

/**
 * Cache helper that preserves array identity when the DAG topology (ids, edges, colours, heights)
 * has not changed. This prevents canvas and layout re-computations on poll ticks when no new blocks arrived.
 */
export function createDagBlocksCache() {
  let lastDag: ApiDagBlock[] | null = null;
  let lastBlocks: Block[] = [];

  return function getCachedBlocks(dag: ApiDagBlock[], order: string[]): Block[] {
    if (lastDag && isDagUnchanged(lastDag, dag)) {
      return lastBlocks;
    }
    lastDag = dag;
    lastBlocks = dagToBlocks(dag, order);
    return lastBlocks;
  };
}
