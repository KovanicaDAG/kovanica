import { createFileRoute, lazyRouteComponent } from "@tanstack/react-router";
import { Shell } from "@/components/layout/shell";
import { RouteLoading } from "@/components/layout/route-loading";
import { RouteError } from "@/components/layout/route-error";

const LazyWalletView = lazyRouteComponent(
  () => import("@/components/wallet/wallet-view"),
  "WalletView",
);

export const Route = createFileRoute("/wallet")({
  component: WalletPage,
  pendingComponent: () => (
    <Shell>
      <RouteLoading message="Loading Kovanica Wallet..." />
    </Shell>
  ),
  errorComponent: ({ error, reset }) => (
    <Shell>
      <RouteError error={error} reset={reset} title="Wallet unavailable" />
    </Shell>
  ),
});

function WalletPage() {
  return (
    <Shell>
      <LazyWalletView />
    </Shell>
  );
}
