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

Lenders register with a **company name** and receive a 4-character account ID (optional Google). Borrowers join with that ID using **Google** (default) or **email**. There are no seeded demo companies or DEMO/BANK accounts.

## Google sign-in (Firebase)

1. Copy `.env.firebase.example` to `.env.firebase` on the **API** (repo root) and fill `FIREBASE_API_KEY`, `FIREBASE_AUTH_DOMAIN`, `FIREBASE_PROJECT_ID`.
2. Firebase Console → Authentication → Sign-in method → enable **Google**.
3. Authentication → Settings → Authorized domains: add `localhost`, `127.0.0.1`, and (if you use the LAN URL) `192.168.100.18`.
4. Restart `cargo run`. Login/register loads config from `GET /auth/config`, opens the Google popup, then posts the ID token to `POST /auth/google`. Borrowers must send the lender account ID on first Google sign-up.

Open the UI at **http://127.0.0.1:3001** (not a `file://` URL) so the popup and cookies work.

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
