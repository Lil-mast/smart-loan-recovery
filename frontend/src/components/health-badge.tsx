import type { HealthBand } from "@/lib/types";
import { bandLabel } from "@/lib/format";

const STYLES: Record<string, string> = {
  healthy: "bg-emerald-100 text-emerald-800",
  watch: "bg-amber-100 text-amber-800",
  at_risk: "bg-orange-100 text-orange-800",
  critical: "bg-red-100 text-red-800",
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
