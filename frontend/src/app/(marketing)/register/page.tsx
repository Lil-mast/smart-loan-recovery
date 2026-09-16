"use client";

import { useEffect, useState } from "react";
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

type BorrowerMethod = "google" | "email";

export default function RegisterPage() {
  const [role, setRole] = useState<UserRole>("borrower");
  const [method, setMethod] = useState<BorrowerMethod>("google");
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [lenderId, setLenderId] = useState("");
  const [organization, setOrganization] = useState("");
  const [agreed, setAgreed] = useState(false);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [createdId, setCreatedId] = useState("");

  useEffect(() => {
    void clearSession();
  }, []);

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
      <AuthShell
        title={role === "lender" ? "Your account ID" : "You’re in"}
        switchPrompt="Ready to work?"
        switchHref={dashboardPath(role)}
        switchLabel="Open dashboard"
      >
        {role === "lender" ? (
          <>
            <p className="text-sm text-muted-foreground">
              Share this ID with borrowers so they can join your book. Keep it — you’ll use it to
              sign in.
            </p>
            <p className="mt-6 rounded-2xl bg-muted px-4 py-6 text-center font-mono text-4xl font-bold tracking-[0.3em]">
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
          <p className="text-sm text-muted-foreground">
            Your borrower workspace is ready. Open the dashboard to see loans from your lender.
          </p>
        )}
        <Button
          type="button"
          className="mt-8 h-12 w-full rounded-full bg-[#111] text-white hover:bg-[#111]/90"
          onClick={() => window.location.assign(dashboardPath(role))}
        >
          Continue to dashboard
        </Button>
      </AuthShell>
    );
  }

  return (
    <AuthShell
      title="Create account"
      switchPrompt="Already registered?"
      switchHref="/login"
      switchLabel="Log in"
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
            ]}
          />
        </div>
      ) : null}

      {error ? <p className="mt-4 text-sm text-destructive">{error}</p> : null}

      {role === "borrower" && method === "google" ? (
        <div className="mt-6 space-y-4">
          <div className="space-y-2">
            <Label htmlFor="lender-id">Lender account ID</Label>
            <Input
              id="lender-id"
              required
              maxLength={4}
              pattern="[A-Za-z0-9]{4}"
              value={lenderId}
              onChange={(e) => setLenderId(e.target.value.toUpperCase())}
              className="h-12 rounded-full font-mono uppercase"
              placeholder="ABCD"
            />
          </div>
          <label className="flex items-center gap-2 text-sm text-muted-foreground">
            <Checkbox checked={agreed} onCheckedChange={(v) => setAgreed(v === true)} />
            I agree to the <span className="font-medium text-foreground underline">Terms & Condition</span>
          </label>
          <GoogleButton
            role="borrower"
            lenderId={lenderId.trim()}
            disabled={!agreed || lenderId.trim().length !== 4}
          />
          {lenderId.trim().length !== 4 ? (
            <p className="text-center text-xs text-muted-foreground">
              Enter your lender’s 4-character ID first.
            </p>
          ) : null}
        </div>
      ) : (
        <form onSubmit={(e) => void onSubmit(e)} className="mt-6 space-y-4">
          <div className="space-y-2">
            <Label htmlFor="name">Full name</Label>
            <Input
              id="name"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="h-12 rounded-full"
              placeholder="Your name"
            />
          </div>
          {role === "lender" ? (
            <div className="space-y-2">
              <Label htmlFor="company">Company</Label>
              <Input
                id="company"
                required
                value={organization}
                onChange={(e) => setOrganization(e.target.value)}
                className="h-12 rounded-full"
                placeholder="Your company or institution"
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
                <PasswordField
                  id="password"
                  value={password}
                  onChange={setPassword}
                  minLength={8}
                  placeholder="At least 8 characters"
                />
              </div>
            </>
          )}
          <label className="flex items-center gap-2 text-sm text-muted-foreground">
            <Checkbox checked={agreed} onCheckedChange={(v) => setAgreed(v === true)} />
            I agree to the <span className="font-medium text-foreground underline">Terms & Condition</span>
          </label>
          <Button
            type="submit"
            disabled={busy}
            className="h-12 w-full rounded-full bg-[#111] text-white hover:bg-[#111]/90"
          >
            {role === "lender" ? "Create workspace" : "Create account"}
          </Button>
        </form>
      )}

      {role === "lender" ? (
        <div className="mt-6">
          <div className="relative my-4">
            <Separator />
            <p className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-white px-3 text-xs text-muted-foreground">
              or
            </p>
          </div>
          <GoogleButton
            role="lender"
            organization={organization}
            disabled={!agreed || !organization.trim()}
          />
        </div>
      ) : null}
    </AuthShell>
  );
}
