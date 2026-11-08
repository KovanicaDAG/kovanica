import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import {
  KovanicaRpcClient,
  subscribeNewHeads,
  type DagTipsResponse,
  type MiningInfo,
} from "@kovanica/api-client";
import {
  generateKey,
  keyFromHexSeed,
  keyFromMnemonic,
  keyFromSeed,
  type AccountKey,
} from "@kovanica/signer";

import {
  HYBRID_TESTNET,
  LOCAL,
  MAINNET,
  NETWORK_STORAGE_KEY,
  TESTNET,
  rejectionMessage,
  resolveNetwork,
  type Network,
} from "./network";

interface Session {
  networks: Network[];
  network: Network;
  notice: string | null;
  selectNetwork: (id: string) => void;
  key: AccountKey | null;
  createKey: () => Promise<AccountKey>;
  importHex: (hex: string) => Promise<AccountKey>;
  importPhrase: (phrase: string) => Promise<AccountKey>;
  importSeed: (seed: Uint8Array) => Promise<AccountKey>;
  lock: () => void;
  height: bigint | null;
  online: boolean;
  headTick: number;
  client: KovanicaRpcClient;
  miningInfo: MiningInfo | null;
  dagTips: DagTipsResponse | null;
}

const SessionContext = createContext<Session | null>(null);

function readQueryNetwork(): string | null {
  return new URLSearchParams(window.location.search).get("network");
}

export function SessionProvider({ children }: { children: ReactNode }) {
  const networks = useMemo(() => {
    const list = [TESTNET, MAINNET, HYBRID_TESTNET];
    // `import.meta.env.DEV` is false in production, so the local node is
    // dropped from the bundle.
    if (import.meta.env.DEV) list.push(LOCAL);
    return list;
  }, []);
  const [network, setNetwork] = useState<Network>(() =>
    resolveNetwork(readQueryNetwork(), localStorage.getItem(NETWORK_STORAGE_KEY), networks),
  );
  const [notice, setNotice] = useState<string | null>(() =>
    rejectionMessage(readQueryNetwork(), networks),
  );
  const [key, setKey] = useState<AccountKey | null>(null);
  const [height, setHeight] = useState<bigint | null>(null);
  const [online, setOnline] = useState(false);
  const [headTick, setHeadTick] = useState(0);
  const [miningInfo, setMiningInfo] = useState<MiningInfo | null>(null);
  const [dagTips, setDagTips] = useState<DagTipsResponse | null>(null);

  const client = useMemo(() => {
    const url = network.rpcUrl.endsWith("/rpc")
      ? network.rpcUrl
      : `${network.rpcUrl.replace(/\/+$/, "")}/rpc`;
    return new KovanicaRpcClient(url);
  }, [network.rpcUrl]);

  useEffect(() => {
    setKey((current) => {
      if (current) current.seed.fill(0);
      return null;
    });
    setHeight(null);
    setOnline(false);
    setMiningInfo(null);
    setDagTips(null);
  }, [network.id]);

  useEffect(() => {
    let cancel = false;
    async function fetchHybrid() {
      try {
        const [mInfoRes, dagTipsRes] = await Promise.allSettled([
          client.getMiningInfo(),
          client.getDagTips(),
        ]);
        if (cancel) return;
        setMiningInfo(mInfoRes.status === "fulfilled" ? mInfoRes.value : null);
        setDagTips(dagTipsRes.status === "fulfilled" ? dagTipsRes.value : null);
      } catch {
        if (!cancel) {
          setMiningInfo(null);
          setDagTips(null);
        }
      }
    }
    void fetchHybrid();
    return () => {
      cancel = true;
    };
  }, [client, headTick, network.id]);

  useEffect(() => {
    if (typeof WebSocket === "undefined") return;
    let closed = false;
    const stop = subscribeNewHeads(
      network.wsUrl,
      (head) => {
        if (closed) return;
        setOnline(true);
        if (head && typeof head === "object" && "round" in head) {
          const round = (head as { round?: unknown }).round;
          if (typeof round === "string") {
            try {
              setHeight(BigInt(round));
            } catch {
              /* ignore a malformed head */
            }
          }
        }
        setHeadTick((tick) => tick + 1);
      },
      () => {
        if (!closed) setOnline(false);
      },
    );
    return () => {
      closed = true;
      stop();
    };
  }, [network.wsUrl]);

  function selectNetwork(id: string) {
    const next = networks.find((item) => item.id === id);
    if (!next || !next.enabled) {
      setNotice(next?.soon ?? MAINNET.soon ?? "That network is not open yet.");
      return;
    }
    setNotice(null);
    if (next.id === network.id) return;
    setKey((current) => {
      if (current) current.seed.fill(0);
      return null;
    });
    localStorage.setItem(NETWORK_STORAGE_KEY, next.id);
    setNetwork(next);
  }

  async function hold(next: AccountKey): Promise<AccountKey> {
    setKey((current) => {
      if (current && current.seed !== next.seed) current.seed.fill(0);
      return next;
    });
    return next;
  }

  const value: Session = {
    networks,
    network,
    notice,
    selectNetwork,
    key,
    createKey: () => generateKey().then(hold),
    importHex: (hex) => keyFromHexSeed(hex).then(hold),
    importPhrase: (phrase) => keyFromMnemonic(phrase).then(hold),
    importSeed: (seed) => keyFromSeed(seed).then(hold),
    lock: () => {
      setKey((current) => {
        if (current) current.seed.fill(0);
        return null;
      });
    },
    height,
    online,
    headTick,
    client,
    miningInfo,
    dagTips,
  };

  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
}

export function useSession(): Session {
  const session = useContext(SessionContext);
  if (!session) throw new Error("useSession outside SessionProvider");
  return session;
}
