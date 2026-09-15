"use client";

import { useState } from "react";
import Link from "next/link";
import { GoogleButton } from "@/components/google-button";
import { apiErrorMessage, apiFetch, clearSession } from "@/lib/api";
import { dashboardPath, type UserRole } from "@/lib/types";

type BorrowerMethod = "google" | "email" | "id";

export default function LoginPage() {
  const [role, setRole] = useState<UserRole>("borrower");
  const [method, setMethod] = useState<BorrowerMethod>("google");
  const [userId, setUserId] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setBusy(true);
    try {
      await apiFetch("/auth/logout", { method: "POST", body: "{}" }).catch(() => undefined);
      await clearSession();
      if (role === "lender" || (role === "borrower" && method === "id")) {
        const res = await apiFetch("/auth/id-login", {
          method: "POST",
          body: JSON.stringify({ user_id: userId.trim().toUpperCase() }),
        });
        const data = (await res.json().catch(() => ({}))) as { role?: string };
        if (!res.ok) throw new Error(apiErrorMessage(data, "Sign-in failed"));
        const got = String(data.role || "").toLowerCase();
        if (got !== role) {
          throw new Error(
            got === "lender"
              ? "This ID is a lender account. Switch to the lender tab."
              : "This ID is a borrower account. Switch to the borrower tab.",
          );
        }
        window.location.assign(dashboardPath(role));
        return;
      }

      const res = await apiFetch("/auth/login", {
        method: "POST",
        body: JSON.stringify({ email: email.trim(), password }),
      });
      const data = (await res.json().catch(() => ({}))) as {
        user?: { role?: string };
      };
      if (!res.ok) throw new Error(apiErrorMessage(data, "Sign-in failed"));
      const got = String(data.user?.role || "").toLowerCase();
      if (got && got !== "borrower") {
        throw new Error("This email belongs to a lender account.");
      }
      window.location.assign("/borrower");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Sign-in failed");
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
        <p className="mt-1 text-sm text-muted">
          Borrowers can use Google. Lenders sign in with their account ID.
        </p>
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

        {role === "borrower" ? (
          <div className="mt-4 grid grid-cols-3 gap-1 rounded-xl bg-background p-1">
            <button
              type="button"
              className={`rounded-lg py-2 text-sm font-semibold ${method === "google" ? "bg-card-high" : "text-muted"}`}
              onClick={() => setMethod("google")}
            >
              Google
            </button>
            <button
              type="button"
              className={`rounded-lg py-2 text-sm font-semibold ${method === "email" ? "bg-card-high" : "text-muted"}`}
              onClick={() => setMethod("email")}
            >
              Email
            </button>
            <button
              type="button"
              className={`rounded-lg py-2 text-sm font-semibold ${method === "id" ? "bg-card-high" : "text-muted"}`}
              onClick={() => setMethod("id")}
            >
              Account ID
            </button>
          </div>
        ) : null}

        {error ? <p className="mt-4 text-sm text-red-300">{error}</p> : null}

        {role === "borrower" && method === "google" ? (
          <div className="mt-6">
            <GoogleButton role="borrower" />
            <p className="mt-3 text-center text-xs text-muted">
              New here? Register first with your lender’s account ID.
            </p>
          </div>
        ) : (
          <form onSubmit={(e) => void onSubmit(e)} className="mt-6 space-y-4">
            {role === "lender" || method === "id" ? (
              <label className="block text-xs font-semibold uppercase text-muted">
                Account ID
                <input
                  required
                  maxLength={4}
                  pattern="[A-Za-z0-9]{4}"
                  value={userId}
                  onChange={(e) => setUserId(e.target.value)}
                  className="mt-1 w-full rounded-lg bg-background px-3 py-2.5 font-mono uppercase"
                  placeholder="ABCD"
                />
              </label>
            ) : (
              <>
                <label className="block text-xs font-semibold uppercase text-muted">
                  Email
                  <input
                    required
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    className="mt-1 w-full rounded-lg bg-background px-3 py-2.5"
                    placeholder="you@email.com"
                  />
                </label>
                <label className="block text-xs font-semibold uppercase text-muted">
                  Password
                  <input
                    required
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="mt-1 w-full rounded-lg bg-background px-3 py-2.5"
                  />
                </label>
              </>
            )}
            <button
              type="submit"
              disabled={busy}
              className="w-full rounded-lg bg-primary-container py-2.5 font-semibold text-on-primary-container"
            >
              Sign in
            </button>
          </form>
        )}

        {role === "lender" ? (
          <div className="mt-4">
            <p className="mb-2 text-center text-xs text-muted">If you linked Google</p>
            <GoogleButton role="lender" />
          </div>
        ) : null}

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
