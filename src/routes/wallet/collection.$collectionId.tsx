import { createFileRoute, Link, notFound } from "@tanstack/react-router";
import { ArrowLeft as ArrowLeftIcon, Sparkles as SparklesIcon } from "lucide-react";
import { api, ApiError, HttpError } from "@/lib/api/client";
import { shortId } from "@/lib/ledger/hash";
import { hexToKvnc } from "@/lib/wallet/address";
import { Shell } from "@/components/layout/shell";
import { RouteError } from "@/components/layout/route-error";
import { Button } from "@/components/ui/button";

interface CollectionDetailResponse {
  collection_id: string;
  assets: Array<{
    asset_id: string;
    metadata_hash: string | null;
    owner_address: string | null;
  }>;
}

export const Route = createFileRoute("/wallet/collection/$collectionId")({
  loader: async ({ params }) => {
    try {
      const data = await api<CollectionDetailResponse>(`/api/collection/${encodeURIComponent(params.collectionId)}`);
      if (!data || !data.collection_id) {
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
  notFoundComponent: () => <CollectionNotFound />,
  errorComponent: ({ error, reset }) => (
    <Shell>
      <RouteError
        title="Collection Endpoint Unavailable"
        error={error}
        reset={reset}
      />
    </Shell>
  ),
  component: CollectionDetailPage,
});

function CollectionNotFound() {
  const { collectionId } = Route.useParams();
  return (
    <Shell>
      <div className="mx-auto flex min-h-[40vh] max-w-md flex-1 flex-col items-center justify-center gap-4 p-8 text-center">
        <div className="flex size-12 items-center justify-center rounded-full bg-purple-500/10 text-purple-400">
          <SparklesIcon className="size-6" aria-hidden="true" />
        </div>
        <div className="space-y-1">
          <h2 className="font-display text-lg font-medium text-fg">Collection Not Found</h2>
          <p className="font-mono text-xs text-muted break-words">
            No collection was found with ID &ldquo;{collectionId}&rdquo; on this network.
          </p>
        </div>
        <Button asChild variant="outline" size="sm" className="mt-2">
          <Link to="/wallet">Back to Wallet</Link>
        </Button>
      </div>
    </Shell>
  );
}
  );
}

function CollectionDetailPage() {
  const collection = Route.useLoaderData();

  return (
    <Shell>
      <div className="mx-auto max-w-5xl px-4 py-6 md:px-6 md:py-8">
      <div className="mb-6 flex items-center gap-3">
        <Link
          to="/wallet"
          className="text-muted hover:text-fg transition-colors"
          aria-label="Back to wallet"
        >
          <ArrowLeftIcon className="size-5" />
        </Link>
        <div>
          <p className="font-mono text-[10px] tracking-wide text-purple uppercase">
            KVP-106 Collection
          </p>
          <h1 className="font-display text-2xl tracking-tight text-fg">
            Collection {shortId(collection.collection_id)}
            <span className="ml-2 rounded-full bg-purple/20 px-2 py-0.5 font-mono text-[10px] text-purple">
              {collection.assets.length} NFTs
            </span>
          </h1>
        </div>
      </div>

      {collection.assets.length === 0 ? (
        <div className="rounded-xl border border-border bg-surface p-8 text-center">
          <AlertCircle className="size-12 mx-auto text-amber" />
          <p className="mt-4 text-sm text-muted">No NFTs found in this collection</p>
        </div>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {collection.assets.map((asset) => (
            <Link
              key={asset.asset_id}
              to="/wallet/nft/$assetId"
              params={{ assetId: asset.asset_id }}
              className="group rounded-xl border border-border bg-surface overflow-hidden transition-all hover:border-purple/50 hover:shadow-lg"
            >
              <div className="aspect-square relative bg-surface overflow-hidden">
                <div className="absolute inset-0 flex items-center justify-center">
                  <SparklesIcon className="size-12 text-purple/30 group-hover:text-purple/50 transition-colors" />
                </div>
                <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/70 to-transparent p-3">
                  <p className="font-mono text-xs text-white truncate">{shortId(asset.asset_id)}</p>
                </div>
              </div>
              <div className="p-3">
                <p className="font-mono text-[10px] text-subtle truncate">{asset.asset_id}</p>
                {asset.owner_address && (
                  <p className="mt-1 font-mono text-[10px] text-muted truncate">
                    Owner: {hexToKvnc(asset.owner_address)}
                  </p>
                )}
                {asset.metadata_hash && (
                  <p className="mt-1 font-mono text-[9px] text-subtle truncate">
                    Meta: {asset.metadata_hash.slice(0, 12)}…
                  </p>
                )}
              </div>
            </Link>
          ))}
        </div>
      )}
      </div>
    </Shell>
  );
}
