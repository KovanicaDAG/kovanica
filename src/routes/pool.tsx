import { createFileRoute } from "@tanstack/react-router";
import { Shell } from "@/components/layout/shell";
import { PoolView } from "@/components/protocol/pool-view";

function PoolPage() {
  return (
    <Shell>
      <PoolView />
    </Shell>
  );
}

export const Route = createFileRoute("/pool")({ component: PoolPage });
