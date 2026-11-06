import { Loader2 } from "lucide-react";

export function RouteLoading({
  message = "Loading...",
  className = "",
}: {
  message?: string;
  className?: string;
}) {
  return (
    <div
      role="status"
      aria-live="polite"
      aria-busy="true"
      className={`flex min-h-[40vh] flex-1 flex-col items-center justify-center gap-3 p-8 text-center text-muted ${className}`}
    >
      <Loader2 className="size-7 animate-spin text-accent" aria-hidden="true" />
      <p className="font-mono text-xs text-subtle">{message}</p>
      <span className="sr-only">Loading page content, please wait...</span>
    </div>
  );
}
