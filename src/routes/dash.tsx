import { createFileRoute, lazyRouteComponent } from "@tanstack/react-router";
import { Shell } from "@/components/layout/shell";
import { RouteLoading } from "@/components/layout/route-loading";
import { RouteError } from "@/components/layout/route-error";

const LazyDashboard = lazyRouteComponent(
  () => import("@/components/dashboard/dashboard"),
  "Dashboard",
);

export const Route = createFileRoute("/dash")({
  component: DashPage,
  pendingComponent: () => (
    <Shell>
      <RouteLoading message="Loading Node Dashboard..." />
    </Shell>
  ),
  errorComponent: ({ error, reset }) => (
    <Shell>
      <RouteError error={error} reset={reset} title="Dashboard unavailable" />
    </Shell>
  ),
});

function DashPage() {
  return (
    <Shell>
      <LazyDashboard />
    </Shell>
  );
}
