import { createFileRoute, lazyRouteComponent } from "@tanstack/react-router";
import { Shell } from "@/components/layout/shell";
import { RouteLoading } from "@/components/layout/route-loading";
import { RouteError } from "@/components/layout/route-error";

const LazyAtomicSwapView = lazyRouteComponent(
  () => import("@/components/wallet/atomic-swap-view"),
  "AtomicSwapView",
);

export const Route = createFileRoute("/wallet/atomic-swap")({
  component: AtomicSwapPage,
  pendingComponent: () => (
    <Shell>
      <RouteLoading message="Loading Atomic Swaps..." />
    </Shell>
  ),
  errorComponent: ({ error, reset }) => (
    <Shell>
      <RouteError error={error} reset={reset} title="Atomic Swaps unavailable" />
    </Shell>
  ),
});

function AtomicSwapPage() {
  return (
    <Shell>
      <LazyAtomicSwapView />
    </Shell>
  );
}
