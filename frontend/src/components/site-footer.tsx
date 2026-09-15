import Link from "next/link";

export function SiteFooter() {
  const year = new Date().getFullYear();
  return (
    <footer className="border-t border-line bg-card">
      <div className="mx-auto grid max-w-6xl gap-10 px-6 py-14 md:grid-cols-4">
        <div className="md:col-span-1">
          <p className="font-semibold tracking-tight">LendWise Recovery</p>
          <p className="mt-3 text-sm text-muted">
            Loan tracking and recovery actions for lenders and the borrowers on their books.
          </p>
        </div>
        <div>
          <p className="font-mono text-[11px] uppercase tracking-widest text-secondary">Product</p>
          <ul className="mt-3 space-y-2 text-sm">
            <li>
              <Link href="/" className="text-muted hover:text-foreground">
                Home
              </Link>
            </li>
            <li>
              <Link href="/login" className="text-muted hover:text-foreground">
                Sign in
              </Link>
            </li>
            <li>
              <Link href="/register" className="text-muted hover:text-foreground">
                Register
              </Link>
            </li>
          </ul>
        </div>
        <div>
          <p className="font-mono text-[11px] uppercase tracking-widest text-secondary">For lenders</p>
          <ul className="mt-3 space-y-2 text-sm text-muted">
            <li>Register your company and get an account ID</li>
            <li>Issue loans to borrowers who join with that ID</li>
            <li>Flag overdues and take the next recovery step</li>
          </ul>
        </div>
        <div>
          <p className="font-mono text-[11px] uppercase tracking-widest text-secondary">For borrowers</p>
          <ul className="mt-3 space-y-2 text-sm text-muted">
            <li>Join with your lender’s account ID</li>
            <li>Sign in with Google or email</li>
            <li>See status, risk, and the recommended action</li>
          </ul>
        </div>
      </div>
      <div className="border-t border-line">
        <p className="mx-auto max-w-6xl px-6 py-4 text-xs text-muted">
          © {year} LendWise Recovery
        </p>
      </div>
    </footer>
  );
}
