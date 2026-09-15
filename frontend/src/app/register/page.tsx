"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { GoogleButton } from "@/components/google-button";
import { apiErrorMessage, apiFetch, clearSession } from "@/lib/api";
import { dashboardPath, type UserRole } from "@/lib/types";

type BorrowerMethod = "google" | "email";

export default function RegisterPage() {
  const [role, setRole] = useState<UserRole>("borrower");
  const [method, setMethod] = useState<BorrowerMethod>("google");
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [lenderId, setLenderId] = useState("");
  const [organization, setOrganization] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [createdId, setCreatedId] = useState("");

  useEffect(() => {
    void clearSession();
  }, []);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setBusy(true);
    try {
      await apiFetch("/auth/logout", { method: "POST", body: "{}" }).catch(() => undefined);
      await clearSession();
      if (role === "lender") {
        const res = await apiFetch("/users", {
          method: "POST",
          body: JSON.stringify({
            name,
            role: "lender",
            organization,
          }),
        });
        const data = (await res.json().catch(() => ({}))) as { id?: string };
        if (!res.ok) {
          throw new Error(apiErrorMessage(data, "Registration failed"));
        }
        const id = String(data.id ?? "").trim();
        if (!id) throw new Error("Registered, but no account ID was returned");
        await apiFetch("/auth/id-login", {
          method: "POST",
          body: JSON.stringify({ user_id: id }),
        }).catch(() => undefined);
        setCreatedId(id);
        return;
      }

      if (method !== "email") {
        throw new Error("Use Google to create this account, or switch to email.");
      }

      const res = await apiFetch("/auth/register", {
        method: "POST",
        body: JSON.stringify({
          name,
          email,
          password,
          role: "borrower",
          lender_id: lenderId.trim().toUpperCase(),
        }),
      });
      const data = (await res.json().catch(() => ({}))) as {
        user?: { local_user_id?: string };
        error?: string;
      };
      if (!res.ok) {
        throw new Error(apiErrorMessage(data, "Registration failed"));
      }
      const id = String(data.user?.local_user_id ?? "").trim();
      setCreatedId(id || "saved");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Registration failed");
    } finally {
      setBusy(false);
    }
  }

  if (createdId) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background px-4 py-16">
        <div className="w-full max-w-lg rounded-2xl border border-line bg-card/80 p-8 shadow-2xl">
          <p className="mb-2 font-mono text-[11px] uppercase tracking-widest text-primary">
            Account created
          </p>
          <h1 className="mb-2 text-2xl font-bold">
            {role === "lender" ? "Your account ID" : "You’re in"}
          </h1>
          {role === "lender" ? (
            <>
              <p className="mb-6 text-sm text-muted">
                Share this ID with borrowers so they can join your book. Keep it — you’ll use it to
                sign in.
              </p>
              <p className="rounded-xl bg-background px-4 py-6 text-center font-mono text-4xl font-bold tracking-[0.3em]">
                {createdId}
              </p>
              <div className="mt-6">
                <GoogleButton
                  role="lender"
                  organization={organization}
                  stayOnPage
                  label="Link Google for faster sign-in"
                />
              </div>
            </>
          ) : (
            <p className="mb-6 text-sm text-muted">
              Your borrower workspace is ready. Open the dashboard to see loans from your lender.
            </p>
          )}
          <button
            type="button"
            onClick={() => window.location.assign(dashboardPath(role))}
            className="mt-8 w-full rounded-lg bg-primary-container py-2.5 font-semibold text-on-primary-container"
          >
            Continue to dashboard
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-background px-4 py-16">
      <div className="w-full max-w-lg">
        <div className="mb-6 flex items-center justify-between">
          <Link href="/" className="text-xl font-bold">
            LendWise<span className="text-secondary">Recovery</span>
          </Link>
          <span className="rounded-full bg-card px-3 py-1 font-mono text-[11px] uppercase text-secondary">
            Onboarding
          </span>
        </div>
        <div className="rounded-2xl border border-line bg-card/80 p-8 shadow-2xl">
          <p className="mb-2 font-mono text-[11px] uppercase tracking-widest text-primary">
            Create account
          </p>
          <h1 className="mb-2 text-2xl font-bold">Join LendWise Recovery</h1>
          <p className="mb-6 text-sm text-muted">
            Lenders open a company workspace. Borrowers join with the lender’s account ID.
          </p>
          <div className="mb-6 grid grid-cols-2 gap-1 rounded-xl bg-background p-1">
            <button
              type="button"
              className={`rounded-lg py-2 text-sm font-semibold ${role === "borrower" ? "bg-card-high text-foreground" : "text-muted"}`}
              onClick={() => setRole("borrower")}
            >
              Borrower
            </button>
            <button
              type="button"
              className={`rounded-lg py-2 text-sm font-semibold ${role === "lender" ? "bg-card-high text-foreground" : "text-muted"}`}
              onClick={() => setRole("lender")}
            >
              Lender
            </button>
          </div>

          {role === "borrower" ? (
            <div className="mb-6 grid grid-cols-2 gap-1 rounded-xl bg-background p-1">
              <button
                type="button"
                className={`rounded-lg py-2 text-sm font-semibold ${method === "google" ? "bg-card-high text-foreground" : "text-muted"}`}
                onClick={() => setMethod("google")}
              >
                Google
              </button>
              <button
                type="button"
                className={`rounded-lg py-2 text-sm font-semibold ${method === "email" ? "bg-card-high text-foreground" : "text-muted"}`}
                onClick={() => setMethod("email")}
              >
                Email
              </button>
            </div>
          ) : null}

          {role === "borrower" ? (
            <label className="mb-4 block text-xs font-semibold uppercase text-muted">
              Lender account ID
              <input
                required
                maxLength={4}
                pattern="[A-Za-z0-9]{4}"
                value={lenderId}
                onChange={(e) => setLenderId(e.target.value.toUpperCase())}
                className="mt-1 w-full rounded-lg bg-background px-3 py-2.5 font-mono uppercase text-foreground"
                placeholder="ABCD"
              />
            </label>
          ) : null}

          {role === "borrower" && method === "google" ? (
            <div>
              {error ? <p className="mb-3 text-sm text-red-300">{error}</p> : null}
              <GoogleButton
                role="borrower"
                lenderId={lenderId.trim()}
                disabled={lenderId.trim().length !== 4}
              />
              {lenderId.trim().length !== 4 ? (
                <p className="mt-2 text-center text-xs text-muted">
                  Enter your lender’s 4-character ID first.
                </p>
              ) : null}
            </div>
          ) : (
            <form onSubmit={(e) => void onSubmit(e)} className="space-y-4">
              <label className="block text-xs font-semibold uppercase text-muted">
                Full name
                <input
                  required
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="mt-1 w-full rounded-lg bg-background px-3 py-2.5 text-foreground"
                  placeholder="Your name"
                />
              </label>
              {role === "lender" ? (
                <label className="block text-xs font-semibold uppercase text-muted">
                  Company
                  <input
                    required
                    value={organization}
                    onChange={(e) => setOrganization(e.target.value)}
                    className="mt-1 w-full rounded-lg bg-background px-3 py-2.5 text-foreground"
                    placeholder="Your company or institution"
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
                      className="mt-1 w-full rounded-lg bg-background px-3 py-2.5 text-foreground"
                      placeholder="you@email.com"
                    />
                  </label>
                  <label className="block text-xs font-semibold uppercase text-muted">
                    Password
                    <input
                      required
                      type="password"
                      minLength={8}
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      className="mt-1 w-full rounded-lg bg-background px-3 py-2.5 text-foreground"
                      placeholder="At least 8 characters"
                    />
                  </label>
                </>
              )}
              {error ? <p className="text-sm text-red-300">{error}</p> : null}
              <div className="flex gap-3">
                <Link href="/" className="flex-1 rounded-lg border border-line py-2.5 text-center">
                  Cancel
                </Link>
                <button
                  type="submit"
                  disabled={busy}
                  className="flex-1 rounded-lg bg-primary-container py-2.5 font-semibold text-on-primary-container"
                >
                  {role === "lender" ? "Create workspace" : "Create account"}
                </button>
              </div>
            </form>
          )}

          {role === "lender" ? (
            <div className="mt-4">
              <p className="mb-2 text-center text-xs text-muted">Or create with Google</p>
              <GoogleButton
                role="lender"
                organization={organization}
                disabled={!organization.trim()}
              />
            </div>
          ) : null}

          <p className="mt-6 text-center text-sm text-muted">
            Already registered?{" "}
            <Link href="/login" className="text-secondary">
              Sign in
            </Link>
          </p>
        </div>
      </div>
    </div>
  );
}
