import { createFileRoute, lazyRouteComponent } from "@tanstack/react-router";
import { RouteLoading } from "@/components/layout/route-loading";
import { RouteError } from "@/components/layout/route-error";

const LazyTokenListingsView = lazyRouteComponent(
  () => import("@/components/wallet/token-listings-view"),
  "TokenListingsView",
);

export const Route = createFileRoute("/wallet/token-listings")({
  component: LazyTokenListingsView,
  pendingComponent: () => <RouteLoading message="Loading Token Listings..." />,
  errorComponent: ({ error, reset }) => (
    <RouteError error={error} reset={reset} title="Token Listings unavailable" />
  ),
});
