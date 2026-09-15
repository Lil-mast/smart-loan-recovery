import { createHmac, timingSafeEqual } from "crypto";
import type { SessionUser, UserRole } from "./types";

const COOKIE = "lw_session";

function secret(): string {
  return process.env.SESSION_SECRET || "dev-only-set-SESSION_SECRET";
}

function normalizeRole(role: string): UserRole | null {
  const r = role.toLowerCase();
  if (r === "borrower" || r === "lender") return r;
  return null;
}

export function signSession(user: SessionUser): string {
  const payload = Buffer.from(JSON.stringify(user), "utf8").toString("base64url");
  const sig = createHmac("sha256", secret()).update(payload).digest("base64url");
  return `${payload}.${sig}`;
}

export function verifySession(token: string | undefined): SessionUser | null {
  if (!token || !token.includes(".")) return null;
  const [payload, sig] = token.split(".");
  if (!payload || !sig) return null;
  const expected = createHmac("sha256", secret()).update(payload).digest("base64url");
  const a = Buffer.from(sig);
  const b = Buffer.from(expected);
  if (a.length !== b.length || !timingSafeEqual(a, b)) return null;
  try {
    const raw = JSON.parse(Buffer.from(payload, "base64url").toString("utf8")) as SessionUser;
    const role = normalizeRole(String(raw.role));
    if (!raw.id || !role || !raw.name) return null;
    return { id: String(raw.id), role, name: String(raw.name) };
  } catch {
    return null;
  }
}

export function sessionFromUnknown(data: unknown): SessionUser | null {
  if (!data || typeof data !== "object") return null;
  const rec = data as Record<string, unknown>;
  const user = rec.user && typeof rec.user === "object" ? (rec.user as Record<string, unknown>) : rec;
  const id = user.local_user_id ?? user.user_id ?? user.id;
  const role = normalizeRole(String(user.role ?? ""));
  const rawName = user.name;
  const name =
    typeof rawName === "string" && rawName.trim() ? rawName.trim() : "Account";
  if (typeof id !== "string" || !role) return null;
  return { id, role, name };
}

export const SESSION_COOKIE = COOKIE;
