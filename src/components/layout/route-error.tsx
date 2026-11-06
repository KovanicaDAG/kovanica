import { AlertCircle, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";

export function RouteError({
  error,
  reset,
  title = "Failed to load route",
}: {
  error?: unknown;
  reset?: () => void;
  title?: string;
}) {
  const message =
    error instanceof Error
      ? error.message
      : typeof error === "string"
        ? error
        : "An unexpected error occurred while loading this page.";

  return (
    <div
      role="alert"
      className="mx-auto flex min-h-[40vh] max-w-md flex-1 flex-col items-center justify-center gap-4 p-8 text-center"
    >
      <div className="flex size-12 items-center justify-center rounded-full bg-red-500/10 text-red-400">
        <AlertCircle className="size-6" aria-hidden="true" />
      </div>
      <div className="space-y-1">
        <h2 className="font-display text-lg font-medium text-fg">{title}</h2>
        <p className="font-mono text-xs text-muted break-words">{message}</p>
      </div>
      {reset && (
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => reset()}
          className="mt-2 inline-flex items-center gap-2"
        >
          <RefreshCw className="size-3.5" aria-hidden="true" />
          <span>Try again</span>
        </Button>
      )}
    </div>
  );
}
