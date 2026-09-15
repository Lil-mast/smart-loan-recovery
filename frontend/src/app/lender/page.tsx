"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";
import { HealthBadge } from "@/components/health-badge";
import { apiErrorMessage, apiFetch, clearSession, getSession } from "@/lib/api";
import { money, recLabel } from "@/lib/format";
import type { ApiUser, Loan, LoanSignal, SessionUser } from "@/lib/types";

export default function LenderPage() {
  const router = useRouter();
  const [user, setUser] = useState<SessionUser | null>(null);
  const [loans, setLoans] = useState<Loan[]>([]);
  const [borrowers, setBorrowers] = useState<ApiUser[]>([]);
  const [signals, setSignals] = useState<LoanSignal[]>([]);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [updatedAt, setUpdatedAt] = useState<Date | null>(null);
  const [principal, setPrincipal] = useState("10000");
  const [rate, setRate] = useState("8.5");
  const [months, setMonths] = useState("12");
  const [borrowerId, setBorrowerId] = useState("");
  const [newBorrowerId, setNewBorrowerId] = useState("");
  const [newName, setNewName] = useState("");
  const [newEmail, setNewEmail] = useState("");
  const [newPrincipal, setNewPrincipal] = useState("");
  const [newRate, setNewRate] = useState("8.5");
  const [newMonths, setNewMonths] = useState("12");
  const [adding, setAdding] = useState(false);
  const [flagging, setFlagging] = useState(false);

  const load = useCallback(async (s: SessionUser) => {
    try {
      const [lr, br, sr] = await Promise.all([
        apiFetch("/loans"),
        apiFetch(`/users?role=borrower&lender_id=${encodeURIComponent(s.id)}`),
        apiFetch("/signals"),
      ]);
      if (!lr.ok) {
        setError("Could not load loans");
        return;
      }
      setLoans((await lr.json()) as Loan[]);
      if (br.ok) {
        const users = (await br.json()) as ApiUser[];
        setBorrowers(users);
        setBorrowerId((prev) => prev || users[0]?.id || "");
      }
      if (sr.ok) {
        setSignals((await sr.json()) as LoanSignal[]);
      }
      setUpdatedAt(new Date());
      setError("");
    } catch {
      setError("Could not reach the API. Is the server running?");
    }
  }, []);

  useEffect(() => {
    void (async () => {
      const s = await getSession();
      if (!s) {
        router.replace("/login?next=/lender");
        return;
      }
      if (s.role !== "lender") {
        router.replace("/borrower");
        return;
      }
      setUser(s);
      await load(s);
    })();
  }, [router, load]);

  useEffect(() => {
    if (!user) return;
    const tick = () => {
      if (document.visibilityState !== "visible") return;
      void load(user);
    };
    const id = window.setInterval(tick, 10_000);
    document.addEventListener("visibilitychange", tick);
    return () => {
      window.clearInterval(id);
      document.removeEventListener("visibilitychange", tick);
    };
  }, [user, load]);

  async function logout() {
    await apiFetch("/auth/logout", { method: "POST", body: "{}" });
    await clearSession();
    router.push("/");
  }

  async function flagOverdues() {
    if (!user) return;
    setFlagging(true);
    setError("");
    setNotice("");
    try {
      const res = await apiFetch("/overdues", { method: "POST" });
      const data = (await res.json().catch(() => ({}))) as {
        flagged_count?: number;
        message?: string;
        error?: string;
      };
      if (!res.ok) throw new Error(apiErrorMessage(data, "Could not flag overdues"));
      const n = Number(data.flagged_count ?? 0);
      setNotice(
        n === 0
          ? "No loans needed a status change. Health scores are already current."
          : `Flagged ${n} loan${n === 1 ? "" : "s"} as overdue or defaulted.`,
      );
      await load(user);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not flag overdues");
    } finally {
      setFlagging(false);
    }
  }

  async function addBorrower(e: React.FormEvent) {
    e.preventDefault();
    if (!user) return;
    setAdding(true);
    setError("");
    setNotice("");
    try {
      const knownId = newBorrowerId.trim().toUpperCase();
      const body: Record<string, unknown> = {};
      if (knownId) body.borrower_id = knownId;
      if (newName.trim()) body.name = newName.trim();
      if (newEmail.trim()) body.email = newEmail.trim();
      const p = Number(newPrincipal);
      if (newPrincipal.trim() && Number.isFinite(p) && p > 0) {
        body.principal = p;
        body.interest_rate = Number(newRate) || 8.5;
        body.months = Number(newMonths) || 12;
      }
      const res = await apiFetch("/borrowers", { method: "POST", body: JSON.stringify(body) });
      const data = (await res.json().catch(() => ({}))) as {
        id?: string;
        name?: string;
        loan_id?: string;
        linked?: boolean;
        message?: string;
        error?: string;
      };
      if (!res.ok) throw new Error(apiErrorMessage(data, "Could not add borrower"));
      const label = data.name?.trim() || newName.trim() || data.id || "Borrower";
      const id = data.id ?? knownId;
      setNotice(
        data.linked
          ? `${label} (${id}) is on your book${data.loan_id ? " with a loan" : ""}.`
          : `Added ${label} as ${id}${data.loan_id ? " with a loan" : ""}. Share that ID so they can sign in.`,
      );
      setNewBorrowerId("");
      setNewName("");
      setNewEmail("");
      setNewPrincipal("");
      setBorrowerId(data.id ?? knownId);
      await load(user);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not add borrower");
    } finally {
      setAdding(false);
    }
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
        months: Number(months) || 12,
      }),
    });
    if (!res.ok) {
      const data = await res.json().catch(() => ({}));
      setError(apiErrorMessage(data, "Create loan failed"));
      return;
    }
    await load(user);
  }

  const borrowersById = useMemo(() => {
    const m = new Map<string, ApiUser>();
    for (const b of borrowers) m.set(b.id, b);
    return m;
  }, [borrowers]);

  const critical = loans.filter((l) => l.health_band === "critical" || l.health_band === "at_risk").length;
  const openSignals = signals.filter((s) => s.status === "open");

  if (!user) return <p className="p-8 text-muted">Loading…</p>;

  return (
    <div className="min-h-screen bg-background lg:pl-64">
      <aside className="fixed left-0 top-0 hidden h-full w-64 flex-col justify-between border-r border-line bg-[#0a0e18] p-6 lg:flex">
        <div>
          <p className="text-lg font-bold">LendWise</p>
          <p className="font-mono text-[11px] uppercase text-secondary">Lender book</p>
          <nav className="mt-8 space-y-1 text-sm">
            <span className="block rounded-lg bg-primary-container px-3 py-2 font-semibold text-on-primary-container">
              Portfolio
            </span>
            <button
              type="button"
              onClick={() => void flagOverdues()}
              disabled={flagging}
              className="block w-full rounded-lg px-3 py-2 text-left text-muted hover:text-foreground disabled:opacity-60"
            >
              {flagging ? "Flagging…" : "Flag overdues"}
            </button>
            <button
              type="button"
              onClick={() => void logout()}
              className="block w-full rounded-lg px-3 py-2 text-left text-muted hover:text-foreground"
            >
              Sign out
            </button>
          </nav>
        </div>
        <p className="font-mono text-[11px] text-muted">
          Live score · {updatedAt ? updatedAt.toLocaleTimeString() : "—"}
        </p>
      </aside>
      <main className="px-6 py-8">
        <div className="mb-8 flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="font-mono text-xs uppercase text-secondary">Portfolio command</p>
            <h1 className="text-3xl font-bold">Lender operations</h1>
            <p className="font-mono text-sm text-muted">
              {user.name} · {user.id}
            </p>
          </div>
          <div className="flex flex-wrap gap-3">
            <button
              type="button"
              onClick={() => void flagOverdues()}
              disabled={flagging}
              className="rounded-lg bg-primary-container px-4 py-2 text-sm font-semibold text-on-primary-container disabled:opacity-60"
            >
              {flagging ? "Flagging…" : "Flag overdues"}
            </button>
            <button
              type="button"
              onClick={() => void logout()}
              className="rounded-lg border border-line px-4 py-2 text-sm font-semibold text-muted lg:hidden"
            >
              Sign out
            </button>
          </div>
        </div>

        <div className="mb-8 grid grid-cols-1 gap-4 sm:grid-cols-4">
          <div className="rounded-xl border border-line bg-card p-5">
            <p className="font-mono text-[11px] uppercase text-muted">Outstanding</p>
            <p className="mt-2 text-2xl font-semibold">
              {money(loans.reduce((s, l) => s + (l.outstanding_amount || 0), 0))}
            </p>
          </div>
          <div className="rounded-xl border border-line bg-card p-5">
            <p className="font-mono text-[11px] uppercase text-muted">Loans</p>
            <p className="mt-2 text-2xl font-semibold">{loans.length}</p>
          </div>
          <div className="rounded-xl border border-line bg-card p-5">
            <p className="font-mono text-[11px] uppercase text-muted">Stressed</p>
            <p className="mt-2 text-2xl font-semibold">{critical}</p>
          </div>
          <div className="rounded-xl border border-line bg-card p-5">
            <p className="font-mono text-[11px] uppercase text-muted">Borrower alerts</p>
            <p className="mt-2 text-2xl font-semibold">{openSignals.length}</p>
          </div>
        </div>

        <form
          onSubmit={(e) => void addBorrower(e)}
          className="mb-8 grid gap-3 rounded-2xl border border-line bg-card p-5 sm:grid-cols-2 lg:grid-cols-6"
        >
          <div className="sm:col-span-2 lg:col-span-6">
            <p className="font-mono text-[11px] uppercase tracking-widest text-secondary">Add borrower</p>
            <p className="mt-1 text-sm text-muted">
              Paste an existing 4-character borrower ID to put them on your book, or enter a name to create someone new.
            </p>
          </div>
          <input
            value={newBorrowerId}
            onChange={(e) => setNewBorrowerId(e.target.value.toUpperCase())}
            maxLength={4}
            className="rounded-lg bg-background px-3 py-2 font-mono text-sm uppercase"
            placeholder="Existing ID"
            autoComplete="off"
            spellCheck={false}
          />
          <input
            required={!newBorrowerId.trim() && !newEmail.trim()}
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            className="rounded-lg bg-background px-3 py-2 text-sm lg:col-span-2"
            placeholder="Full name"
          />
          <input
            type="email"
            value={newEmail}
            onChange={(e) => setNewEmail(e.target.value)}
            className="rounded-lg bg-background px-3 py-2 text-sm lg:col-span-2"
            placeholder="Email (optional)"
          />
          <input
            value={newPrincipal}
            onChange={(e) => setNewPrincipal(e.target.value)}
            className="rounded-lg bg-background px-3 py-2 text-sm"
            placeholder="Loan amount (optional)"
          />
          <div className="grid grid-cols-2 gap-3 lg:contents">
            <input
              value={newRate}
              onChange={(e) => setNewRate(e.target.value)}
              className="rounded-lg bg-background px-3 py-2 text-sm"
              placeholder="Rate %"
            />
            <input
              value={newMonths}
              onChange={(e) => setNewMonths(e.target.value)}
              className="rounded-lg bg-background px-3 py-2 text-sm"
              placeholder="Months"
            />
          </div>
          <button
            type="submit"
            disabled={adding}
            className="rounded-lg bg-primary-container py-2 text-sm font-semibold text-on-primary-container lg:col-span-6"
          >
            {adding ? "Adding…" : "Add to book"}
          </button>
        </form>

        <form onSubmit={(e) => void createLoan(e)} className="mb-8 grid gap-3 rounded-xl border border-line bg-card p-5 sm:grid-cols-5">
          <select
            value={borrowerId}
            onChange={(e) => setBorrowerId(e.target.value)}
            className="rounded-lg bg-background px-3 py-2 text-sm sm:col-span-2"
          >
            {borrowers.length === 0 ? <option value="">No borrowers yet</option> : null}
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
          <div className="flex gap-3">
            <input
              value={months}
              onChange={(e) => setMonths(e.target.value)}
              className="w-20 rounded-lg bg-background px-3 py-2 text-sm"
              placeholder="Mo"
            />
            <button type="submit" className="flex-1 rounded-lg bg-card-high py-2 text-sm font-semibold">
              Issue loan
            </button>
          </div>
        </form>

        {notice ? <p className="mb-4 text-sm text-emerald-300">{notice}</p> : null}
        {error ? <p className="mb-4 text-red-300">{error}</p> : null}

        {openSignals.length > 0 ? (
          <div className="mb-8 rounded-2xl border border-amber-500/30 bg-amber-500/10 p-5">
            <p className="font-mono text-[11px] uppercase tracking-widest text-amber-200">Live borrower signals</p>
            <ul className="mt-3 space-y-2 text-sm">
              {openSignals.map((s) => (
                <li key={s.id} className="flex flex-wrap justify-between gap-2">
                  <span>
                    <span className="font-semibold">
                      {borrowersById.get(s.borrower_id)?.name ?? s.borrower_id}
                    </span>{" "}
                    {s.kind === "can_pay_early" ? "can pay early" : "raised a concern"}
                    {s.proposed_date ? ` · ${s.proposed_date.slice(0, 10)}` : ""}
                    <span className="text-muted"> — {s.message}</span>
                  </span>
                  <span className="font-mono text-xs text-muted">{s.loan_id.slice(0, 8)}</span>
                </li>
              ))}
            </ul>
          </div>
        ) : null}

        <div className="overflow-x-auto rounded-2xl border border-line">
          <table className="w-full text-left text-sm">
            <thead className="bg-card-high text-muted">
              <tr>
                <th className="px-4 py-3">Borrower</th>
                <th className="px-4 py-3">Outstanding</th>
                <th className="px-4 py-3">Health</th>
                <th className="px-4 py-3">DPD</th>
                <th className="px-4 py-3">Next due</th>
                <th className="px-4 py-3">Action</th>
              </tr>
            </thead>
            <tbody>
              {loans.length === 0 ? (
                <tr>
                  <td className="px-4 py-6 text-muted" colSpan={6}>
                    No loans yet. Add a borrower, then issue a loan.
                  </td>
                </tr>
              ) : (
                loans.map((l) => (
                  <tr key={l.id} className="border-t border-line">
                    <td className="px-4 py-3">
                      <p className="font-semibold">{borrowersById.get(l.borrower_id)?.name ?? l.borrower_id}</p>
                      <p className="font-mono text-xs text-muted">{l.borrower_id}</p>
                    </td>
                    <td className="px-4 py-3">{money(l.outstanding_amount ?? l.principal)}</td>
                    <td className="px-4 py-3">
                      <HealthBadge band={l.health_band ?? "watch"} score={l.health_score} />
                    </td>
                    <td className="px-4 py-3 font-mono">{l.days_past_due ?? 0}d</td>
                    <td className="px-4 py-3 text-muted">{l.next_due ? l.next_due.slice(0, 10) : "—"}</td>
                    <td className="px-4 py-3 capitalize text-muted">{recLabel(l.ai_recommendation)}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </main>
    </div>
  );
}
