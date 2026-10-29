import { Download, Smartphone, Cpu, MessageSquare, Shield } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export const APP_DOWNLOADS = {
  wallet: {
    name: "Kovanica Wallet",
    description: "Non-custodial mobile wallet with PRO-21 addresses, deterministic key derivation, and instant KUNA transfers.",
    androidApk: "/download/kovanica-wallet-0.3.5.apk",
    releases: "https://github.com/KovanicaDAG/kovanica/releases/tag/v0.3.5-wallet",
    repo: "https://github.com/KovanicaDAG/kovanica/tree/main/apps/android-wallet",
    tag: "v0.3.5 · Android",
  },
  lightnode: {
    name: "Kovanica Lightnode",
    description: "Embedded DAG light client. Verifies Mysticeti DAG consensus headers, tracks account balances locally, and syncs trustlessly.",
    androidApk: "/download/kovanica-lightnode-0.8.6.apk",
    releases: "https://github.com/KovanicaDAG/kovanica/releases/tag/v0.8.6-android",
    repo: "https://github.com/KovanicaDAG/kovanica/tree/main/apps/android-chain",
    tag: "v0.8.6 · Android",
  },
  comms: {
    name: "Kovanica Comms",
    description: "Decentralized P2P messaging and encrypted peer communication powered by Kovanica DAG transport and cryptographic identity.",
    androidApk: "/download/kovanica-comms-0.2.0.apk",
    releases: "https://github.com/KovanicaDAG/kovanica/releases/tag/v0.2.0-comms",
    repo: "https://github.com/KovanicaDAG/kovanica/tree/main/apps/android-comms",
    tag: "v0.2.0 · Android",
  },
} as const;

type Props = {
  className?: string;
  variant?: "card" | "strip";
};

