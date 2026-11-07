import { createFileRoute, lazyRouteComponent, Outlet, useMatches } from "@tanstack/react-router";
import { Shell } from "@/components/layout/shell";
import { RouteLoading } from "@/components/layout/route-loading";
import { RouteError } from "@/components/layout/route-error";

const LazyExplorerView = lazyRouteComponent(
  () => import("@/components/explorer/explorer-view"),
  "ExplorerView",
);

export const Route = createFileRoute("/explorer")({
  component: ExplorerPage,
  pendingComponent: () => (
    <Shell>
      <RouteLoading message="Loading BlockDAG Explorer..." />
    </Shell>
  ),
  errorComponent: ({ error, reset }) => (
    <Shell>
      <RouteError error={error} reset={reset} title="Explorer unavailable" />
    </Shell>
  ),
});

function ExplorerPage() {
  const matches = useMatches();
  const isExact = matches[matches.length - 1]?.routeId === Route.id;
  if (!isExact) {
    return <Outlet />;
  }
  return (
    <Shell>
      <LazyExplorerView />
    </Shell>
  );
}
