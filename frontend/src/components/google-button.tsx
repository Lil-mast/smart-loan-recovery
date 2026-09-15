"use client";

import { useState } from "react";
import { initializeApp, getApps } from "firebase/app";
import { GoogleAuthProvider, getAuth, signInWithPopup } from "firebase/auth";
import { apiErrorMessage, apiFetch } from "@/lib/api";
import type { UserRole } from "@/lib/types";

type Props = {
  role: UserRole;
  onDone: () => void;
};

export function GoogleButton({ role, onDone }: Props) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  async function handleClick() {
    setError("");
    setBusy(true);
    try {
      const cfgRes = await apiFetch("/auth/config");
      if (!cfgRes.ok) throw new Error("Could not load Firebase config");
      const cfg = (await cfgRes.json()) as {
        apiKey?: string;
        authDomain?: string;
        projectId?: string;
        storageBucket?: string;
      };
      if (!cfg.apiKey || !cfg.projectId) {
        throw new Error("Google sign-in is not configured on the API");
      }
      const app = getApps()[0] ?? initializeApp(cfg);
      const auth = getAuth(app);
      const provider = new GoogleAuthProvider();
      provider.addScope("email");
      provider.addScope("profile");
      const result = await signInWithPopup(auth, provider);
      const idToken = await result.user.getIdToken();
      const res = await apiFetch("/auth/google", {
        method: "POST",
        body: JSON.stringify({ id_token: idToken, role }),
      });
      if (!res.ok) {
        const err = await res.json().catch(() => ({}));
        throw new Error(apiErrorMessage(err, "Google sign-in failed"));
      }
      onDone();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Google sign-in failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div>
      <button
        type="button"
        onClick={() => void handleClick()}
        disabled={busy}
        className="flex w-full items-center justify-center gap-2 rounded-lg bg-white px-4 py-2.5 text-sm font-semibold text-gray-800 disabled:opacity-60"
      >
        Sign in with Google
      </button>
      {error ? <p className="mt-2 text-center text-sm text-red-300">{error}</p> : null}
    </div>
  );
}
