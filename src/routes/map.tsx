import { createFileRoute, lazyRouteComponent } from "@tanstack/react-router";
import { Shell } from "@/components/layout/shell";
import { RouteLoading } from "@/components/layout/route-loading";
import { RouteError } from "@/components/layout/route-error";

const LazyOriginsDashboard = lazyRouteComponent(
  () => import("@/components/map/dashboard"),
  "OriginsDashboard",
);

export const Route = createFileRoute("/map")({
  component: MapPage,
  pendingComponent: () => (
    <Shell>
      <RouteLoading message="Loading Origins Map..." />
    </Shell>
  ),
  errorComponent: ({ error, reset }) => (
    <Shell>
      <RouteError error={error} reset={reset} title="Origins Map unavailable" />
    </Shell>
  ),
});

function MapPage() {
  return (
    <Shell>
      <LazyOriginsDashboard />
    </Shell>
  );
}
