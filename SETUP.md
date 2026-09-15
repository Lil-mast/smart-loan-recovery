# Setup Guide

This document walks through installing, configuring, and running **LendWise Recovery** on a local machine, in Docker, and on Fly.io.

The Rust process is the **API** (JSON). The web UI is the Next.js app in `frontend/` on port **3001** (`pnpm dev`), which proxies `/api/v1/*` to `API_URL` (default `http://127.0.0.1:3000`). Always start the Rust binary from the **repository root**.

## Prerequisites

| Tool | Version / notes |
|------|-----------------|
| [Rust](https://www.rust-lang.org/tools/install) | **1.92 or later** (matches the Docker builder image) |
| Git | Any recent version |
| SQLite | Bundled with the `rusqlite` crate; no system SQLite install required |
| Docker | Optional, for container builds |
| [Fly CLI](https://fly.io/docs/flyctl/install/) | Optional, for production deploys |
| A [Firebase](https://console.firebase.google.com/) project | Optional; demo login works without it |

Install Rust with rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version
```

## Clone the repository

```bash
git clone <repository-url>
cd smart-loan-recovery
```

## Environment files

The app loads two files from the **current working directory**:

1. `.env` — server, database, and session settings (`dotenv::dotenv()`)
2. `.env.firebase` — Firebase + JWT settings (`dotenv::from_filename(".env.firebase")`)

Neither file is committed (see `.gitignore`). Missing files are allowed: the server still starts, using defaults and a Firebase fallback so demo mode works.

### `.env` (optional)

Create `.env` in the repo root if you need to override defaults:

```bash
# Bind address (use 0.0.0.0 to accept connections from other machines / Docker)
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# SQLite file path (created automatically)
DATABASE_URL=loans.db

# Cookie session key. Must be non-empty. Omit this variable to use the built-in
# development default. In production, set a long random value:
#   openssl rand -hex 32
SESSION_SECRET=

# Set to "production" so session cookies are marked Secure (HTTPS)
# RUST_ENV=production

# Request logs
RUST_LOG=info
```

**Do not** set `SESSION_SECRET` to an empty string. Either omit it or set a real secret.

| Variable | Default | Purpose |
|----------|---------|---------|
| `SERVER_HOST` | `127.0.0.1` | Listen address |
| `SERVER_PORT` | `3000` | Listen port |
| `DATABASE_URL` | `loans.db` | SQLite path |
| `SESSION_SECRET` | built-in dev key | Cookie encryption |
| `RUST_ENV` | unset | `production` enables Secure cookies |
| `RUST_LOG` | unset | `info` / `debug` logging |

### Firebase (optional)

Copy the example and fill in values from Firebase Console → Project settings → General:

```bash
cp .env.firebase.example .env.firebase
```

```bash
FIREBASE_PROJECT_ID=your-project-id
FIREBASE_API_KEY=your-web-api-key
FIREBASE_AUTH_DOMAIN=your-project-id.firebaseapp.com
FIREBASE_STORAGE_BUCKET=your-project-id.appspot.com

# openssl rand -hex 32
JWT_SECRET=replace-with-at-least-32-characters
JWT_EXPIRATION_HOURS=24
REFRESH_TOKEN_EXPIRATION_DAYS=7

# Optional admin SDK (token verification / custom tokens)
# FIREBASE_SERVICE_ACCOUNT_KEY_PATH=/absolute/path/to/serviceAccount.json
# FIREBASE_SERVICE_ACCOUNT_JSON_BASE64=...

FRONTEND_URL=http://localhost:3000
RUST_LOG=info
```

In Firebase Console, enable **Authentication → Email/Password** and, if you use Google Sign-In, **Google**.

Without `.env.firebase`, the server logs a warning and continues. Session-based demo login and the REST API still work.

## Local development

From the repository root:

```bash
cargo run
```

With logs:

```bash
RUST_LOG=info cargo run
```

First compile can take several minutes. Subsequent builds are incremental.

### URLs

| URL | What it is |
|-----|------------|
| http://127.0.0.1:3000/ | JSON API status |
| http://127.0.0.1:3001/ | Next.js UI (`pnpm dev` in `frontend/`) |

CORS allows localhost **3001** (and Vercel) so the Next BFF and optional direct browser calls can reach the API. Prefer the Next proxy (`/api/v1`) so cookies stay on the UI origin.

### Demo data

On first start, if the `loans` table is empty, the database seeds sample lenders (M-shwari, Branch, Tala, and others), a demo borrower, a demo lender, and a sample loan so the UI is usable immediately.

| Role | User id | Name |
|------|---------|------|
| Borrower | `DEMO` | Demo Borrower |
| Lender | `BANK` | Demo Lender |

The SQLite file is `loans.db` in the working directory (unless you change `DATABASE_URL`). JSON backups `users_backup.json` and `loans_backup.json` may be written by CLI demo commands.

### CLI (no HTTP server)

If you pass a subcommand, the process runs that command and exits instead of starting the web server:

```bash
cargo run -- --help

cargo run -- register-user --name "Ada" --role borrower
cargo run -- register-user --name "Bank Corp" --role lender

cargo run -- create-loan \
  --borrower-id <borrower-uuid> \
  --lender-id <lender-uuid> \
  --principal 10000 \
  --interest-rate 5.5 \
  --months 12

cargo run -- flag-overdues
cargo run -- recommend --loan-id <loan-uuid>
cargo run -- demo
```

## Tests

```bash
cargo test
cargo test --test integration_tests
```

## Docker

Build and run (port **3000**, host `0.0.0.0` inside the image):

```bash
docker build -t smart-loan-recovery .
docker run --rm -p 3000:3000 smart-loan-recovery
```

Then the API is at http://127.0.0.1:3000/. Run the Next app separately (`cd frontend && pnpm dev`) for the UI.

The image is API-only (no static HTML). Persist SQLite across restarts:

```bash
mkdir -p data
docker run --rm -p 3000:3000 \
  -e DATABASE_URL=/data/loans.db \
  -v "$(pwd)/data:/data" \
  smart-loan-recovery
```

Pass Firebase secrets with `-e` or `--env-file .env.firebase`. Do not bake `.env*` files into the image (they are listed in `.dockerignore`).

## Fly.io

This repo already includes `fly.toml` (`app = 'lendwise-recovery'`, HTTP on port **3000**).

```bash
fly auth login
fly deploy
```

Set production secrets (never commit them):

```bash
fly secrets set SESSION_SECRET="$(openssl rand -hex 32)"
fly secrets set RUST_ENV=production
# Optional Firebase
fly secrets set FIREBASE_PROJECT_ID=... FIREBASE_API_KEY=... JWT_SECRET=...
```

Pushes to `main` deploy via `.github/workflows/fly-deploy.yml` when `FLY_API_TOKEN` is configured as a GitHub secret.

Live demo: https://lendwise-recovery.fly.dev/

## Verify the install

```bash
curl -s http://127.0.0.1:3000/ | python -m json.tool
```

You should see `"message": "Smart Loan Recovery API is running!"` and a list of endpoints.

Open http://127.0.0.1:3001 after `pnpm dev` in `frontend/` and confirm the landing page loads.

## Troubleshooting

| Symptom | What to check |
|---------|----------------|
| Session / login errors | `SESSION_SECRET` must not be empty. |
| Firebase init error in logs | Missing or invalid `.env.firebase`. Demo mode still runs. |
| Port already in use | Change `SERVER_PORT` or stop the other process. |
| Cookies not sent on HTTPS | Set `RUST_ENV=production`. |
| CORS errors | Use the Next app on port 3001 (`/api/v1` proxy). |

## Related docs

- [README.md](README.md) — features, API overview, user guide
- [FIREBASE_AUTH_TESTING.md](docs/FIREBASE_AUTH_TESTING.md) — Postman collection for `/auth/*`
- [docs/FRONTEND_GUIDE.md](docs/FRONTEND_GUIDE.md) — UI usage
- [docs/CONTAINERIZATION.md](docs/CONTAINERIZATION.md) — Docker details
- [docs/MAINTENANCE.md](docs/MAINTENANCE.md) — operations
