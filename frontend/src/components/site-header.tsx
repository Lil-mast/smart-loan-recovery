import Link from "next/link";

export function SiteHeader({ signedIn }: { signedIn?: boolean }) {
  return (
    <header className="fixed inset-x-0 top-0 z-50 border-b border-line bg-[#0F172A]/80 backdrop-blur-xl">
      <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-6">
        <Link href="/" className="font-semibold tracking-tight text-foreground">
          LendWise
        </Link>
        <nav className="flex items-center gap-3 text-sm">
          {signedIn ? (
            <Link href="/login" className="text-muted hover:text-foreground">
              Account
            </Link>
          ) : (
            <>
              <Link href="/login" className="text-muted hover:text-foreground">
                Login
              </Link>
              <Link
                href="/register"
                className="rounded-lg bg-primary-container px-4 py-2 font-semibold text-on-primary-container"
              >
                Launch Platform
              </Link>
            </>
          )}
        </nav>
      </div>
    </header>
  );
}
