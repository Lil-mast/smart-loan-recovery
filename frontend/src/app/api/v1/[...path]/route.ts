import { NextRequest, NextResponse } from "next/server";
import { SESSION_COOKIE, sessionFromUnknown, signSession, verifySession } from "@/lib/session";

const UPSTREAM = (process.env.API_URL || "https://lendwise-recovery.fly.dev").replace(/\/$/, "");

const ALLOWED_PREFIXES = [
  "/users",
  "/borrowers",
  "/loans",
  "/overdues",
  "/recommend/",
  "/signals",
  "/auth/",
  "/test",
];

function attachSession(res: NextResponse, json: unknown) {
  const session = sessionFromUnknown(json);
  if (!session) return;
  res.cookies.set(SESSION_COOKIE, signSession(session), {
    httpOnly: true,
    sameSite: "lax",
    secure: process.env.NODE_ENV === "production",
    path: "/",
    maxAge: 60 * 60 * 24 * 7,
  });
}

async function demoLoginFromUserList(body: ArrayBuffer | undefined): Promise<NextResponse | null> {
  let userId = "";
  try {
    const parsed = JSON.parse(new TextDecoder().decode(body ?? new ArrayBuffer(0))) as {
      user_id?: string;
    };
    userId = String(parsed.user_id ?? "").trim();
  } catch {
    return NextResponse.json({ error: "Invalid JSON", message: "Invalid login request" }, { status: 400 });
  }
  if (!userId) {
    return NextResponse.json(
      { error: "Missing ID", message: "Enter your 4-character user ID" },
      { status: 400 },
    );
  }

  const listRes = await fetch(`${UPSTREAM}/users`);
  if (!listRes.ok) return null;
  const users = (await listRes.json()) as Array<{ id?: string; role?: string; name?: string }>;
  const needle = userId.toUpperCase();
  const user = users.find((u) => String(u.id ?? "").toUpperCase() === needle);
  if (!user) {
    return NextResponse.json(
      { error: "Not found", message: `No account found for ID ${userId}` },
      { status: 400 },
    );
  }
  const res = NextResponse.json({
    user_id: user.id,
    role: user.role,
    name: user.name,
  });
  attachSession(res, user);
  return res;
}

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

  let upstream: Response;
  try {
    upstream = await fetch(url, {
      method: req.method,
      headers,
      body,
      redirect: "manual",
    });
  } catch (err) {
    const detail = err instanceof Error ? err.message : "upstream unreachable";
    return NextResponse.json(
      {
        error: "API is not running",
        message: `Could not reach ${UPSTREAM} (${detail}). Start the Rust API with cargo run on port 3000.`,
      },
      { status: 503 },
    );
  }

  if (
    (path === "/auth/demo-login" || path === "/auth/id-login") &&
    req.method === "POST" &&
    (upstream.status === 404 || upstream.status === 405)
  ) {
    const fallback = await demoLoginFromUserList(body);
    if (fallback) return fallback;
  }

  const outHeaders = new Headers();
  const ct = upstream.headers.get("content-type");
  if (ct) outHeaders.set("content-type", ct);

  const setCookies = upstream.headers.getSetCookie?.() ?? [];
  for (const c of setCookies) {
    outHeaders.append("set-cookie", c.replace(/;\s*Domain=[^;]*/gi, ""));
  }

  const buf = await upstream.arrayBuffer();
  const res = new NextResponse(buf, { status: upstream.status, headers: outHeaders });

  if (
    upstream.ok &&
    (path === "/auth/demo-login" ||
      path === "/auth/id-login" ||
      path === "/auth/google" ||
      path === "/auth/login" ||
      path === "/auth/register" ||
      (path === "/users" &&
        req.method === "POST" &&
        !verifySession(req.cookies.get(SESSION_COOKIE)?.value)))
  ) {
    try {
      const json: unknown = JSON.parse(new TextDecoder().decode(buf));
      attachSession(res, json);
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
