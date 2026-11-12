import { Smartphone, Cpu, MessageSquare } from "lucide-react";
import { cn } from "@/lib/utils";

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
        <div className="flex items-center gap-2">
          <Smartphone className="size-4 text-gold" aria-hidden />
          <span className="text-xs font-medium text-fg">
            Native Android builds are not currently published.
          </span>
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
            Native Clients
          </h2>
          <p className="text-xs text-muted">
            Android builds of the Wallet, Lightnode, and Comms apps are not
            currently published.
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
                Not published
              </span>
            </div>
            <h3 className="mt-3 font-medium text-fg text-sm">
              Kovanica Wallet
            </h3>
            <p className="mt-1 text-xs text-muted leading-relaxed">
              Non-custodial mobile wallet with PRO-21 addresses, deterministic
              key derivation, and instant KVNC transfers.
            </p>
          </div>

          <div className="mt-4 pt-3 border-t border-border/50">
            <p className="text-[10px] text-subtle">
              The previously published Android APK was withdrawn.
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
                Not published
              </span>
            </div>
            <h3 className="mt-3 font-medium text-fg text-sm">
              Kovanica Lightnode
            </h3>
            <p className="mt-1 text-xs text-muted leading-relaxed">
              Embedded DAG light client. Tracks DAG headers, checks account
              balances locally, and connects trustlessly.
            </p>
          </div>

          <div className="mt-4 pt-3 border-t border-border/50">
            <p className="text-[10px] text-subtle">
              The previously published Android APK was withdrawn.
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
                Not published
              </span>
            </div>
            <h3 className="mt-3 font-medium text-fg text-sm">
              Kovanica Comms
            </h3>
            <p className="mt-1 text-xs text-muted leading-relaxed">
              Decentralized P2P messaging and encrypted peer communication
              powered by Kovanica DAG transport and cryptographic identity.
            </p>
          </div>

          <div className="mt-4 pt-3 border-t border-border/50">
            <p className="text-[10px] text-subtle">
              The previously published Android APK was withdrawn; it is not
              being rebuilt right now.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
