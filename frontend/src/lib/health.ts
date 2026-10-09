export function recLabel(key: string): string {
  if (key === "send_reminder") return "Send reminder";
  if (key === "renegotiate_terms") return "Renegotiate";
  if (key === "escalate_to_collection") return "Escalate";
  return key.replaceAll("_", " ");
}

export function healthLabel(band?: string): string {
  if (band === "healthy") return "Healthy";
  if (band === "watch") return "Watch";
  if (band === "at_risk") return "At risk";
  if (band === "critical") return "Critical";
  return "Unknown";
}

export function healthClass(band?: string): string {
  if (band === "healthy") return "bg-emerald-100 text-emerald-800";
  if (band === "watch") return "bg-amber-100 text-amber-800";
  if (band === "at_risk") return "bg-orange-100 text-orange-800";
  return "bg-red-100 text-red-800";
}

export function formatDay(iso?: string | null): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso.slice(0, 10);
  return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
}

export function money(n: number): string {
  return n.toLocaleString(undefined, { maximumFractionDigits: 0 });
}
