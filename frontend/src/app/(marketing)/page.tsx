import Link from "next/link";
import { Mail, Phone } from "lucide-react";
import { FaqSection } from "@/components/marketing/faq-section";
import { LiquidGlassText } from "@/components/marketing/liquid-glass-text";
import { MarqueeRow } from "@/components/marketing/marquee-row";
import { ProductSpotlight } from "@/components/marketing/product-spotlight";
import { SiteFooter } from "@/components/site-footer";
import { SiteHeader } from "@/components/site-header";
import { Button } from "@/components/ui/button";
import { site } from "@/lib/site";

const steps = [
  {
    n: "01",
    title: "Register",
    body: "A lender creates a company workspace and shares a 4-character account ID. Borrowers join that book with Google or email.",
  },
  {
    n: "02",
    title: "Track",
    body: "See principal, status, and health on one screen. Empty books stay empty until you add real loans — nothing is invented.",
  },
  {
    n: "03",
    title: "Flag overdue",
    body: "Mark accounts that have slipped the schedule. The book updates from the installment calendar, not a guess.",
  },
  {
    n: "04",
    title: "Recover",
    body: "Each case gets a recommended action so teams remind, renegotiate terms, or escalate to collections with the same score.",
  },
];

export default function HomePage() {
  return (
    <div className="min-h-screen">
      <SiteHeader />
      <main className="relative overflow-hidden pt-16">
        <div className="pointer-events-none absolute inset-x-0 top-0 h-[720px] bg-[radial-gradient(800px_circle_at_50%_-10%,rgba(126,182,255,0.55),transparent_60%),radial-gradient(600px_circle_at_80%_20%,rgba(232,241,255,0.9),transparent_55%)]" />

        <section className="relative mx-auto max-w-6xl px-6 pb-16 pt-20 text-center">
          <p className="mb-6 inline-flex items-center gap-2 rounded-full bg-card px-4 py-1.5 font-mono text-[11px] tracking-widest text-primary uppercase shadow-sm">
            Predictive debt recovery
          </p>
          <h1 className="mx-auto max-w-4xl text-5xl leading-[0.95] font-extrabold tracking-tight md:text-7xl lg:text-[5.5rem]">
            Recover more of
            <br />
            every book with
            <br />
            <LiquidGlassText>Smart AI insights</LiquidGlassText>
          </h1>
          <p className="mx-auto mt-6 max-w-2xl text-lg text-muted-foreground">
            Track each loan, flag overdues, and get the next recovery action — remind, renegotiate,
            or escalate — in one workspace for your company and your borrowers.
          </p>
          <div className="mt-10 flex flex-col items-center justify-center gap-3 sm:flex-row">
            <Button size="lg" className="h-12 rounded-full px-8" asChild>
              <Link href="/register">Get started</Link>
            </Button>
            <Button size="lg" variant="outline" className="h-12 rounded-full px-8" asChild>
              <Link href="#how-it-works">How it works</Link>
            </Button>
          </div>
        </section>

        <section className="border-y border-line bg-card/60 py-4">
          <p className="mb-3 text-center font-mono text-[11px] tracking-[0.28em] text-muted-foreground uppercase">
            Growing partnership around recovery
          </p>
          <MarqueeRow
            items={[
              "Remind",
              "Renegotiate",
              "Escalate",
              "Health score",
              "Overdue flags",
              "Account ID",
              "Google sign-in",
              "Repayment calendar",
            ]}
          />
        </section>

        <section id="features" className="mx-auto max-w-6xl scroll-mt-24 px-6 py-20">
          <p className="font-mono text-[11px] tracking-widest text-primary uppercase">Features</p>
          <h2 className="mt-2 max-w-xl text-3xl font-bold tracking-tight md:text-4xl">
            One book. A score. The next action.
          </h2>
          <p className="mt-4 max-w-2xl text-muted-foreground">
            Lenders run recovery for their company. Borrowers see where they stand. The model does
            not mix roles or invent loans.
          </p>
          <div className="mt-10">
            <ProductSpotlight />
          </div>
          <div className="mt-8 grid gap-4 md:grid-cols-2">
            <div className="rounded-2xl border border-line bg-card p-8">
              <p className="font-mono text-[11px] tracking-widest text-primary uppercase">Lenders</p>
              <h3 className="mt-2 text-2xl font-bold">Run recovery for your book</h3>
              <p className="mt-3 text-sm leading-relaxed text-muted-foreground">
                Register with your company name and receive a 4-character account ID. Issue loans,
                watch health, flag overdues, and follow the recommended next action. Borrowers join
                using that ID.
              </p>
            </div>
            <div className="rounded-2xl border border-line bg-card p-8">
              <p className="font-mono text-[11px] tracking-widest text-primary uppercase">Borrowers</p>
              <h3 className="mt-2 text-2xl font-bold">See where you stand</h3>
              <p className="mt-3 text-sm leading-relaxed text-muted-foreground">
                Enter your lender’s account ID, then sign in with Google (or email). Your dashboard
                lists loans, status, and the recovery step your lender is taking.
              </p>
            </div>
          </div>
        </section>

        <section id="how-it-works" className="scroll-mt-24 bg-card py-20">
          <div className="mx-auto max-w-6xl px-6">
            <p className="font-mono text-[11px] tracking-widest text-primary uppercase">How it works</p>
            <h2 className="mt-2 max-w-xl text-3xl font-bold tracking-tight md:text-4xl">
              Explore a simple, honest process
            </h2>
            <p className="mt-4 max-w-2xl text-muted-foreground">
              Start with a workspace, then let the installment calendar and health score drive the
              next conversation.
            </p>
            <div className="mt-10 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              {steps.map((step) => (
                <div key={step.n} className="rounded-2xl border border-line bg-background p-6">
                  <p className="font-mono text-sm text-primary">{step.n}</p>
                  <h3 className="mt-2 text-xl font-bold">{step.title}</h3>
                  <p className="mt-2 text-sm text-muted-foreground">{step.body}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="mx-auto max-w-6xl px-6 py-20">
          <p className="font-mono text-[11px] tracking-widest text-primary uppercase">Engine</p>
          <h2 className="mt-2 max-w-xl text-3xl font-bold tracking-tight md:text-4xl">
            Rust scoring, Google sign-in
          </h2>
          <p className="mt-4 max-w-2xl text-muted-foreground">
            Health bands come from `src/scoring.rs`. Google is Firebase on the API. No fake partner
            logos — these are the integrations that actually run.
          </p>
          <div className="mt-8 grid gap-4 md:grid-cols-2">
            <div className="rounded-2xl border border-line bg-card p-8">
              <h3 className="text-xl font-bold">Analytics module</h3>
              <p className="mt-2 text-sm text-muted-foreground">
                Coverage, days past due, consecutive misses, schedule lag. Bands: healthy, watch, at
                risk, critical.
              </p>
            </div>
            <div className="rounded-2xl border border-line bg-card p-8">
              <h3 className="text-xl font-bold">Identity</h3>
              <p className="mt-2 text-sm text-muted-foreground">
                Lenders: company name → account ID. Borrowers: that ID + Google or email. Same Google
                cannot be both roles.
              </p>
            </div>
          </div>
        </section>

        <section id="about" className="scroll-mt-24 bg-card py-20">
          <div className="mx-auto max-w-6xl px-6">
            <p className="font-mono text-[11px] tracking-widest text-primary uppercase">About us</p>
            <h2 className="mt-2 text-3xl font-bold tracking-tight md:text-4xl">Meet the owner</h2>
            <p className="mt-4 max-w-2xl text-muted-foreground">
              The person behind {site.name} — replace the placeholders in{" "}
              <code className="text-xs">frontend/src/lib/site.ts</code> with your name, contact, and
              bio.
            </p>
            <div className="mt-10 max-w-xl rounded-2xl border border-line bg-background p-8">
              <p className="font-mono text-[11px] tracking-widest text-primary uppercase">
                {site.owner.role}
              </p>
              <h3 className="mt-2 text-2xl font-bold">{site.owner.name}</h3>
              <p className="mt-4 text-sm leading-relaxed text-muted-foreground">{site.owner.bio}</p>
              <div className="mt-6 flex flex-col gap-2 text-sm">
                <a
                  href={`mailto:${site.owner.email}`}
                  className="inline-flex items-center gap-2 text-foreground hover:underline"
                >
                  <Mail className="size-4" />
                  {site.owner.email}
                </a>
                <a
                  href={`tel:${site.owner.phone}`}
                  className="inline-flex items-center gap-2 text-foreground hover:underline"
                >
                  <Phone className="size-4" />
                  {site.owner.phone}
                </a>
              </div>
            </div>
          </div>
        </section>

        <section id="faq" className="mx-auto max-w-3xl scroll-mt-24 px-6 py-20">
          <p className="text-center font-mono text-[11px] tracking-widest text-primary uppercase">
            FAQ
          </p>
          <h2 className="mt-2 text-center text-3xl font-bold tracking-tight md:text-4xl">
            Burning questions about LendWise
          </h2>
          <div className="mt-10">
            <FaqSection />
          </div>
        </section>

        <section id="contact" className="mx-auto max-w-6xl scroll-mt-24 px-6 pb-20">
          <div className="rounded-[2rem] bg-[#0b3a6a] px-8 py-14 text-center text-white">
            <h2 className="text-3xl font-bold tracking-tight md:text-5xl">Ready? Let’s talk.</h2>
            <p className="mx-auto mt-4 max-w-xl text-sky-100">
              Questions about a workspace, a book, or this product — write or call{" "}
              {site.owner.name}.
            </p>
            <div className="mt-8 flex flex-col items-center justify-center gap-3 sm:flex-row">
              <Button size="lg" className="h-12 rounded-full bg-white px-8 text-[#0b3a6a] hover:bg-sky-100" asChild>
                <a href={`mailto:${site.owner.email}`}>Email {site.owner.name}</a>
              </Button>
              <Button
                size="lg"
                variant="outline"
                className="h-12 rounded-full border-white/40 bg-transparent px-8 text-white hover:bg-white/10"
                asChild
              >
                <Link href="/register">Create a workspace</Link>
              </Button>
            </div>
          </div>
        </section>
      </main>
      <SiteFooter />
    </div>
  );
}
