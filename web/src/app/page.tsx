import Link from "next/link";
import { SiteHeader } from "@/components/site-header";

export default function HomePage() {
  return (
    <div className="min-h-screen bg-background">
      <SiteHeader />
      <main className="relative overflow-hidden pt-16">
        <div className="pointer-events-none absolute left-1/2 top-0 h-[420px] w-[800px] -translate-x-1/2 bg-primary-container/20 blur-[140px]" />
        <section className="mx-auto max-w-6xl px-6 pb-20 pt-20 text-center">
          <p className="mb-6 inline-flex items-center gap-2 rounded-full bg-card-high px-4 py-1.5 font-mono text-[11px] uppercase tracking-widest text-secondary">
            Predictive debt recovery
          </p>
          <h1 className="mx-auto max-w-4xl text-4xl font-extrabold leading-tight tracking-tight md:text-6xl">
            AI-Driven Loan Recovery for the{" "}
            <span className="bg-gradient-to-r from-secondary via-primary to-primary-container bg-clip-text text-transparent">
              Modern Lender
            </span>
          </h1>
          <p className="mx-auto mt-6 max-w-2xl text-lg text-muted">
            Track risk, flag overdues, and get the next recovery action without leaving the dashboard.
          </p>
          <div className="mt-10 flex flex-col items-center justify-center gap-3 sm:flex-row">
            <Link
              href="/register"
              className="rounded-lg bg-primary-container px-8 py-3 font-semibold text-on-primary-container"
            >
              Deploy System
            </Link>
            <Link href="/login" className="rounded-lg bg-card-high px-8 py-3 font-semibold text-foreground">
              Sign in
            </Link>
          </div>
          <div className="mt-16 rounded-2xl border border-line bg-card/80 p-6 text-left shadow-2xl">
            <div className="mb-4 flex items-center justify-between font-mono text-[11px] text-muted">
              <span>CORE://RECOVERY</span>
              <span className="text-secondary">LIVE</span>
            </div>
            <svg viewBox="0 0 700 160" className="h-40 w-full" aria-hidden>
              <defs>
                <linearGradient id="curve" x1="0" x2="0" y1="0" y2="1">
                  <stop offset="0%" stopColor="#4cd7f6" stopOpacity="0.35" />
                  <stop offset="100%" stopColor="#4cd7f6" stopOpacity="0" />
                </linearGradient>
              </defs>
              <path
                d="M0 120 C80 110, 140 70, 220 80 S360 40, 460 55 S620 20, 700 28"
                fill="none"
                stroke="#4cd7f6"
                strokeWidth="2.5"
              />
              <path
                d="M0 120 C80 110, 140 70, 220 80 S360 40, 460 55 S620 20, 700 28 L700 160 L0 160 Z"
                fill="url(#curve)"
              />
            </svg>
          </div>
        </section>
        <section className="mx-auto grid max-w-6xl grid-cols-1 gap-4 px-6 pb-20 sm:grid-cols-3">
          {[
            ["01", "Register", "Create a borrower or lender workspace."],
            ["02", "Track", "See status, balance, and risk on one screen."],
            ["03", "Recover", "Remind, renegotiate, or escalate."],
          ].map(([n, t, d]) => (
            <div key={n} className="rounded-2xl border border-line bg-card p-6">
              <p className="font-mono text-sm text-secondary">{n}</p>
              <h2 className="mt-2 text-xl font-bold">{t}</h2>
              <p className="mt-2 text-sm text-muted">{d}</p>
            </div>
          ))}
        </section>
      </main>
    </div>
  );
}
