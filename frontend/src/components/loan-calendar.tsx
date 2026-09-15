"use client";

import { useMemo, useState } from "react";
import type { Loan } from "@/lib/types";
import { dayKey, formatDay, moneyExact } from "@/lib/format";

const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

type DayMark = {
  status: "paid" | "overdue" | "upcoming" | "due_today";
  amount: number;
  loanId: string;
};

function marksFor(loans: Loan[], month: Date): Map<string, DayMark[]> {
  const map = new Map<string, DayMark[]>();
  const y = month.getFullYear();
  const m = month.getMonth();
  const today = dayKey(new Date().toISOString());
  for (const loan of loans) {
    for (const inst of loan.installments ?? []) {
      const due = new Date(inst.due);
      if (due.getFullYear() !== y || due.getMonth() !== m) continue;
      const key = dayKey(inst.due);
      let status: DayMark["status"];
      if (inst.status === "paid") status = "paid";
      else if (key === today) status = "due_today";
      else if (inst.status === "overdue") status = "overdue";
      else status = "upcoming";
      const list = map.get(key) ?? [];
      list.push({ status, amount: inst.expected, loanId: loan.id });
      map.set(key, list);
    }
  }
  return map;
}

const DOT: Record<DayMark["status"], string> = {
  paid: "bg-emerald-400",
  overdue: "bg-red-400",
  upcoming: "bg-sky-400",
  due_today: "bg-amber-300",
};

export function LoanCalendar({ loans }: { loans: Loan[] }) {
  const [cursor, setCursor] = useState(() => {
    const n = new Date();
    return new Date(n.getFullYear(), n.getMonth(), 1);
  });
  const [picked, setPicked] = useState<string | null>(null);

  const grid = useMemo(() => {
    const first = new Date(cursor.getFullYear(), cursor.getMonth(), 1);
    const startPad = first.getDay();
    const daysInMonth = new Date(cursor.getFullYear(), cursor.getMonth() + 1, 0).getDate();
    const cells: Array<{ day: number | null; key: string | null }> = [];
    for (let i = 0; i < startPad; i++) cells.push({ day: null, key: null });
    for (let d = 1; d <= daysInMonth; d++) {
      const dt = new Date(cursor.getFullYear(), cursor.getMonth(), d);
      const key = dayKey(new Date(dt.getTime() - dt.getTimezoneOffset() * 60000).toISOString());
      cells.push({ day: d, key });
    }
    while (cells.length % 7 !== 0) cells.push({ day: null, key: null });
    return cells;
  }, [cursor]);

  const marks = useMemo(() => marksFor(loans, cursor), [loans, cursor]);
  const selected = picked ? marks.get(picked) ?? [] : [];
  const label = cursor.toLocaleDateString(undefined, { month: "long", year: "numeric" });

  return (
    <div className="rounded-2xl border border-line bg-card p-5">
      <div className="mb-4 flex items-center justify-between">
        <div>
          <p className="font-mono text-[11px] uppercase tracking-widest text-secondary">Repayment calendar</p>
          <h2 className="text-lg font-bold">{label}</h2>
        </div>
        <div className="flex gap-2">
          <button
            type="button"
            className="rounded-lg border border-line px-3 py-1 text-sm"
            onClick={() => setCursor(new Date(cursor.getFullYear(), cursor.getMonth() - 1, 1))}
          >
            Prev
          </button>
          <button
            type="button"
            className="rounded-lg border border-line px-3 py-1 text-sm"
            onClick={() => setCursor(new Date(cursor.getFullYear(), cursor.getMonth() + 1, 1))}
          >
            Next
          </button>
        </div>
      </div>
      <div className="grid grid-cols-7 gap-1 text-center text-[11px] uppercase text-muted">
        {WEEKDAYS.map((d) => (
          <div key={d} className="py-1">
            {d}
          </div>
        ))}
      </div>
      <div className="grid grid-cols-7 gap-1">
        {grid.map((cell, i) => {
          const hits = cell.key ? marks.get(cell.key) : undefined;
          const isPicked = cell.key && picked === cell.key;
          return (
            <button
              key={`${cell.key ?? "e"}-${i}`}
              type="button"
              disabled={!cell.day}
              onClick={() => cell.key && setPicked(cell.key)}
              className={`min-h-14 rounded-lg p-1 text-left text-sm ${
                cell.day ? "bg-background hover:bg-card-high" : ""
              } ${isPicked ? "ring-1 ring-secondary" : ""}`}
            >
              {cell.day ? (
                <>
                  <span className="font-mono text-xs text-muted">{cell.day}</span>
                  {hits?.length ? (
                    <span className="mt-1 flex flex-wrap gap-0.5">
                      {hits.slice(0, 3).map((h, idx) => (
                        <span key={`${h.loanId}-${idx}`} className={`h-1.5 w-1.5 rounded-full ${DOT[h.status]}`} />
                      ))}
                    </span>
                  ) : null}
                </>
              ) : null}
            </button>
          );
        })}
      </div>
      <div className="mt-4 flex flex-wrap gap-3 text-xs text-muted">
        <span className="inline-flex items-center gap-1"><span className="h-1.5 w-1.5 rounded-full bg-red-400" /> Overdue</span>
        <span className="inline-flex items-center gap-1"><span className="h-1.5 w-1.5 rounded-full bg-amber-300" /> Due today</span>
        <span className="inline-flex items-center gap-1"><span className="h-1.5 w-1.5 rounded-full bg-sky-400" /> Upcoming</span>
        <span className="inline-flex items-center gap-1"><span className="h-1.5 w-1.5 rounded-full bg-emerald-400" /> Paid</span>
      </div>
      {picked ? (
        <div className="mt-4 rounded-xl bg-background p-3 text-sm">
          <p className="font-semibold">{formatDay(picked)}</p>
          {selected.length === 0 ? (
            <p className="mt-1 text-muted">No installments on this day.</p>
          ) : (
            <ul className="mt-2 space-y-1">
              {selected.map((m, i) => (
                <li key={`${m.loanId}-${i}`} className="flex justify-between text-muted">
                  <span className="capitalize">{m.status.replaceAll("_", " ")}</span>
                  <span className="font-mono text-foreground">{moneyExact(m.amount)}</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      ) : null}
    </div>
  );
}
