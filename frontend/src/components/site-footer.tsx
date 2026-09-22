import Image from "next/image";
import Link from "next/link";
import { site } from "@/lib/site";

export function SiteFooter() {
  const year = new Date().getFullYear();
  return (
    <footer className="border-t border-line bg-card">
      <div className="mx-auto grid max-w-6xl gap-10 px-6 py-14 md:grid-cols-4">
        <div>
          <p className="font-semibold tracking-tight">{site.name}</p>
          <p className="mt-3 text-sm text-muted-foreground">{site.tagline}</p>
        </div>
        <div>
          <p className="font-mono text-[11px] tracking-widest text-primary uppercase">Product</p>
          <ul className="mt-3 space-y-2 text-sm">
            <li>
              <Link href="#features" className="text-muted-foreground hover:text-foreground">
                Features
              </Link>
            </li>
            <li>
              <Link href="/login" className="text-muted-foreground hover:text-foreground">
                Sign in
              </Link>
            </li>
            <li>
              <Link href="/register" className="text-muted-foreground hover:text-foreground">
                Register
              </Link>
            </li>
          </ul>
        </div>
        <div>
          <p className="font-mono text-[11px] tracking-widest text-primary uppercase">Owner</p>
          <div className="mt-3 flex items-center gap-3">
            <Image
              src={site.owner.photo}
              alt=""
              width={40}
              height={40}
              className="size-10 rounded-full object-cover object-[center_12%]"
            />
            <p className="text-sm font-medium text-foreground">{site.owner.name}</p>
          </div>
          <ul className="mt-3 space-y-2 text-sm text-muted-foreground">
            <li>{site.owner.role}</li>
            <li>{site.owner.location}</li>
            {site.owner.linktree ? (
              <li>
                <a
                  href={site.owner.linktree}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-foreground"
                >
                  Linktree
                </a>
              </li>
            ) : null}
            {site.owner.email ? (
              <li>
                <a href={`mailto:${site.owner.email}`} className="hover:text-foreground">
                  {site.owner.email}
                </a>
              </li>
            ) : null}
            {site.owner.phone ? (
              <li>
                <a href={`tel:${site.owner.phone}`} className="hover:text-foreground">
                  {site.owner.phone}
                </a>
              </li>
            ) : null}
          </ul>
        </div>
        <div>
          <p className="font-mono text-[11px] tracking-widest text-primary uppercase">Workspaces</p>
          <ul className="mt-3 space-y-2 text-sm text-muted-foreground">
            <li>Lenders get a company ID and recovery book</li>
            <li>Borrowers join with that ID via Google or email</li>
            <li>Remind, renegotiate, or escalate from one score</li>
          </ul>
        </div>
      </div>
      <div className="border-t border-line">
        <p className="mx-auto max-w-6xl px-6 py-4 text-xs text-muted-foreground">
          © {year} {site.name} · {site.owner.name}, {site.owner.location}
        </p>
      </div>
    </footer>
  );
}
