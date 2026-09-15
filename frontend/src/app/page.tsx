import Link from "next/link";
import { SiteFooter } from "@/components/site-footer";
import { SiteHeader } from "@/components/site-header";

export default function HomePage() {
  return (
    <div className="min-h-screen bg-background">
      <SiteHeader />
      <main className="relative overflow-hidden pt-16">
        <div className="pointer-events-none absolute left-1/2 top-0 h-[420px] w-[800px] -translate-x-1/2 bg-primary-container/20 blur-[140px]" />
        <section className="mx-auto max-w-6xl px-6 pb-16 pt-20 text-center">
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
            Track each loan, flag overdues, and get the next recovery action — remind, renegotiate,
            or escalate — in one workspace for your company and your borrowers.
          </p>
          <div className="mt-10 flex flex-col items-center justify-center gap-3 sm:flex-row">
            <Link
              href="/register"
              className="rounded-lg bg-primary-container px-8 py-3 font-semibold text-on-primary-container"
            >
              Register
            </Link>
            <Link href="/login" className="rounded-lg bg-card-high px-8 py-3 font-semibold text-foreground">
              Sign in
            </Link>
          </div>
        </section>

        <section className="mx-auto grid max-w-6xl grid-cols-1 gap-4 px-6 pb-16 md:grid-cols-2">
          <div className="rounded-2xl border border-line bg-card p-8 text-left">
            <p className="font-mono text-[11px] uppercase tracking-widest text-secondary">Lenders</p>
            <h2 className="mt-2 text-2xl font-bold">Run recovery for your book</h2>
            <p className="mt-3 text-sm leading-relaxed text-muted">
              Register with your company name (no email required) and receive a 4-character account
              ID. Issue loans, watch risk scores, flag overdues, and follow the recommended next
              action. Borrowers join using that ID.
            </p>
          </div>
          <div className="rounded-2xl border border-line bg-card p-8 text-left">
            <p className="font-mono text-[11px] uppercase tracking-widest text-secondary">Borrowers</p>
            <h2 className="mt-2 text-2xl font-bold">See where you stand</h2>
            <p className="mt-3 text-sm leading-relaxed text-muted">
              Enter your lender’s account ID, then sign in with Google (or email if you prefer).
              Your dashboard lists your loans, status, and the recovery step your lender is taking.
            </p>
          </div>
        </section>

        <section className="mx-auto max-w-6xl px-6 pb-16">
          <h2 className="text-center text-2xl font-bold">What the product does</h2>
          <p className="mx-auto mt-3 max-w-2xl text-center text-sm text-muted">
            Every loan has a risk score. Overdue accounts can be flagged in bulk. Each case gets a
            recommended action so teams are not guessing the next step.
          </p>
          <div className="mt-8 grid grid-cols-1 gap-4 sm:grid-cols-3">
            {[
              [
                "01",
                "Register",
                "A lender creates a company workspace and shares their account ID. A borrower joins that book with Google or email.",
              ],
              [
                "02",
                "Track",
                "See principal, status, and risk on one screen. Empty books stay empty until you add real loans — nothing is invented.",
              ],
              [
                "03",
                "Recover",
                "Flag overdues, then remind, renegotiate terms, or escalate to collections based on the model’s recommendation.",
              ],
            ].map(([n, t, d]) => (
              <div key={n} className="rounded-2xl border border-line bg-card p-6">
                <p className="font-mono text-sm text-secondary">{n}</p>
                <h3 className="mt-2 text-xl font-bold">{t}</h3>
                <p className="mt-2 text-sm text-muted">{d}</p>
              </div>
            ))}
          </div>
        </section>

        <section className="mx-auto max-w-6xl px-6 pb-20">
          <div className="rounded-2xl border border-line bg-card/80 p-8 text-center">
            <h2 className="text-2xl font-bold">How you start</h2>
            <p className="mx-auto mt-3 max-w-xl text-sm text-muted">
              Lender: name + company → account ID → dashboard. Borrower: that ID + Google or email →
              dashboard. Optional Google on a lender account is only for faster sign-in later.
            </p>
            <Link
              href="/register"
              className="mt-6 inline-block rounded-lg bg-primary-container px-8 py-3 font-semibold text-on-primary-container"
            >
              Get started
            </Link>
          </div>
        </section>
      </main>
      <SiteFooter />
    </div>
  );
}
