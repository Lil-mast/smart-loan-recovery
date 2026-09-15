"use client";

import { initializeApp, getApps, type FirebaseApp } from "firebase/app";
import { GoogleAuthProvider, getAuth, signInWithPopup, type AuthError } from "firebase/auth";
import { useState } from "react";
import { apiErrorMessage, apiFetch, clearSession } from "@/lib/api";
import { dashboardPath, type UserRole } from "@/lib/types";

type FirebaseWebConfig = {
  apiKey: string;
  authDomain: string;
  projectId: string;
  storageBucket?: string;
};

function firebaseApp(cfg: FirebaseWebConfig): FirebaseApp {
  const existing = getApps()[0];
  if (existing) return existing;
  return initializeApp({
    apiKey: cfg.apiKey,
    authDomain: cfg.authDomain,
    projectId: cfg.projectId,
    storageBucket: cfg.storageBucket,
  });
}

function googleErrorMessage(e: unknown): string {
  const code = typeof e === "object" && e && "code" in e ? String((e as AuthError).code) : "";
  const msg = e instanceof Error ? e.message : "";
  if (code === "auth/configuration-not-found" || msg.includes("CONFIGURATION_NOT_FOUND")) {
    return "Firebase Authentication is not enabled. In Firebase Console: Authentication → Get started, then enable Google and Email/Password.";
  }
  if (code === "auth/popup-blocked") {
    return "The Google popup was blocked. Allow popups for this site and try again.";
  }
  if (code === "auth/popup-closed-by-user") {
    return "Google sign-in was cancelled.";
  }
  if (code === "auth/unauthorized-domain") {
    return "Add this site to Firebase Auth authorized domains (localhost, 127.0.0.1, and your LAN IP).";
  }
  if (code === "auth/operation-not-allowed") {
    return "Enable Google as a sign-in provider in the Firebase console.";
  }
  if (e instanceof Error && e.message) return e.message;
  return "Google sign-in failed";
}

type Props = {
  role: UserRole;
  lenderId?: string;
  organization?: string;
  disabled?: boolean;
  stayOnPage?: boolean;
  label?: string;
};

export function GoogleButton({
  role,
  lenderId,
  organization,
  disabled,
  stayOnPage,
  label,
}: Props) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  async function handleClick() {
    setError("");
    setBusy(true);
    try {
      const cfgRes = await apiFetch("/auth/config");
      if (!cfgRes.ok) throw new Error("Could not load Firebase config from the API");
      const cfg = (await cfgRes.json()) as Partial<FirebaseWebConfig>;
      if (!cfg.apiKey || !cfg.authDomain || !cfg.projectId) {
        throw new Error("Firebase is not configured. Set FIREBASE_* in the API .env.firebase file.");
      }

      const auth = getAuth(firebaseApp(cfg as FirebaseWebConfig));
      const provider = new GoogleAuthProvider();
      provider.addScope("email");
      provider.addScope("profile");
      provider.setCustomParameters({ prompt: "select_account" });

      const result = await signInWithPopup(auth, provider);
      const idToken = await result.user.getIdToken();

      if (!stayOnPage) {
        await apiFetch("/auth/logout", { method: "POST", body: "{}" }).catch(() => undefined);
        await clearSession();
      }

      const res = await apiFetch("/auth/google", {
        method: "POST",
        body: JSON.stringify({
          id_token: idToken,
          role,
          lender_id: lenderId || undefined,
          organization: organization || undefined,
          link_existing: stayOnPage ? true : undefined,
        }),
      });
      const payload = (await res.json().catch(() => ({}))) as {
        user?: { role?: string };
        error?: string;
        message?: string;
        existing_role?: string;
      };
      if (!res.ok) {
        throw new Error(apiErrorMessage(payload, "Google sign-in failed"));
      }

      if (stayOnPage) {
        setBusy(false);
        return;
      }

      const got = String(payload.user?.role ?? "").toLowerCase();
      if (got && got !== role) {
        throw new Error(
          `This Google account is a ${got} workspace. Open that dashboard, or use a different Google account.`,
        );
      }
      window.location.assign(dashboardPath(role));
    } catch (e) {
      setError(googleErrorMessage(e));
      setBusy(false);
    }
  }

  return (
    <div>
      <button
        type="button"
        onClick={() => void handleClick()}
        disabled={busy || disabled}
        className="flex w-full items-center justify-center gap-2 rounded-lg bg-white px-4 py-2.5 text-sm font-semibold text-gray-800 disabled:opacity-60"
      >
        {busy ? "Connecting to Google…" : label ?? "Continue with Google"}
      </button>
      {error ? <p className="mt-2 text-center text-sm text-red-300">{error}</p> : null}
    </div>
  );
}
