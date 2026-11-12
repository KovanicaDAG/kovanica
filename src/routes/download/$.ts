import { createFileRoute } from "@tanstack/react-router";

// /download/* - redirect to the canonical artifact locations:
//   install.sh            -> raw.githubusercontent (monorepo, always current)
//   kovanica-node-linux-* -> GitHub Releases, v<version>-node tag (see
//                            .github/workflows/release-node.yml)
// Redirects (302) keep binary traffic off the Nitro server.

const RAW = "https://raw.githubusercontent.com/KovanicaDAG/kovanica/main";

// Bump NODE_RELEASE together with the default VERSION in
// node/scripts/install.sh.
const NODE_RELEASE =
  "https://github.com/KovanicaDAG/kovanica/releases/download/v0.2.0-node";

const DOWNLOADS: Record<string, string> = {
  "install.sh": `${RAW}/node/scripts/install.sh`,
  "kovanica-node-linux-x64": `${NODE_RELEASE}/kovanica-node-x86_64-linux.tar.gz`,
  "kovanica-node-linux-arm64": `${NODE_RELEASE}/kovanica-node-aarch64-linux.tar.gz`,
  "kovanica-chain4-genesis.tar.gz": `${NODE_RELEASE}/kovanica-chain4-genesis.tar.gz`,
};

export const Route = createFileRoute("/download/$")({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const url = new URL(request.url);
        const name = url.pathname.replace(/^\/download\//, "");
        const target = DOWNLOADS[name];
        if (!target) {
          return new Response("not found", { status: 404 });
        }
        return Response.redirect(target, 302);
      },
    },
  },
});