export function WalletDownloads({ className, variant = "card" }: Props) {
  const isCard = variant === "card";

  if (!isCard) {
    return (
      <section
        className={cn(
          "rounded-lg border border-border/80 bg-surface/80 p-3",
          className,
        )}
        aria-label="Download native applications"
      >
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex items-center gap-2">
            <Smartphone className="size-4 text-gold" aria-hidden />
            <span className="text-xs font-medium text-fg">Native Apps (APK):</span>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Button asChild size="sm" variant="outline" className="h-7 text-xs">
              <a href={APP_DOWNLOADS.wallet.androidApk} download>
                <Download className="mr-1.5 size-3.5 text-gold" />
                Wallet APK
              </a>
            </Button>
            <Button asChild size="sm" variant="outline" className="h-7 text-xs">
              <a href={APP_DOWNLOADS.lightnode.androidApk} download>
                <Download className="mr-1.5 size-3.5 text-teal" />
                Lightnode APK
              </a>
            </Button>
            <Button asChild size="sm" variant="outline" className="h-7 text-xs">
              <a href={APP_DOWNLOADS.comms.androidApk} download>
                <Download className="mr-1.5 size-3.5 text-blue" />
                Comms APK
              </a>
            </Button>
          </div>
        </div>
      </section>
    );
  }

  return (
    <section
      className={cn("mt-8 space-y-4", className)}
      aria-label="Kovanica native ecosystem applications"
    >
      <div className="flex flex-col sm:flex-row sm:items-baseline sm:justify-between gap-1">
        <div>
          <h2 className="text-lg font-semibold tracking-tight text-fg">
            Native Client Suite
          </h2>
          <p className="text-xs text-muted">
            Direct sideload builds for Android & iOS — connected to Chain 4 Testnet.
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Wallet */}
        <div className="flex flex-col justify-between rounded-xl border border-border/80 bg-surface/90 p-4 transition-colors hover:border-gold/40">
          <div>
            <div className="flex items-center justify-between">
              <div className="flex size-9 items-center justify-center rounded-lg bg-gold/10 text-gold">
                <Smartphone className="size-4" aria-hidden />
              </div>
              <span className="rounded bg-surface-2 px-1.5 py-0.5 font-mono text-[10px] text-muted">
                {APP_DOWNLOADS.wallet.tag}
              </span>
            </div>
            <h3 className="mt-3 font-medium text-fg text-sm">
              {APP_DOWNLOADS.wallet.name}
            </h3>
            <p className="mt-1 text-xs text-muted leading-relaxed">
              {APP_DOWNLOADS.wallet.description}
            </p>
          </div>

          <div className="mt-4 pt-3 border-t border-border/50 space-y-2">
            <div className="grid grid-cols-2 gap-2">
              <Button asChild size="sm" variant="outline" className="h-8 text-xs font-normal justify-center">
                <a href={APP_DOWNLOADS.wallet.androidApk} download>
                  <Download className="mr-1 size-3 text-gold" />
                  Android APK
                </a>
              </Button>
              <Button asChild size="sm" variant="outline" className="h-8 text-xs font-normal justify-center">
                <a
                  href={APP_DOWNLOADS.wallet.releases}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Shield className="mr-1 size-3 text-gold/80" />
                  Release notes
                </a>
              </Button>
            </div>
            <p className="text-[10px] text-subtle text-center">
              Signed with the Kovanica release key · arm64-v8a
            </p>
          </div>
        </div>

        {/* Lightnode */}
        <div className="flex flex-col justify-between rounded-xl border border-border/80 bg-surface/90 p-4 transition-colors hover:border-teal/40">
          <div>
            <div className="flex items-center justify-between">
              <div className="flex size-9 items-center justify-center rounded-lg bg-teal/10 text-teal">
                <Cpu className="size-4" aria-hidden />
              </div>
              <span className="rounded bg-surface-2 px-1.5 py-0.5 font-mono text-[10px] text-muted">
                {APP_DOWNLOADS.lightnode.tag}
              </span>
            </div>
            <h3 className="mt-3 font-medium text-fg text-sm">
              {APP_DOWNLOADS.lightnode.name}
            </h3>
            <p className="mt-1 text-xs text-muted leading-relaxed">
              {APP_DOWNLOADS.lightnode.description}
            </p>
          </div>

          <div className="mt-4 pt-3 border-t border-border/50 space-y-2">
            <div className="grid grid-cols-2 gap-2">
              <Button asChild size="sm" variant="outline" className="h-8 text-xs font-normal justify-center">
                <a href={APP_DOWNLOADS.lightnode.androidApk} download>
                  <Download className="mr-1 size-3 text-teal" />
                  Android APK
                </a>
              </Button>
              <Button asChild size="sm" variant="outline" className="h-8 text-xs font-normal justify-center">
                <a
                  href={APP_DOWNLOADS.lightnode.releases}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Shield className="mr-1 size-3 text-teal" />
                  Release notes
                </a>
              </Button>
            </div>
            <p className="text-[10px] text-subtle text-center">
              Embedded P2P light client for mobile & desktop
            </p>
          </div>
        </div>

        {/* Comms */}
        <div className="flex flex-col justify-between rounded-xl border border-border/80 bg-surface/90 p-4 transition-colors hover:border-blue/40">
          <div>
            <div className="flex items-center justify-between">
              <div className="flex size-9 items-center justify-center rounded-lg bg-blue/10 text-blue">
                <MessageSquare className="size-4" aria-hidden />
              </div>
              <span className="rounded bg-surface-2 px-1.5 py-0.5 font-mono text-[10px] text-muted">
                {APP_DOWNLOADS.comms.tag}
              </span>
            </div>
            <h3 className="mt-3 font-medium text-fg text-sm">
              {APP_DOWNLOADS.comms.name}
            </h3>
            <p className="mt-1 text-xs text-muted leading-relaxed">
              {APP_DOWNLOADS.comms.description}
            </p>
          </div>

          <div className="mt-4 pt-3 border-t border-border/50 space-y-2">
            <div className="grid grid-cols-2 gap-2">
              <Button asChild size="sm" variant="outline" className="h-8 text-xs font-normal justify-center">
                <a href={APP_DOWNLOADS.comms.androidApk} download>
                  <Download className="mr-1 size-3 text-blue" />
                  Android APK
                </a>
              </Button>
              <Button asChild size="sm" variant="outline" className="h-8 text-xs font-normal justify-center">
                <a
                  href={APP_DOWNLOADS.comms.repo}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Shield className="mr-1 size-3 text-blue" />
                  Source
                </a>
              </Button>
            </div>
            <p className="text-[10px] text-subtle text-center">
              P2P encrypted messaging over DAG transport
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
