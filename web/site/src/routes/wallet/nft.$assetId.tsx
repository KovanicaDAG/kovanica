import { createFileRoute } from "@tanstack/react-router";
import { Link, useLoaderData, Outlet } from "@tanstack/react-router";
import { ArrowLeft, Image, Sparkles as SparklesIcon, ExternalLink, Copy, AlertCircle, Loader2, Gem as GemIcon, ArrowLeft as ArrowLeftIcon } from "lucide-react";
import { api, useApiSource, isPublic } from "@/lib/api/client";
import { shortId } from "@/lib/ledger/hash";
import { hexToKvnc } from "@/lib/wallet/address";
import { isNativeAsset } from "@/lib/api/contract";
import { cn } from "@/lib/utils";

interface NftDetailResponse {
  asset_id: string;
  kind: string;
  max_supply: number;
  minted: number;
  metadata_hash: string | null;
  collection_id: string | null;
  creator: string | null;
  owner: string | null;
  owner_tx: string | null;
  owner_index: number | null;
}

interface NftMetadata {
  name?: string;
  description?: string;
  image?: string;
  external_url?: string;
  attributes?: Array<{ trait_type: string; value: string | number }>;
  collection?: { id: string; name: string };
}

export const Route = createFileRoute("/wallet/nft/$assetId")({
  loader: async ({ params }) => {
    const source = "local";
    const url = `/api/nft/${params.assetId}?source=${source}`;
    const data = await api<NftDetailResponse>(url);
    return data;
  },
  component: NftDetailPage,
});

interface NftDetailResponse {
  asset_id: string;
  kind: string;
  max_supply: number;
  minted: number;
  metadata_hash: string | null;
  collection_id: string | null;
  creator: string | null;
  owner: string | null;
  owner_tx: string | null;
  owner_index: number | null;
}

interface NftMetadata {
  name?: string;
  description?: string;
  image?: string;
  external_url?: string;
  attributes?: Array<{ trait_type: string; value: string | number }>;
  collection?: { id: string; name: string };
}

