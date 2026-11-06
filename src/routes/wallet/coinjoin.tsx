import { createFileRoute, lazyRouteComponent } from "@tanstack/react-router";
import { Shell } from "@/components/layout/shell";
import { RouteLoading } from "@/components/layout/route-loading";
import { RouteError } from "@/components/layout/route-error";

const LazyCoinJoinView = lazyRouteComponent(
  () => import("@/components/wallet/coinjoin-view"),
  "CoinJoinView",
);

export const Route = createFileRoute("/wallet/coinjoin")({
  component: CoinJoinPage,
  pendingComponent: () => (
    <Shell>
      <RouteLoading message="Loading CoinJoin..." />
    </Shell>
  ),
  errorComponent: ({ error, reset }) => (
    <Shell>
      <RouteError error={error} reset={reset} title="CoinJoin unavailable" />
    </Shell>
  ),
});

function CoinJoinPage() {
  return (
    <Shell>
      <LazyCoinJoinView />
    </Shell>
  );
}
