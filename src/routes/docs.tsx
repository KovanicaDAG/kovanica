import { createFileRoute, lazyRouteComponent } from "@tanstack/react-router";
import { Shell } from "@/components/layout/shell";
import { RouteLoading } from "@/components/layout/route-loading";
import { RouteError } from "@/components/layout/route-error";

const LazyDocsView = lazyRouteComponent(
  () => import("@/components/docs/docs-view"),
  "DocsView",
);

export const Route = createFileRoute("/docs")({
  component: DocsPage,
  pendingComponent: () => (
    <Shell>
      <RouteLoading message="Loading Documentation..." />
    </Shell>
  ),
  errorComponent: ({ error, reset }) => (
    <Shell>
      <RouteError error={error} reset={reset} title="Documentation unavailable" />
    </Shell>
  ),
});

function DocsPage() {
  return (
    <Shell>
      <LazyDocsView />
    </Shell>
  );
}
