import type { HealthBand } from "./types";

export function money(n: number): string {
  return n.toLocaleString(undefined, { style: "currency", currency: "USD", maximumFractionDigits: 0 });
}

export function moneyExact(n: number): string {
  return n.toLocaleString(undefined, { style: "currency", currency: "USD", maximumFractionDigits: 2 });
}

export function bandLabel(band: HealthBand | string): string {
  if (band === "at_risk") return "At risk";
  if (band === "healthy") return "Healthy";
  if (band === "watch") return "Watch";
  if (band === "critical") return "Critical";
  return band.replaceAll("_", " ");
}

export function recLabel(key: string): string {
  if (key === "send_reminder") return "Send reminder";
  if (key === "renegotiate_terms") return "Renegotiate";
  if (key === "escalate_to_collection") return "Escalate";
  return key.replaceAll("_", " ");
}

export function dayKey(iso: string): string {
  return iso.slice(0, 10);
}

export function formatDay(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso.slice(0, 10);
  return d.toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" });
}
