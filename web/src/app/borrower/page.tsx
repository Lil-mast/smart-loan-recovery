"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { apiFetch, clearSession, getSession } from "@/lib/api";
import type { Loan, SessionUser } from "@/lib/types";

export default function BorrowerPage() {
  const router = useRouter();
  const [user, setUser] = useState<SessionUser | null>(null);
  const [loans, setLoans] = useState<Loan[]>([]);
  const [error, setError] = useState("");

  useEffect(() => {
    void (async () => {
      const s = await getSession();
      if (!s) {
        router.replace("/login?next=/borrower");
        return;
      }
      setUser(s);
      const res = await apiFetch(`/loans?borrower_id=${encodeURIComponent(s.id)}`);
      if (!res.ok) {
        setError("Could not load loans");
        return;
      }
      setLoans((await res.json()) as Loan[]);
    })();
  }, [router]);

  async function logout() {
    await apiFetch("/auth/logout", { method: "POST" });
    await clearSession();
    router.push("/");
  }

  if (!user) return <p className="p-8 text-muted">Loading…</p>;

  return (
    <div className="min-h-screen bg-background px-6 py-10">
      <div className="mx-auto max-w-4xl">
        <div className="mb-8 flex items-center justify-between">
          <div>
            <p className="font-mono text-xs uppercase text-secondary">Borrower hub</p>
            <h1 className="text-3xl font-bold">{user.name}</h1>
            <p className="font-mono text-sm text-muted">ID {user.id}</p>
          </div>
          <div className="flex gap-3">
            <Link href="/" className="text-sm text-muted">
              Home
            </Link>
            <button type="button" onClick={() => void logout()} className="text-sm text-muted">
              Log out
            </button>
          </div>
        </div>
        {error ? <p className="text-red-300">{error}</p> : null}
        <div className="overflow-hidden rounded-2xl border border-line">
          <table className="w-full text-left text-sm">
            <thead className="bg-card-high text-muted">
              <tr>
                <th className="px-4 py-3">Loan</th>
                <th className="px-4 py-3">Principal</th>
                <th className="px-4 py-3">Status</th>
                <th className="px-4 py-3">Risk</th>
                <th className="px-4 py-3">Action</th>
              </tr>
            </thead>
            <tbody>
              {loans.length === 0 ? (
                <tr>
                  <td className="px-4 py-6 text-muted" colSpan={5}>
                    No loans yet.
                  </td>
                </tr>
              ) : (
                loans.map((l) => (
                  <tr key={l.id} className="border-t border-line">
                    <td className="px-4 py-3 font-mono text-xs">{l.id.slice(0, 8)}</td>
                    <td className="px-4 py-3">${l.principal.toLocaleString()}</td>
                    <td className="px-4 py-3 capitalize">{l.status}</td>
                    <td className="px-4 py-3">{(l.risk_score * 100).toFixed(0)}%</td>
                    <td className="px-4 py-3">{l.ai_recommendation.replaceAll("_", " ")}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
