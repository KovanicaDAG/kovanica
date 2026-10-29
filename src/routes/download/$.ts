import { createFileRoute } from "@tanstack/react-router";

// /download/* - redirect to the canonical artifact locations:
//   install.sh            -> raw.githubusercontent (monorepo, always current)
//   kovanica-node-linux-* -> GitHub Releases, v<version>-node tag (see
//                            .github/workflows/release-node.yml)
//   wallet artifacts      -> GitHub Releases (when published)
// Redirects (302) keep binary traffic off the Nitro server.

const RAW = "https://raw.githubusercontent.com/KovanicaDAG/kovanica/main";

// Node binaries ship on their own tag; `releases/latest` is the wallet APK.
// Bump NODE_RELEASE together with the default VERSION in
// node/scripts/install.sh.
const NODE_RELEASE =
  "https://github.com/KovanicaDAG/kovanica/releases/download/v0.2.0-node";

// The Android apps ship as three separate releases (one app per tag), so
// releases/latest/download only ever contains one of them. Pin the tag and
// bump these three URLs together with the version tags in wallet-downloads.tsx.
const ANDROID_RELEASE = {
  wallet:
    "https://github.com/KovanicaDAG/kovanica/releases/download/v0.3.4-wallet/Kovanica-Wallet-0.3.4-arm64-v8a.apk",
  lightnode:
    "https://github.com/KovanicaDAG/kovanica/releases/download/v0.8.5-android/Kovanica-Lightnode-0.8.5-arm64-v8a.apk",
  comms:
    "https://github.com/KovanicaDAG/kovanica/releases/download/v0.2.0-comms/kovanica-comms-0.2.0.apk",
};

// Older pinned paths keep working: their releases are never deleted, so a link
// cached by a CDN or pasted in a chat still resolves to the build it names.
const ANDROID_ARCHIVE: Record<string, string> = {
  "kovanica-wallet-0.3.3.apk":
    "https://github.com/KovanicaDAG/kovanica/releases/download/v0.3.3-wallet/Kovanica-Wallet-0.3.3-arm64-v8a.apk",
  "kovanica-lightnode-0.8.4.apk":
    "https://github.com/KovanicaDAG/kovanica/releases/download/v0.8.4-android/Kovanica-Lightnode-0.8.4-arm64-v8a.apk",
};

const DOWNLOADS: Record<string, string> = {
  ...ANDROID_ARCHIVE,
  "install.sh": `${RAW}/node/scripts/install.sh`,
  "kovanica-node-linux-x64": `${NODE_RELEASE}/kovanica-node-x86_64-linux.tar.gz`,
  "kovanica-node-linux-arm64": `${NODE_RELEASE}/kovanica-node-aarch64-linux.tar.gz`,
  "kovanica-chain4-genesis.tar.gz": `${NODE_RELEASE}/kovanica-chain4-genesis.tar.gz`,
  // Versioned aliases: the CDN caches 404s, so a new path per build keeps
  // downloads working the moment a release lands.
  "kovanica-wallet-0.3.4.apk": ANDROID_RELEASE.wallet,
  "kovanica-lightnode-0.8.5.apk": ANDROID_RELEASE.lightnode,
  "kovanica-comms-0.2.0.apk": ANDROID_RELEASE.comms,
  "kovanica-wallet-latest.apk": ANDROID_RELEASE.wallet,
  "kovanica-lightnode-latest.apk": ANDROID_RELEASE.lightnode,
  "kovanica-comms-latest.apk": ANDROID_RELEASE.comms,
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
