"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { HealthBadge } from "@/components/health-badge";
import { LoanCalendar } from "@/components/loan-calendar";
import { apiErrorMessage, apiFetch, clearSession, getSession } from "@/lib/api";
import { formatDay, money, moneyExact } from "@/lib/format";
import type { Loan, LoanSignal, SessionUser } from "@/lib/types";

type Alert = {
  id: string;
  tone: "critical" | "warn" | "info";
  title: string;
  body: string;
};

function buildAlerts(loans: Loan[], signals: LoanSignal[]): Alert[] {
  const out: Alert[] = [];
  const today = new Date().toISOString().slice(0, 10);
  for (const loan of loans) {
    for (const inst of loan.installments ?? []) {
      const due = inst.due.slice(0, 10);
      if (inst.status === "overdue") {
        out.push({
          id: `${loan.id}-od-${due}`,
          tone: "critical",
          title: "Overdue installment",
          body: `${moneyExact(inst.expected)} was due ${formatDay(inst.due)}. Tell your lender if you can pay early or need room.`,
        });
      } else if (inst.status !== "paid") {
        const days = Math.round((new Date(inst.due).getTime() - Date.now()) / 86_400_000);
        if (due === today) {
          out.push({
            id: `${loan.id}-today-${due}`,
            tone: "warn",
            title: "Due today",
            body: `${moneyExact(inst.expected)} is due today.`,
          });
        } else if (days >= 0 && days <= 7) {
          out.push({
            id: `${loan.id}-soon-${due}`,
            tone: "info",
            title: "Coming up",
            body: `${moneyExact(inst.expected)} is due in ${days} day${days === 1 ? "" : "s"} (${formatDay(inst.due)}).`,
          });
        }
      }
    }
  }
  for (const s of signals) {
    if (s.kind === "can_pay_early") {
      out.push({
        id: s.id,
        tone: "info",
        title: "Early-pay note sent",
        body: s.message,
      });
    }
  }
  return out.slice(0, 8);
}

export default function BorrowerPage() {
  const router = useRouter();
  const [user, setUser] = useState<SessionUser | null>(null);
  const [loans, setLoans] = useState<Loan[]>([]);
  const [signals, setSignals] = useState<LoanSignal[]>([]);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [busyLoan, setBusyLoan] = useState("");

  const load = useCallback(async () => {
    try {
      const [lr, sr] = await Promise.all([apiFetch("/loans"), apiFetch("/signals")]);
      if (!lr.ok) {
        setError("Could not load loans");
        return;
      }
      setLoans((await lr.json()) as Loan[]);
      if (sr.ok) setSignals((await sr.json()) as LoanSignal[]);
      setError("");
    } catch {
      setError("Could not reach the API. Is the server running?");
    }
  }, []);

  useEffect(() => {
    void (async () => {
      const s = await getSession();
      if (!s) {
        router.replace("/login?next=/borrower");
        return;
      }
      if (s.role !== "borrower") {
        router.replace("/lender");
        return;
      }
      setUser(s);
      await load();
    })();
  }, [router, load]);

  useEffect(() => {
    if (!user) return;
    const tick = () => {
      if (document.visibilityState !== "visible") return;
      void load();
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

  async function signal(loanId: string, kind: "can_pay_early" | "concern") {
    setBusyLoan(loanId);
    setError("");
    try {
      const proposed =
        kind === "can_pay_early" ? new Date(Date.now() + 3 * 86_400_000).toISOString() : undefined;
      const res = await apiFetch(`/loans/${encodeURIComponent(loanId)}/signals`, {
        method: "POST",
        body: JSON.stringify({
          kind,
          proposed_date: proposed,
          message:
            kind === "can_pay_early"
              ? "I can pay this loan earlier than the remaining schedule."
              : "I may struggle with the next installment and want to talk.",
        }),
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) throw new Error(apiErrorMessage(data, "Could not notify lender"));
      setNotice(
        kind === "can_pay_early"
          ? "Your lender can see that you are ready to pay early."
          : "Your lender has been flagged that you need to talk.",
      );
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not notify lender");
    } finally {
      setBusyLoan("");
    }
  }

  const alerts = useMemo(() => buildAlerts(loans, signals), [loans, signals]);

  if (!user) return <p className="p-8 text-muted">Loading…</p>;

  return (
    <div className="min-h-screen bg-background px-6 py-10">
      <div className="mx-auto max-w-5xl">
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

        {error ? <p className="mb-4 text-red-300">{error}</p> : null}
        {notice ? <p className="mb-4 text-sm text-emerald-300">{notice}</p> : null}

        {alerts.length > 0 ? (
          <div className="mb-8 space-y-2">
            {alerts.map((a) => (
              <div
                key={a.id}
                className={`rounded-xl border px-4 py-3 text-sm ${
                  a.tone === "critical"
                    ? "border-red-500/30 bg-red-500/10"
                    : a.tone === "warn"
                      ? "border-amber-500/30 bg-amber-500/10"
                      : "border-line bg-card"
                }`}
              >
                <p className="font-semibold">{a.title}</p>
                <p className="mt-1 text-muted">{a.body}</p>
              </div>
            ))}
          </div>
        ) : (
          <p className="mb-8 rounded-xl border border-line bg-card px-4 py-3 text-sm text-muted">
            No payment alerts right now. The calendar still shows every upcoming installment.
          </p>
        )}

        <div className="mb-8">
          <LoanCalendar loans={loans} />
        </div>

        <div className="overflow-hidden rounded-2xl border border-line">
          <table className="w-full text-left text-sm">
            <thead className="bg-card-high text-muted">
              <tr>
                <th className="px-4 py-3">Loan</th>
                <th className="px-4 py-3">Outstanding</th>
                <th className="px-4 py-3">Health</th>
                <th className="px-4 py-3">Next due</th>
                <th className="px-4 py-3">Tell lender</th>
              </tr>
            </thead>
            <tbody>
              {loans.length === 0 ? (
                <tr>
                  <td className="px-4 py-6 text-muted" colSpan={5}>
                    No loans yet. Your lender can add you and issue a loan from their dashboard.
                  </td>
                </tr>
              ) : (
                loans.map((l) => (
                  <tr key={l.id} className="border-t border-line">
                    <td className="px-4 py-3 font-mono text-xs">{l.id.slice(0, 8)}</td>
                    <td className="px-4 py-3">{money(l.outstanding_amount ?? l.principal)}</td>
                    <td className="px-4 py-3">
                      <HealthBadge band={l.health_band ?? "watch"} score={l.health_score} />
                    </td>
                    <td className="px-4 py-3 text-muted">
                      {l.next_due ? formatDay(l.next_due) : "—"}
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex flex-wrap gap-2">
                        <button
                          type="button"
                          disabled={busyLoan === l.id}
                          className="text-secondary"
                          onClick={() => void signal(l.id, "can_pay_early")}
                        >
                          Pay early
                        </button>
                        <button
                          type="button"
                          disabled={busyLoan === l.id}
                          className="text-muted"
                          onClick={() => void signal(l.id, "concern")}
                        >
                          Need time
                        </button>
                      </div>
                    </td>
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
