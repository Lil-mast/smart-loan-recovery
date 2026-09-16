import type { ReactNode } from "react";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { BrandMark } from "@/components/marketing/brand-mark";
import { site } from "@/lib/site";

export function AuthShell({
  title,
  switchPrompt,
  switchHref,
  switchLabel,
  children,
}: {
  title: string;
  switchPrompt: string;
  switchHref: string;
  switchLabel: string;
  children: ReactNode;
}) {
  return (
    <div className="flex min-h-screen items-center justify-center bg-[radial-gradient(1200px_circle_at_20%_20%,#7eb6ff,transparent_45%),radial-gradient(900px_circle_at_90%_80%,#0b3a6a,transparent_50%),linear-gradient(160deg,#123a5c,#1b6fd8_55%,#e8f1ff)] p-4 md:p-8">
      <div className="grid w-full max-w-5xl overflow-hidden rounded-[2rem] bg-white shadow-[0_40px_80px_-24px_rgba(11,58,106,0.45)] md:grid-cols-2">
        <div className="relative hidden min-h-[560px] md:block">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img
            src="/auth-panel.jpg"
            alt=""
            className="absolute inset-0 h-full w-full object-cover object-[center_20%]"
          />
          <div className="absolute inset-0 bg-gradient-to-t from-[#0b3a6a]/80 via-[#123a5c]/35 to-[#1b6fd8]/20" />
          <div className="absolute top-8 left-1/2 -translate-x-1/2 text-white">
            <BrandMark className="h-10 w-14" />
          </div>
        </div>
        <div className="flex min-w-0 flex-col px-8 py-8 md:px-12 md:py-10">
          <Link
            href="/"
            className="inline-flex w-fit items-center text-muted-foreground hover:text-foreground"
            aria-label={`Back to ${site.shortName} home`}
          >
            <ArrowLeft className="size-4" />
          </Link>
          <h1 className="mt-8 text-4xl font-semibold tracking-tight text-[#0b3a6a]">{title}</h1>
          <p className="mt-2 text-sm text-muted-foreground">
            {switchPrompt}{" "}
            <Link href={switchHref} className="font-medium text-foreground underline underline-offset-4">
              {switchLabel}
            </Link>
          </p>
          <div className="mt-8 flex-1">{children}</div>
        </div>
      </div>
    </div>
  );
}
