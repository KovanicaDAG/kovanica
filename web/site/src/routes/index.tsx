import { createFileRoute } from "@tanstack/react-router";
import { Shell } from "@/components/layout/shell";
import { LandingChrome } from "@/components/layout/landing-chrome";
import { HomeLanding } from "@/components/landing/home";
import { getHost, hostRoleFromHost } from "@/lib/host";

export const Route = createFileRoute("/")({
  loader: async () => {
    let host = "kovanica.online";
    try {
      host = await getHost();
    } catch {
      /* apex fallback */
    }
    return { host, role: hostRoleFromHost(host) };
  },
  component: Home,
});

function Home() {
  const { role } = Route.useLoaderData();

  // Apex = pure project face (NETWORK.md §4). No Explorer/Wallet strip.
  if (role === "landing") {
    return (
      <LandingChrome>
        <HomeLanding />
      </LandingChrome>
    );
  }

  // testnet / shared hosts — full interactive chrome
  return (
    <Shell>
      <HomeLanding />
    </Shell>
  );
}
