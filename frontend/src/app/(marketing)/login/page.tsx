"use client";

import { useState } from "react";
import { GoogleButton } from "@/components/google-button";
import { AuthPills } from "@/components/marketing/auth-pills";
import { AuthShell } from "@/components/marketing/auth-shell";
import { PasswordField } from "@/components/marketing/password-field";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { apiErrorMessage, apiFetch, clearSession } from "@/lib/api";
import { dashboardPath, type UserRole } from "@/lib/types";

type BorrowerMethod = "google" | "email" | "id";

export default function LoginPage() {
  const [role, setRole] = useState<UserRole>("borrower");
  const [method, setMethod] = useState<BorrowerMethod>("google");
  const [userId, setUserId] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [agreed, setAgreed] = useState(false);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    if (!agreed) {
      setError("Please agree to the terms before continuing.");
      return;
    }
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

  const showGoogle = role === "borrower" ? method === "google" : true;
  const showForm = role === "lender" || method !== "google";

  return (
    <AuthShell
      title="Log in"
      switchPrompt="Don’t have an account?"
      switchHref="/register"
      switchLabel="Create an Account"
    >
      <AuthPills
        value={role}
        onChange={setRole}
        options={[
          { value: "borrower", label: "Borrower" },
          { value: "lender", label: "Lender" },
        ]}
      />

      {role === "borrower" ? (
        <div className="mt-3">
          <AuthPills
            value={method}
            onChange={setMethod}
            options={[
              { value: "google", label: "Google" },
              { value: "email", label: "Email" },
              { value: "id", label: "Account ID" },
            ]}
          />
        </div>
      ) : null}

      {error ? <p className="mt-4 text-sm text-destructive">{error}</p> : null}

      {showForm ? (
        <form onSubmit={(e) => void onSubmit(e)} className="mt-6 space-y-4">
          {role === "lender" || method === "id" ? (
            <div className="space-y-2">
              <Label htmlFor="user-id">Account ID</Label>
              <Input
                id="user-id"
                required
                maxLength={4}
                pattern="[A-Za-z0-9]{4}"
                value={userId}
                onChange={(e) => setUserId(e.target.value)}
                className="h-12 rounded-full font-mono uppercase"
                placeholder="ABCD"
              />
            </div>
          ) : (
            <>
              <div className="space-y-2">
                <Label htmlFor="email">Email Address</Label>
                <Input
                  id="email"
                  required
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="h-12 rounded-full"
                  placeholder="you@email.com"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <PasswordField id="password" value={password} onChange={setPassword} />
              </div>
            </>
          )}
          <label className="flex items-center gap-2 text-sm text-muted-foreground">
            <Checkbox checked={agreed} onCheckedChange={(v) => setAgreed(v === true)} />
            I agree to the <span className="font-medium text-foreground underline">Terms & Condition</span>
          </label>
          <Button type="submit" disabled={busy} className="h-12 w-full rounded-full bg-[#111] text-white hover:bg-[#111]/90">
            Log in
          </Button>
        </form>
      ) : (
        <div className="mt-6 space-y-4">
          <label className="flex items-center gap-2 text-sm text-muted-foreground">
            <Checkbox checked={agreed} onCheckedChange={(v) => setAgreed(v === true)} />
            I agree to the <span className="font-medium text-foreground underline">Terms & Condition</span>
          </label>
          <GoogleButton role="borrower" disabled={!agreed} />
          <p className="text-center text-xs text-muted-foreground">
            New here? Register first with your lender’s account ID.
          </p>
        </div>
      )}

      {showGoogle && role === "lender" ? (
        <div className="mt-6">
          <div className="relative my-4">
            <Separator />
            <p className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-white px-3 text-xs text-muted-foreground">
              or
            </p>
          </div>
          <p className="mb-2 text-center text-xs text-muted-foreground">If you linked Google</p>
          <GoogleButton role="lender" disabled={!agreed} />
        </div>
      ) : null}
    </AuthShell>
  );
}
