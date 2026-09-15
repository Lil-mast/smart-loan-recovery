"use client";

import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { GoogleButton } from "@/components/google-button";
import { apiErrorMessage, apiFetch } from "@/lib/api";
import type { UserRole } from "@/lib/types";

export default function LoginPage() {
  const router = useRouter();
  const [role, setRole] = useState<UserRole>("borrower");
  const [userId, setUserId] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setBusy(true);
    try {
      const res = await apiFetch("/auth/demo-login", {
        method: "POST",
        body: JSON.stringify({ user_id: userId.trim() }),
      });
      const data = (await res.json().catch(() => ({}))) as {
        role?: string;
      };
      if (!res.ok) throw new Error(apiErrorMessage(data, "Login failed"));
      const got = String(data.role || "").toLowerCase();
      if (got !== role) {
        throw new Error(`This ID is a ${got}. Switch account type.`);
      }
      router.push(role === "lender" ? "/lender" : "/borrower");
      router.refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Login failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-background px-4">
      <div className="w-full max-w-md rounded-2xl border border-line bg-card p-8">
        <Link href="/" className="text-sm text-muted">
          ← Home
        </Link>
        <h1 className="mt-4 text-2xl font-bold">Sign in</h1>
        <p className="mt-1 text-sm text-muted">Use your 4-character ID or Google.</p>
        <div className="mt-6 grid grid-cols-2 gap-1 rounded-xl bg-background p-1">
          <button
            type="button"
            className={`rounded-lg py-2 text-sm font-semibold ${role === "borrower" ? "bg-card-high" : "text-muted"}`}
            onClick={() => setRole("borrower")}
          >
            Borrower
          </button>
          <button
            type="button"
            className={`rounded-lg py-2 text-sm font-semibold ${role === "lender" ? "bg-card-high" : "text-muted"}`}
            onClick={() => setRole("lender")}
          >
            Lender
          </button>
        </div>
        <form onSubmit={(e) => void onSubmit(e)} className="mt-6 space-y-4">
          <label className="block text-xs font-semibold uppercase text-muted">
            User ID
            <input
              required
              maxLength={4}
              pattern="[A-Za-z0-9]{4}"
              value={userId}
              onChange={(e) => setUserId(e.target.value)}
              className="mt-1 w-full rounded-lg bg-background px-3 py-2.5 font-mono uppercase"
              placeholder="DEMO"
            />
          </label>
          <p className="text-xs text-muted">Demo: borrower DEMO · lender BANK</p>
          {error ? <p className="text-sm text-red-300">{error}</p> : null}
          <button
            type="submit"
            disabled={busy}
            className="w-full rounded-lg bg-primary-container py-2.5 font-semibold text-on-primary-container"
          >
            Login
          </button>
        </form>
        <div className="mt-4">
          <GoogleButton role={role} onDone={() => router.push(role === "lender" ? "/lender" : "/borrower")} />
        </div>
        <p className="mt-6 text-center text-sm text-muted">
          New here?{" "}
          <Link href="/register" className="text-secondary">
            Register
          </Link>
        </p>
      </div>
    </div>
  );
}
