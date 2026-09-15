"use client";

import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { GoogleButton } from "@/components/google-button";
import { apiErrorMessage, apiFetch } from "@/lib/api";
import type { UserRole } from "@/lib/types";

const LENDERS = ["M-shwari", "Branch", "Tala", "Eazzy Loan", "KCB-Mpesa"];

export default function RegisterPage() {
  const router = useRouter();
  const [role, setRole] = useState<UserRole>("borrower");
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [lenderName, setLenderName] = useState("");
  const [organization, setOrganization] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setBusy(true);
    try {
      const res = await apiFetch("/users", {
        method: "POST",
        body: JSON.stringify({
          name,
          email: email || null,
          role,
          lender_name: role === "borrower" ? lenderName : null,
          organization: role === "lender" ? organization : null,
        }),
      });
      const data = (await res.json().catch(() => ({}))) as { id?: string };
      if (!res.ok) {
        throw new Error(apiErrorMessage(data, "Registration failed"));
      }
      alert(`Registered. Your user ID is ${data.id ?? ""}. Sign in with it.`);
      router.push("/login");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Registration failed");
    } finally {
      setBusy(false);
    }
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
            Institutional onboarding
          </p>
          <h1 className="mb-2 text-2xl font-bold">Create Your Recovery Intelligence Account</h1>
          <p className="mb-6 text-sm text-muted">Borrower and lender workspaces for loan recovery.</p>
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
            <label className="block text-xs font-semibold uppercase text-muted">
              Email (optional)
              <input
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="mt-1 w-full rounded-lg bg-background px-3 py-2.5 text-foreground"
                placeholder="name@firm.com"
              />
            </label>
            {role === "borrower" ? (
              <label className="block text-xs font-semibold uppercase text-muted">
                Choose lender / bank
                <select
                  required
                  value={lenderName}
                  onChange={(e) => setLenderName(e.target.value)}
                  className="mt-1 w-full rounded-lg bg-background px-3 py-2.5 text-foreground"
                >
                  <option value="">Select institution</option>
                  {LENDERS.map((l) => (
                    <option key={l} value={l}>
                      {l}
                    </option>
                  ))}
                </select>
              </label>
            ) : (
              <label className="block text-xs font-semibold uppercase text-muted">
                Organization
                <input
                  required
                  value={organization}
                  onChange={(e) => setOrganization(e.target.value)}
                  className="mt-1 w-full rounded-lg bg-background px-3 py-2.5 text-foreground"
                  placeholder="Bank or organization"
                />
              </label>
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
                Complete registration
              </button>
            </div>
          </form>
          <div className="mt-4">
            <GoogleButton role={role} onDone={() => router.push(role === "lender" ? "/lender" : "/borrower")} />
          </div>
          <p className="mt-6 text-center text-sm text-muted">
            Already have an ID?{" "}
            <Link href="/login" className="text-secondary">
              Sign in
            </Link>
            {" · "}demo borrower DEMO · lender BANK
          </p>
        </div>
      </div>
    </div>
  );
}
