# Frontend

The product UI is the **Next.js** app in `frontend/`. It is the only web app.

`frontend/legacy` was the old static HTML (landing, register, borrower/lender pages) that Actix used to serve at `/app/`. The Next app never imported it. That folder is **removed**. Rust is API-only now.

## Run locally

```bash
# terminal 1 — API
cargo run

# terminal 2 — UI
cd frontend
pnpm install
pnpm dev
```

Open http://127.0.0.1:3001

The Next server proxies `/api/v1/*` to the Rust API (`API_URL`, default `http://127.0.0.1:3000`), so the browser stays same-origin and does not need CORS for those calls.

Demo IDs: borrower `DEMO`, lender `BANK`.

## Deploy (Vercel)

Set the project **Root Directory** to `frontend`, then:

| Env | Purpose |
|-----|---------|
| `API_URL` | Public or private URL of the Rust API |
| `SESSION_SECRET` | HMAC secret for the httpOnly `lw_session` cookie |

Do not put API secrets in `NEXT_PUBLIC_*` variables.

## Layout

```
frontend/
  src/app/          pages and BFF routes
  src/components/
  src/lib/          API helper + session cookie
  src/proxy.ts      role gates for /borrower and /lender
```

Stitch design mocks live in `docs/design/` (reference only).