export default function NftDetailPage() {
  const nft = useLoaderData<NftDetailResponse>();
  const source = useApiSource();
  const live = isPublic(source);
  const [metadata, setMetadata] = useState<NftMetadata | null>(null);
  const [loadingMetadata, setLoadingMetadata] = useState(false);

  useEffect(() => {
    if (nft.metadata_hash) {
      setLoadingMetadata(true);
      // Try to fetch metadata from IPFS
      // In a real implementation, we'd need the original URI
      // For now, show unresolved state
      setLoadingMetadata(false);
    }
  }, [nft.metadata_hash]);

  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text);
    // Could add toast here
  };

  const formatAddress = (addr: string | null) => {
    if (!addr) return "Unknown";
    return hexToKvnc(addr);
  };

  return (
    <div className="mx-auto max-w-3xl px-4 py-6 md:px-6 md:py-8">
      <div className="mb-6 flex items-center gap-3">
        <Link
          to="/wallet"
          className="text-muted hover:text-fg transition-colors"
          aria-label="Back to wallet"
        >
          <ArrowLeft className="size-5" />
        </Link>
        <div>
          <p className="font-mono text-[10px] tracking-wide text-gold uppercase">KVP-106 NFT</p>
          <h1 className="font-display text-2xl tracking-tight text-fg">
            {metadata?.name || `NFT ${shortId(nft.asset_id)}`}
          </h1>
        </div>
      </div>

      <div className="space-y-6">
        {/* NFT Image / Visual */}
        <div className="relative aspect-square rounded-xl border border-border bg-surface overflow-hidden">
          {metadata?.image ? (
            <img
              src={metadata.image.replace("ipfs://", "https://ipfs.io/ipfs/")}
              alt={metadata.name || "NFT"}
              className="w-full h-full object-cover"
            />
          ) : (
            <div className="flex h-full items-center justify-center">
              <GemIcon className="size-16 text-purple/50" />
            </div>
          )}
          <div className="absolute top-3 right-3 flex gap-1">
            <span className="rounded-full bg-purple/90 px-2 py-0.5 font-mono text-[10px] text-white">
              NFT
            </span>
            {nft.collection_id && (
              <Link
                to={`/wallet/collection/${nft.collection_id}`}
                className="rounded-full bg-surface/90 px-2 py-0.5 font-mono text-[10px] text-fg hover:bg-surface"
              >
                <Sparkles className="size-3" />
              </Link>
            )}
          </div>
        </div>

        {/* Metadata */}
        {metadata && (
          <div className="space-y-3">
            {metadata.description && (
              <p className="text-sm text-muted leading-relaxed">{metadata.description}</p>
            )}
            {metadata.attributes && metadata.attributes.length > 0 && (
              <div className="flex flex-wrap gap-2">
                {metadata.attributes.map((attr, i) => (
                  <span
                    key={i}
                    className="rounded-full border border-border bg-surface px-3 py-1 text-sm text-fg"
                  >
                    {attr.trait_type}: {attr.value}
                  </span>
                ))}
              </div>
            )}
            {metadata.external_url && (
              <a
                href={metadata.external_url}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-1 text-sm text-gold hover:underline"
              >
                <ExternalLink className="size-3.5" />
                View External
              </a>
            )}
          </div>
        )}

        {!metadata && nft.metadata_hash && (
          <div className="rounded-lg border border-border bg-surface p-4 text-center">
            <AlertCircle className="size-6 mx-auto text-amber" />
            <p className="mt-2 text-sm text-muted">Metadata unavailable</p>
            <p className="mt-1 font-mono text-[10px] text-subtle">
              Hash: {nft.metadata_hash.slice(0, 16)}…
            </p>
            <p className="mt-2 text-[11px] text-subtle">
              Original URI not stored on-chain. Add IPFS gateway support to resolve.
            </p>
          </div>
        )}

        {/* Details Grid */}
        <div className="grid gap-4 md:grid-cols-2">
          <div className="rounded-xl border border-border bg-surface p-4">
            <p className="text-[10px] tracking-wide text-subtle uppercase">Asset ID</p>
            <div className="mt-1 flex items-center gap-2">
              <code className="break-all font-mono text-xs text-fg flex-1">{nft.asset_id}</code>
              <button
                onClick={() => copyToClipboard(nft.asset_id, "Asset ID")}
                className="text-muted hover:text-fg"
                aria-label="Copy asset ID"
              >
                <Copy className="size-4" />
              </button>
            </div>
          </div>

          <div className="rounded-xl border border-border bg-surface p-4">
            <p className="text-[10px] tracking-wide text-subtle uppercase">Current Owner</p>
            <div className="mt-1 flex items-center gap-2">
              <code className="break-all font-mono text-xs text-fg flex-1">{formatAddress(nft.owner)}</code>
              {nft.owner && (
                <button
                  onClick={() => copyToClipboard(nft.owner, "Owner address")}
                  className="text-muted hover:text-fg"
                  aria-label="Copy owner address"
                >
                  <Copy className="size-4" />
                </button>
              )}
            </div>
          </div>

          <div className="rounded-xl border border-border bg-surface p-4">
            <p className="text-[10px] tracking-wide text-subtle uppercase">Supply</p>
            <p className="mt-1 font-display text-2xl tabular-nums text-fg">
              {nft.minted} / {nft.max_supply}
            </p>
          </div>

          {nft.collection_id && (
            <div className="rounded-xl border border-border bg-surface p-4">
              <p className="text-[10px] tracking-wide text-subtle uppercase">Collection</p>
              <div className="mt-1 flex items-center gap-2">
                <code className="break-all font-mono text-xs text-fg flex-1">{nft.collection_id}</code>
                <Link
                  to={`/wallet/collection/${nft.collection_id}`}
                  className="text-gold hover:underline text-sm"
                >
                  View Collection
                  <ExternalLink className="size-3.5 ml-1" />
                </Link>
              </div>
            </div>
          )}

          {nft.creator && (
            <div className="rounded-xl border border-border bg-surface p-4">
              <p className="text-[10px] tracking-wide text-subtle uppercase">Creator</p>
              <p className="mt-1 font-mono text-xs text-fg">{hexToKvnc(nft.creator)}</p>
            </div>
          )}

          {nft.owner_tx && (
            <div className="rounded-xl border border-border bg-surface p-4">
              <p className="text-[10px] tracking-wide text-subtle uppercase">Owner Transaction</p>
              <div className="mt-1 flex items-center gap-2">
                <code className="break-all font-mono text-xs text-fg flex-1">{nft.owner_tx}</code>
                <button
                  onClick={() => copyToClipboard(nft.owner_tx, "Transaction ID")}
                  className="text-muted hover:text-fg"
                  aria-label="Copy transaction ID"
                >
                  <Copy className="size-4" />
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

