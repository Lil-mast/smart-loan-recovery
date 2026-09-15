import { NextRequest, NextResponse } from "next/server";
import { SESSION_COOKIE, sessionFromUnknown, signSession } from "@/lib/session";

const UPSTREAM = (process.env.API_URL || "https://lendwise-recovery.fly.dev").replace(/\/$/, "");

const ALLOWED_PREFIXES = [
  "/users",
  "/loans",
  "/overdues",
  "/recommend/",
  "/auth/",
  "/test",
];

function allowed(path: string): boolean {
  if (path === "/" || path === "") return true;
  return ALLOWED_PREFIXES.some((p) => path === p || path.startsWith(p));
}

async function proxy(req: NextRequest, pathParts: string[]): Promise<NextResponse> {
  const search = req.nextUrl.search;
  const path = `/${pathParts.join("/")}`;
  if (!allowed(path)) {
    return NextResponse.json({ error: "Not found" }, { status: 404 });
  }

  const url = `${UPSTREAM}${path}${search}`;
  const headers = new Headers();
  const cookie = req.headers.get("cookie");
  if (cookie) headers.set("cookie", cookie);
  const contentType = req.headers.get("content-type");
  if (contentType) headers.set("content-type", contentType);
  const auth = req.headers.get("authorization");
  if (auth) headers.set("authorization", auth);

  const body =
    req.method === "GET" || req.method === "HEAD" ? undefined : await req.arrayBuffer();

  const upstream = await fetch(url, {
    method: req.method,
    headers,
    body,
    redirect: "manual",
  });

  const outHeaders = new Headers();
  const ct = upstream.headers.get("content-type");
  if (ct) outHeaders.set("content-type", ct);

  const setCookies = upstream.headers.getSetCookie?.() ?? [];
  for (const c of setCookies) {
    outHeaders.append("set-cookie", c.replace(/;\s*Domain=[^;]*/gi, ""));
  }

  const buf = await upstream.arrayBuffer();
  const res = new NextResponse(buf, { status: upstream.status, headers: outHeaders });

  if (upstream.ok && (path === "/auth/demo-login" || path === "/auth/google")) {
    try {
      const json: unknown = JSON.parse(new TextDecoder().decode(buf));
      const session = sessionFromUnknown(json);
      if (session) {
        res.cookies.set(SESSION_COOKIE, signSession(session), {
          httpOnly: true,
          sameSite: "lax",
          secure: process.env.NODE_ENV === "production",
          path: "/",
          maxAge: 60 * 60 * 24 * 7,
        });
      }
    } catch {
      /* ignore non-JSON */
    }
  }

  if (path === "/auth/logout") {
    res.cookies.set(SESSION_COOKIE, "", { httpOnly: true, path: "/", maxAge: 0 });
  }

  return res;
}

export async function GET(
  req: NextRequest,
  ctx: { params: Promise<{ path: string[] }> },
) {
  const { path } = await ctx.params;
  return proxy(req, path ?? []);
}

export async function POST(
  req: NextRequest,
  ctx: { params: Promise<{ path: string[] }> },
) {
  const { path } = await ctx.params;
  return proxy(req, path ?? []);
}

export async function PUT(
  req: NextRequest,
  ctx: { params: Promise<{ path: string[] }> },
) {
  const { path } = await ctx.params;
  return proxy(req, path ?? []);
}

export async function DELETE(
  req: NextRequest,
  ctx: { params: Promise<{ path: string[] }> },
) {
  const { path } = await ctx.params;
  return proxy(req, path ?? []);
}
