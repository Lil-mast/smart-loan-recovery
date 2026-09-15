import type { HealthBand } from "@/lib/types";
import { bandLabel } from "@/lib/format";

const STYLES: Record<string, string> = {
  healthy: "bg-emerald-500/15 text-emerald-300",
  watch: "bg-amber-500/15 text-amber-200",
  at_risk: "bg-orange-500/15 text-orange-300",
  critical: "bg-red-500/15 text-red-300",
};

export function HealthBadge({ band, score }: { band: HealthBand | string; score?: number }) {
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-semibold ${STYLES[band] ?? "bg-card-high text-muted"}`}
    >
      <span className="h-1.5 w-1.5 rounded-full bg-current" />
      {bandLabel(band)}
      {typeof score === "number" ? <span className="font-mono opacity-80">{Math.round(score)}</span> : null}
    </span>
  );
}
