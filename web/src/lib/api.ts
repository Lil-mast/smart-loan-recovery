import type { SessionUser } from "./types";

export async function apiFetch(path: string, init: RequestInit = {}): Promise<Response> {
  const headers = new Headers(init.headers);
  if (init.body && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }
  return fetch(`/api/v1${path}`, {
    ...init,
    headers,
    credentials: "include",
  });
}

export async function getSession(): Promise<SessionUser | null> {
  const res = await fetch("/api/session", { credentials: "include" });
  if (!res.ok) return null;
  return (await res.json()) as SessionUser;
}

export async function clearSession(): Promise<void> {
  await fetch("/api/session", { method: "DELETE", credentials: "include" });
}

export function apiErrorMessage(data: unknown, fallback: string): string {
  if (!data || typeof data !== "object") return fallback;
  const rec = data as Record<string, unknown>;
  if (typeof rec.message === "string" && rec.message) return rec.message;
  if (typeof rec.error === "string" && rec.error) return rec.error;
  return fallback;
}
