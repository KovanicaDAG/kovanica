import { createFileRoute, Link, notFound } from "@tanstack/react-router";
import { AlertCircle, ArrowLeft, Copy, ExternalLink, Gem as GemIcon, Sparkles } from "lucide-react";
import { api, ApiError, HttpError } from "@/lib/api/client";
import { shortId } from "@/lib/ledger/hash";
import { hexToKvnc } from "@/lib/wallet/address";
import { Shell } from "@/components/layout/shell";
import { RouteError } from "@/components/layout/route-error";
import { Button } from "@/components/ui/button";

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

export const Route = createFileRoute("/wallet/nft/$assetId")({
  loader: async ({ params, location }) => {
    try {
      const searchSource = (location.search as Record<string, unknown>)?.source;
      const queryParam = typeof searchSource === "string" ? `?source=${encodeURIComponent(searchSource)}` : "";
      const data = await api<NftDetailResponse>(`/api/nft/${encodeURIComponent(params.assetId)}${queryParam}`);
      if (!data || !data.asset_id) {
        throw notFound();
      }
      return data;
    } catch (err: unknown) {
      if (
        (err instanceof ApiError && err.status === 404) ||
        (err instanceof HttpError && err.status === 404) ||
        (typeof err === "object" && err !== null && "status" in err && (err as { status: unknown }).status === 404) ||
        (err instanceof Error && err.message.toLowerCase().includes("not found"))
      ) {
        throw notFound();
      }
      throw err;
    }
  },
  notFoundComponent: NftNotFoundPage,
  errorComponent: NftErrorPage,
  component: NftDetailPage,
});

function NftNotFoundPage() {
  const { assetId } = Route.useParams();
  return (
    <Shell>
      <div className="mx-auto flex min-h-[40vh] max-w-md flex-1 flex-col items-center justify-center gap-4 p-8 text-center">
        <div className="flex size-12 items-center justify-center rounded-full bg-purple-500/10 text-purple-400">
          <GemIcon className="size-6" aria-hidden="true" />
        </div>
        <div className="space-y-1">
          <h2 className="font-display text-lg font-medium text-fg">NFT Not Found</h2>
          <p className="font-mono text-xs text-muted break-words">
            No NFT was found with asset ID &ldquo;{assetId}&rdquo; on this network.
          </p>
        </div>
        <Button asChild variant="outline" size="sm" className="mt-2">
          <Link to="/wallet">Back to Wallet</Link>
        </Button>
      </div>
    </Shell>
  );
}

function NftErrorPage({ error, reset }: { error?: unknown; reset?: () => void }) {
  return (
    <Shell>
      <RouteError
        title="NFT Endpoint Unavailable"
        error={error}
        reset={reset}
      />
    </Shell>
  );
}

function NftDetailPage() {
  const nft = Route.useLoaderData();

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const formatAddress = (addr: string | null) => {
    if (!addr) return "Unknown";
    return hexToKvnc(addr);
  };

  return (
    <Shell>
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
            NFT {shortId(nft.asset_id)}
          </h1>
        </div>
      </div>

      <div className="space-y-6">
        {/* NFT Image / Visual */}
        <div className="relative aspect-square rounded-xl border border-border bg-surface overflow-hidden">
          <div className="flex h-full items-center justify-center">
            <GemIcon className="size-16 text-purple/50" />
          </div>
          <div className="absolute top-3 right-3 flex gap-1">
            <span className="rounded-full bg-purple/90 px-2 py-0.5 font-mono text-[10px] text-white">
              NFT
            </span>
            {nft.collection_id && (
              <Link
                to="/wallet/collection/$collectionId"
                params={{ collectionId: nft.collection_id }}
                className="rounded-full bg-surface/90 px-2 py-0.5 font-mono text-[10px] text-fg hover:bg-surface"
                aria-label="View collection"
              >
                <Sparkles className="size-3" />
              </Link>
            )}
          </div>
        </div>

        {/* Metadata */}
        {nft.metadata_hash && (
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
                onClick={() => copyToClipboard(nft.asset_id)}
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
              <code className="break-all font-mono text-xs text-fg flex-1">
                {formatAddress(nft.owner)}
              </code>
              {nft.owner && (
                <button
                  onClick={() => copyToClipboard(nft.owner!)}
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
                <code className="break-all font-mono text-xs text-fg flex-1">
                  {nft.collection_id}
                </code>
                <Link
                  to="/wallet/collection/$collectionId"
                  params={{ collectionId: nft.collection_id }}
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
                  onClick={() => copyToClipboard(nft.owner_tx!)}
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
    </Shell>
  );
}
