import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";
import { SESSION_COOKIE, verifySession } from "@/lib/session";

export function proxy(req: NextRequest) {
  const { pathname } = req.nextUrl;
  const session = verifySession(req.cookies.get(SESSION_COOKIE)?.value);

  if (pathname.startsWith("/lender")) {
    if (!session) {
      return NextResponse.redirect(new URL("/login?next=/lender", req.url));
    }
    if (session.role !== "lender") {
      return NextResponse.redirect(new URL("/borrower", req.url));
    }
  }

  if (pathname.startsWith("/borrower")) {
    if (!session) {
      return NextResponse.redirect(new URL("/login?next=/borrower", req.url));
    }
    if (session.role !== "borrower") {
      return NextResponse.redirect(new URL("/lender", req.url));
    }
  }

  return NextResponse.next();
}

export const proxyConfig = {
  matcher: ["/lender", "/lender/:path*", "/borrower", "/borrower/:path*"],
};

export const config = {
  matcher: ["/lender", "/lender/:path*", "/borrower", "/borrower/:path*"],
};
