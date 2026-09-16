import { BrandMark } from "@/components/marketing/brand-mark";

export function ProductSpotlight() {
  return (
    <div className="relative overflow-hidden rounded-[2rem] border border-line bg-card p-4 shadow-[0_40px_80px_-40px_rgba(11,58,106,0.35)] md:p-6">
      <div className="mb-4 flex items-center justify-between px-2">
        <div className="flex items-center gap-2 text-sm font-semibold">
          <BrandMark className="h-4 w-6 text-primary" />
          Lender book
        </div>
        <span className="rounded-full bg-card-high px-3 py-1 font-mono text-[11px] text-primary uppercase">
          Live health
        </span>
      </div>
      <div className="grid gap-3 md:grid-cols-3">
        {[
          ["Growth", "Remind", "On-time coverage is holding"],
          ["Sales", "Renegotiate", "Two accounts need new terms"],
          ["Efficiency", "Escalate", "Critical band — collections"],
        ].map(([label, action, note]) => (
          <div key={label} className="rounded-2xl bg-card-high p-4">
            <p className="text-xs text-muted-foreground">{label}</p>
            <p className="mt-2 text-2xl font-bold tracking-tight">{action}</p>
            <p className="mt-1 text-xs text-muted-foreground">{note}</p>
          </div>
        ))}
      </div>
      <div className="mt-3 overflow-hidden rounded-2xl bg-[#0b3a6a] p-4 text-white">
        <p className="font-mono text-[11px] tracking-widest text-sky-200 uppercase">Next action</p>
        <p className="mt-2 text-lg font-semibold">Flag overdues, then follow the score — not a guess.</p>
      </div>
    </div>
  );
}
