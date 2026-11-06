/**
 * Shell — full protocol chrome (testnet / shared hosts only).
 * Cleaned to strictly reflect crates and contracts present in the codebase:
 * Core: Explorer, Wallet, Faucet, Network
 * Contracts: Multisig, HTLC, Vaults, Pools (/pool)
 * Secondary: Map, Roadmap, Docs.
 */
import { useState } from "react";
import { Link, useRouterState } from "@tanstack/react-router";
import * as Dialog from "@radix-ui/react-dialog";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import {
  Compass,
  Map as MapIcon,
  Wallet,
  Coins,
  Users,
  FileText,
  Activity,
  Route,
  Lock,
  Vault,
  Droplets,
  Waves,
  ChevronDown,
  Menu,
  X,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { SourceSwitch } from "@/components/layout/source-switch";
import { NetworkBadge } from "@/components/layout/network-badge";
import { getBadgeColor } from "@/lib/network";
import { Button } from "@/components/ui/button";

const CORE_NAV = [
  { to: "/explorer", label: "Explorer", icon: Compass },
  { to: "/wallet", label: "Wallet", icon: Wallet },
  { to: "/faucet", label: "Faucet", icon: Droplets },
  { to: "/network", label: "Network", icon: Activity },
] as const;

const CONTRACT_NAV = [
  { to: "/pool", label: "Pools", icon: Waves, desc: "Constant-product liquidity pools" },
  { to: "/multisig", label: "Multisig", icon: Users, desc: "M-of-N shared custody" },
  { to: "/htlc", label: "HTLC", icon: Lock, desc: "Hash Time-Locked Contracts" },
  { to: "/vaults", label: "Vaults", icon: Vault, desc: "Time-locked vaults (CLTV / CSV)" },
] as const;

const SECONDARY_NAV = [
  { to: "/map", label: "Map", icon: MapIcon },
  { to: "/roadmap", label: "Roadmap", icon: Route },
  { to: "/docs", label: "Docs", icon: FileText },
] as const;

export function Shell({ children }: { children: React.ReactNode }) {
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  const [mobileOpen, setMobileOpen] = useState(false);

  const isItemActive = (to: string) =>
    to === "/" ? pathname === "/" : pathname === to || pathname.startsWith(`${to}/`);

  const isContractsActive = CONTRACT_NAV.some((item) => isItemActive(item.to));
  const isSecondaryActive = SECONDARY_NAV.some((item) => isItemActive(item.to));

  return (
    <div className="flex min-h-dvh flex-col bg-bg text-fg">
      <header
        className="sticky top-0 z-30 flex h-14 shrink-0 items-center justify-between gap-3 border-b border-border bg-bg/95 px-4 backdrop-blur-sm md:h-16 md:px-6"
        style={{ borderTop: `1px solid ${getBadgeColor()}` }}
      >
        <Link to="/" className="flex min-w-0 items-baseline gap-2">
          <span className="font-display text-xl tracking-tight text-fg italic md:text-2xl">
            Kovanica
          </span>
          <span className="font-mono text-[10px] tracking-brand text-blue uppercase md:text-xs">
            Protocol
          </span>
        </Link>

        {/* Primary nav: active for tablet (md) and desktop (lg/xl) */}
        <nav className="hidden items-center gap-1 xl:gap-1.5 md:flex" aria-label="Primary">
          {/* Core Navigation */}
          {CORE_NAV.map((item) => {
            const on = isItemActive(item.to);
            return (
              <Link
                key={item.to}
                to={item.to}
                aria-current={on ? "page" : undefined}
                className={cn(
                  "inline-flex h-9 items-center rounded-md px-2.5 text-sm font-medium transition-colors duration-150",
                  on
                    ? "border-b-2 border-gold bg-surface-2 font-semibold text-fg shadow-xs"
                    : "text-muted hover:bg-surface-2 hover:text-fg",
                )}
              >
                {item.label}
              </Link>
            );
          })}

          {/* Contracts Dropdown */}
          <DropdownMenu.Root>
            <DropdownMenu.Trigger asChild>
              <button
                type="button"
                aria-current={isContractsActive ? "page" : undefined}
                className={cn(
                  "inline-flex h-9 items-center gap-1 rounded-md px-2.5 text-sm font-medium transition-colors duration-150 outline-none",
                  isContractsActive
                    ? "border-b-2 border-gold bg-surface-2 font-semibold text-fg shadow-xs"
                    : "text-muted hover:bg-surface-2 hover:text-fg",
                )}
              >
                <span>Contracts</span>
                <ChevronDown className="size-3.5 opacity-70" />
              </button>
            </DropdownMenu.Trigger>
            <DropdownMenu.Portal>
              <DropdownMenu.Content
                align="start"
                sideOffset={8}
                className="z-50 min-w-52 rounded-xl border border-border bg-surface p-1.5 shadow-xl focus:outline-none"
              >
                <div className="px-2.5 py-1 text-[10px] font-mono uppercase tracking-wider text-subtle">
                  Smart Contracts
                </div>
                {CONTRACT_NAV.map((item) => {
                  const on = isItemActive(item.to);
                  const Icon = item.icon;
                  return (
                    <DropdownMenu.Item key={item.to} asChild>
                      <Link
                        to={item.to}
                        aria-current={on ? "page" : undefined}
                        className={cn(
                          "flex items-center gap-2.5 rounded-lg px-2.5 py-2 text-xs font-medium outline-none transition-colors",
                          on
                            ? "bg-surface-2 font-semibold text-fg"
                            : "text-muted hover:bg-surface-2 hover:text-fg",
                        )}
                      >
                        <Icon className="size-4 shrink-0 text-blue" />
                        <div className="flex flex-col">
                          <span className="text-fg">{item.label}</span>
                          <span className="text-[10px] font-normal text-muted">{item.desc}</span>
                        </div>
                      </Link>
                    </DropdownMenu.Item>
                  );
                })}
              </DropdownMenu.Content>
            </DropdownMenu.Portal>
          </DropdownMenu.Root>

          {/* Secondary links */}
          <Link
            to="/map"
            aria-current={isItemActive("/map") ? "page" : undefined}
            className={cn(
              "hidden xl:inline-flex h-9 items-center rounded-md px-2.5 text-sm font-medium transition-colors duration-150",
              isItemActive("/map")
                ? "border-b-2 border-gold bg-surface-2 font-semibold text-fg shadow-xs"
                : "text-muted hover:bg-surface-2 hover:text-fg",
            )}
          >
            Map
          </Link>
          <Link
            to="/roadmap"
            aria-current={isItemActive("/roadmap") ? "page" : undefined}
            className={cn(
              "hidden lg:inline-flex h-9 items-center rounded-md px-2.5 text-sm font-medium transition-colors duration-150",
              isItemActive("/roadmap")
                ? "border-b-2 border-gold bg-surface-2 font-semibold text-fg shadow-xs"
                : "text-muted hover:bg-surface-2 hover:text-fg",
            )}
          >
            Roadmap
          </Link>
          <Link
            to="/docs"
            aria-current={isItemActive("/docs") ? "page" : undefined}
            className={cn(
              "inline-flex h-9 items-center rounded-md px-2.5 text-sm font-medium transition-colors duration-150",
              isItemActive("/docs")
                ? "border-b-2 border-gold bg-surface-2 font-semibold text-fg shadow-xs"
                : "text-muted hover:bg-surface-2 hover:text-fg",
            )}
          >
            Docs
          </Link>

          {/* Tablet (md to lg) More dropdown so Map and Roadmap are reachable */}
          <div className="lg:hidden">
            <DropdownMenu.Root>
              <DropdownMenu.Trigger asChild>
                <button
                  type="button"
                  aria-label="More navigation links"
                  className={cn(
                    "inline-flex h-9 items-center gap-1 rounded-md px-2 text-sm font-medium transition-colors outline-none",
                    isItemActive("/map") || isItemActive("/roadmap")
                      ? "border-b-2 border-gold bg-surface-2 font-semibold text-fg shadow-xs"
                      : "text-muted hover:bg-surface-2 hover:text-fg",
                  )}
                >
                  <span>More</span>
                  <ChevronDown className="size-3.5 opacity-70" />
                </button>
              </DropdownMenu.Trigger>
              <DropdownMenu.Portal>
                <DropdownMenu.Content
                  align="start"
                  sideOffset={8}
                  className="z-50 min-w-44 rounded-xl border border-border bg-surface p-1.5 shadow-xl focus:outline-none"
                >
                  <DropdownMenu.Item asChild>
                    <Link
                      to="/roadmap"
                      aria-current={isItemActive("/roadmap") ? "page" : undefined}
                      className={cn(
                        "flex items-center gap-2 rounded-lg px-2.5 py-2 text-xs font-medium outline-none transition-colors",
                        isItemActive("/roadmap") ? "bg-surface-2 font-semibold text-fg" : "text-muted hover:bg-surface-2 hover:text-fg",
                      )}
                    >
                      <Route className="size-4 text-teal" />
                      <span>Roadmap</span>
                    </Link>
                  </DropdownMenu.Item>
                  <DropdownMenu.Item asChild>
                    <Link
                      to="/map"
                      aria-current={isItemActive("/map") ? "page" : undefined}
                      className={cn(
                        "flex items-center gap-2 rounded-lg px-2.5 py-2 text-xs font-medium outline-none transition-colors",
                        isItemActive("/map") ? "bg-surface-2 font-semibold text-fg" : "text-muted hover:bg-surface-2 hover:text-fg",
                      )}
                    >
                      <MapIcon className="size-4 text-blue" />
                      <span>Map</span>
                    </Link>
                  </DropdownMenu.Item>
                </DropdownMenu.Content>
              </DropdownMenu.Portal>
            </DropdownMenu.Root>
          </div>
        </nav>

        {/* Header right: Network badges & source switches */}
        <div className="flex items-center gap-1.5">
          <span className="hidden md:inline-flex">
            <NetworkBadge />
          </span>
          <span className="md:hidden">
            <SourceSwitch compact />
          </span>
          <span className="hidden md:inline-flex">
            <SourceSwitch />
          </span>
          <Link
            to="/roadmap"
            className={cn(
              "inline-flex size-10 items-center justify-center rounded-md md:hidden",
              isItemActive("/roadmap") ? "text-fg" : "text-muted",
            )}
            aria-label="Roadmap"
          >
            <Route className="size-5" />
          </Link>
          <Link
            to="/docs"
            className={cn(
              "inline-flex size-10 items-center justify-center rounded-md md:hidden",
              isItemActive("/docs") ? "text-fg" : "text-muted",
            )}
            aria-label="Docs"
          >
            <FileText className="size-5" />
          </Link>
        </div>
      </header>

      <div className="flex min-h-0 flex-1 flex-col pb-[calc(4.25rem+env(safe-area-inset-bottom))] md:pb-0">
        {children}
      </div>

      {/* Mobile bottom navigation: exactly 5 slots with accessible More sheet */}
      <nav
        aria-label="Mobile"
        className="fixed inset-x-0 bottom-0 z-30 border-t border-border bg-bg/95 pb-[env(safe-area-inset-bottom)] backdrop-blur-sm md:hidden"
      >
        <ul className="grid grid-cols-5">
          {CORE_NAV.map((item) => {
            const Icon = item.icon;
            const on = isItemActive(item.to);
            return (
              <li key={item.to}>
                <Link
                  to={item.to}
                  aria-current={on ? "page" : undefined}
                  className={cn(
                    "flex min-h-14 flex-col items-center justify-center gap-0.5 px-0.5 text-[10px] font-medium leading-tight",
                    on ? "font-semibold text-fg" : "text-muted",
                  )}
                >
                  <Icon className="size-5 shrink-0" strokeWidth={on ? 2.2 : 1.8} />
                  <span className="truncate max-w-full">{item.label}</span>
                </Link>
              </li>
            );
          })}
          <li>
            <Dialog.Root open={mobileOpen} onOpenChange={setMobileOpen}>
              <Dialog.Trigger asChild>
                <button
                  type="button"
                  aria-expanded={mobileOpen}
                  aria-label="More navigation options"
                  className={cn(
                    "flex h-full w-full min-h-14 flex-col items-center justify-center gap-0.5 px-0.5 text-[10px] font-medium leading-tight",
                    isContractsActive || isSecondaryActive || pathname === "/"
                      ? "font-semibold text-fg"
                      : "text-muted",
                  )}
                >
                  <Menu
                    className="size-5 shrink-0"
                    strokeWidth={isContractsActive || isSecondaryActive ? 2.2 : 1.8}
                  />
                  <span className="truncate max-w-full">More</span>
                </button>
              </Dialog.Trigger>
              <Dialog.Portal>
                <Dialog.Overlay className="fixed inset-0 z-40 bg-black/60 backdrop-blur-xs animate-in fade-in" />
                <Dialog.Content className="fixed inset-x-0 bottom-0 z-50 max-h-[85vh] overflow-y-auto rounded-t-2xl border-t border-border bg-surface p-5 shadow-2xl focus:outline-none">
                  <div className="flex items-center justify-between border-b border-border pb-3">
                    <Dialog.Title className="font-display text-lg tracking-tight text-fg italic">
                      Kovanica Navigation
                    </Dialog.Title>
                    <Dialog.Close asChild>
                      <Button variant="ghost" size="icon" aria-label="Close navigation sheet">
                        <X className="size-4" />
                      </Button>
                    </Dialog.Close>
                  </div>

                  {/* Contracts group */}
                  <div className="mt-4">
                    <p className="text-[10px] font-mono uppercase tracking-wider text-subtle">
                      Contracts
                    </p>
                    <div className="mt-2 grid grid-cols-1 gap-1.5">
                      {CONTRACT_NAV.map((item) => {
                        const Icon = item.icon;
                        const on = isItemActive(item.to);
                        return (
                          <Link
                            key={item.to}
                            to={item.to}
                            onClick={() => setMobileOpen(false)}
                            aria-current={on ? "page" : undefined}
                            className={cn(
                              "flex items-center justify-between rounded-xl border border-border/60 bg-bg/50 p-3 transition-colors",
                              on
                                ? "border-gold/50 bg-surface-2 font-medium text-fg"
                                : "text-muted hover:text-fg",
                            )}
                          >
                            <div className="flex items-center gap-3">
                              <div className="flex size-8 items-center justify-center rounded-lg bg-surface-2">
                                <Icon className="size-4 text-blue" />
                              </div>
                              <div>
                                <p className="text-sm font-medium text-fg">{item.label}</p>
                                <p className="text-xs text-muted">{item.desc}</p>
                              </div>
                            </div>
                          </Link>
                        );
                      })}
                    </div>
                  </div>

                  {/* Platform & Secondary */}
                  <div className="mt-5">
                    <p className="text-[10px] font-mono uppercase tracking-wider text-subtle">
                      Platform
                    </p>
                    <div className="mt-2 grid grid-cols-2 gap-2">
                      <Link
                        to="/"
                        onClick={() => setMobileOpen(false)}
                        aria-current={pathname === "/" ? "page" : undefined}
                        className={cn(
                          "flex items-center gap-2 rounded-lg border border-border/60 bg-bg/50 p-2.5 text-xs font-medium transition-colors",
                          pathname === "/"
                            ? "border-gold/50 bg-surface-2 font-medium text-fg"
                            : "text-muted hover:text-fg",
                        )}
                      >
                        <Coins className="size-4 text-gold" />
                        <span>Home</span>
                      </Link>
                      {SECONDARY_NAV.map((item) => {
                        const Icon = item.icon;
                        const on = isItemActive(item.to);
                        return (
                          <Link
                            key={item.to}
                            to={item.to}
                            onClick={() => setMobileOpen(false)}
                            aria-current={on ? "page" : undefined}
                            className={cn(
                              "flex items-center gap-2 rounded-lg border border-border/60 bg-bg/50 p-2.5 text-xs font-medium transition-colors",
                              on
                                ? "border-gold/50 bg-surface-2 font-medium text-fg"
                                : "text-muted hover:text-fg",
                            )}
                          >
                            <Icon className="size-4 text-teal" />
                            <span>{item.label}</span>
                          </Link>
                        );
                      })}
                    </div>
                  </div>

                  {/* Mobile Network info / source switch */}
                  <div className="mt-5 flex items-center justify-between border-t border-border pt-4">
                    <NetworkBadge />
                    <SourceSwitch />
                  </div>
                </Dialog.Content>
              </Dialog.Portal>
            </Dialog.Root>
          </li>
        </ul>
      </nav>
    </div>
  );
}
