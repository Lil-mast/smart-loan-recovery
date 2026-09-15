"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { apiFetch, clearSession, getSession } from "@/lib/api";
import type { ApiUser, Loan, SessionUser } from "@/lib/types";

function recLabel(key: string): string {
  if (key === "send_reminder") return "Send reminder";
  if (key === "renegotiate_terms") return "Renegotiate";
  if (key === "escalate_to_collection") return "Escalate";
  return key.replaceAll("_", " ");
}

export default function LenderPage() {
  const router = useRouter();
  const [user, setUser] = useState<SessionUser | null>(null);
  const [loans, setLoans] = useState<Loan[]>([]);
  const [borrowers, setBorrowers] = useState<ApiUser[]>([]);
  const [error, setError] = useState("");
  const [principal, setPrincipal] = useState("10000");
  const [rate, setRate] = useState("8.5");
  const [borrowerId, setBorrowerId] = useState("");

  async function load(s: SessionUser) {
    const [lr, br] = await Promise.all([
      apiFetch(`/loans?lender_id=${encodeURIComponent(s.id)}`),
      apiFetch(`/users?role=borrower&lender_id=${encodeURIComponent(s.id)}`),
    ]);
    if (!lr.ok) {
      setError("Could not load loans");
      return;
    }
    const list = (await lr.json()) as Loan[];
    setLoans(list);
    if (br.ok) {
      const users = (await br.json()) as ApiUser[];
      setBorrowers(users);
      if (users[0] && !borrowerId) setBorrowerId(users[0].id);
    }
  }

  useEffect(() => {
    void (async () => {
      const s = await getSession();
      if (!s) {
        router.replace("/login?next=/lender");
        return;
      }
      setUser(s);
      await load(s);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps -- initial load
  }, [router]);

  async function logout() {
    await apiFetch("/auth/logout", { method: "POST" });
    await clearSession();
    router.push("/");
  }

  async function flagOverdues() {
    const res = await apiFetch("/overdues", { method: "POST" });
    if (!res.ok) {
      setError("Flag overdues failed (are you signed in as lender?)");
      return;
    }
    if (user) await load(user);
  }

  async function recommend(id: string) {
    const res = await apiFetch(`/recommend/${encodeURIComponent(id)}`, { method: "POST" });
    const j = (await res.json().catch(() => ({}))) as {
      risk_score?: number;
      recommended_action?: string;
    };
    if (!res.ok) {
      alert("Could not load recommendation");
      return;
    }
    alert(`Risk: ${Math.round(Number(j.risk_score ?? 0) * 100)}%\n${String(j.recommended_action ?? "")}`);
  }

  async function createLoan(e: React.FormEvent) {
    e.preventDefault();
    if (!user || !borrowerId) return;
    const res = await apiFetch("/loans", {
      method: "POST",
      body: JSON.stringify({
        borrower_id: borrowerId,
        lender_id: user.id,
        principal: Number(principal),
        interest_rate: Number(rate),
        months: 12,
      }),
    });
    if (!res.ok) {
      setError("Create loan failed");
      return;
    }
    await load(user);
  }

  if (!user) return <p className="p-8 text-muted">Loading…</p>;

  const recovered = loans.reduce((s, l) => s + (l.principal - l.outstanding_amount), 0);
  const highRisk = loans.filter((l) => l.status === "overdue" || l.status === "defaulted" || l.risk_score >= 0.45).length;

  return (
    <div className="min-h-screen bg-background lg:pl-64">
      <aside className="fixed left-0 top-0 hidden h-full w-64 flex-col justify-between border-r border-line bg-[#0a0e18] p-6 lg:flex">
        <div>
          <p className="text-lg font-bold">LendWise</p>
          <p className="font-mono text-[11px] uppercase text-secondary">Institutional AI</p>
          <nav className="mt-8 space-y-1 text-sm">
            <span className="block rounded-lg bg-primary-container px-3 py-2 font-semibold text-on-primary-container">
              Lender portfolio
            </span>
            <Link href="/" className="block rounded-lg px-3 py-2 text-muted">
              Home
            </Link>
          </nav>
        </div>
        <button type="button" onClick={() => void logout()} className="text-left text-sm text-muted">
          Exit to main
        </button>
      </aside>
      <main className="px-6 py-8">
        <div className="mb-8 flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="font-mono text-xs uppercase text-secondary">Portfolio command</p>
            <h1 className="text-3xl font-bold">Lender Operations</h1>
            <p className="font-mono text-sm text-muted">
              {user.name} · {user.id}
            </p>
          </div>
          <button
            type="button"
            onClick={() => void flagOverdues()}
            className="rounded-lg bg-primary-container px-4 py-2 text-sm font-semibold text-on-primary-container"
          >
            Flag overdues
          </button>
        </div>
        <div className="mb-8 grid grid-cols-1 gap-4 sm:grid-cols-3">
          <div className="rounded-xl border border-line bg-card p-5">
            <p className="font-mono text-[11px] uppercase text-muted">Principal book</p>
            <p className="mt-2 text-2xl font-semibold">
              ${loans.reduce((s, l) => s + l.principal, 0).toLocaleString()}
            </p>
          </div>
          <div className="rounded-xl border border-line bg-card p-5">
            <p className="font-mono text-[11px] uppercase text-muted">Loans</p>
            <p className="mt-2 text-2xl font-semibold">{loans.length}</p>
          </div>
          <div className="rounded-xl border border-line bg-card p-5">
            <p className="font-mono text-[11px] uppercase text-muted">High risk</p>
            <p className="mt-2 text-2xl font-semibold">{highRisk}</p>
            <p className="mt-1 text-xs text-muted">Est. recovered ${recovered.toLocaleString()}</p>
          </div>
        </div>
        <form onSubmit={(e) => void createLoan(e)} className="mb-8 grid gap-3 rounded-xl border border-line bg-card p-5 sm:grid-cols-4">
          <select
            value={borrowerId}
            onChange={(e) => setBorrowerId(e.target.value)}
            className="rounded-lg bg-background px-3 py-2 text-sm"
          >
            {borrowers.map((b) => (
              <option key={b.id} value={b.id}>
                {b.name} ({b.id})
              </option>
            ))}
          </select>
          <input
            value={principal}
            onChange={(e) => setPrincipal(e.target.value)}
            className="rounded-lg bg-background px-3 py-2 text-sm"
            placeholder="Principal"
          />
          <input
            value={rate}
            onChange={(e) => setRate(e.target.value)}
            className="rounded-lg bg-background px-3 py-2 text-sm"
            placeholder="Rate %"
          />
          <button type="submit" className="rounded-lg bg-card-high py-2 text-sm font-semibold">
            New loan
          </button>
        </form>
        {error ? <p className="mb-4 text-red-300">{error}</p> : null}
        <div className="overflow-x-auto rounded-2xl border border-line">
          <table className="w-full text-left text-sm">
            <thead className="bg-card-high text-muted">
              <tr>
                <th className="px-4 py-3">Borrower</th>
                <th className="px-4 py-3">Amount</th>
                <th className="px-4 py-3">Status</th>
                <th className="px-4 py-3">AI</th>
                <th className="px-4 py-3" />
              </tr>
            </thead>
            <tbody>
              {loans.map((l) => (
                <tr key={l.id} className="border-t border-line">
                  <td className="px-4 py-3 font-mono text-xs">{l.borrower_id}</td>
                  <td className="px-4 py-3">${l.principal.toLocaleString()}</td>
                  <td className="px-4 py-3 capitalize">{l.status}</td>
                  <td className="px-4 py-3">{recLabel(l.ai_recommendation)}</td>
                  <td className="px-4 py-3">
                    <button
                      type="button"
                      className="text-secondary"
                      onClick={() => void recommend(l.id)}
                    >
                      Recommend
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </main>
    </div>
  );
}
