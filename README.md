# LendWise Recovery

[![Deployed on Fly.io](https://img.shields.io/badge/Deployed%20on-Fly.io-blue)](https://lendwise-recovery.fly.dev/)
[![Rust](https://img.shields.io/badge/Rust-1.92-orange)](https://www.rust-lang.org/)
[![Docker](https://img.shields.io/badge/Docker-Ready-blue)](https://www.docker.com/)
[![Rust Africa Hackathon](https://img.shields.io/badge/Built%20for-%23RustAfricaHackathon-brightgreen)](https://rust-africa-hackathon.io/)

🚀 **Built for #RustAfricaHackathon** 🚀

An innovative AI-enhanced loan recovery system built with Rust, featuring a secure web API, authentication, and intelligent recovery recommendations. This project demonstrates modern financial technology solutions optimized for African fintech challenges.

**Objectives, traceability to code, and a short comparison** to other recovery approaches: [docs/OBJECTIVES_TRACEABILITY.md](docs/OBJECTIVES_TRACEABILITY.md).

**Who sees which dashboard, how borrowers join, and how loan health is scored:** [docs/ROLES_AND_DASHBOARDS.md](docs/ROLES_AND_DASHBOARDS.md).

## 🌐 Live Demo

**Production URL**: https://lendwise-recovery.fly.dev/

## ✨ Features

### 🔐 **Authentication & Security**
- User registration and login system
- Session-based authentication with secure cookies
- Role-based access control (Borrowers & Lenders) — **separate dashboards**, no role mixing
- Lenders: company workspace and 4-character account ID (optional Google)
- Borrowers: Google, email, or account ID, joining with a **lender** ID (or added by the lender)

### 🏦 **Loan Management**
- Complete loan lifecycle tracking
- Live status (Active, Overdue, Defaulted, Repaid) from the installment schedule
- Contractual repayment calendar and FIFO payment application
- Principal, interest, term, outstanding, and coverage

### 🤖 **AI-Powered Recovery**
- Deterministic health score (0–100) in `src/scoring.rs`: coverage, days past due, consecutive misses, schedule lag
- Bands: healthy / watch / at risk / critical, with remind / renegotiate / escalate
- Borrower early-pay and “need time” signals visible on the lender book
- Automated overdue / default detection on each loan read

### 💾 **Data Persistence**
- SQLite database with automatic schema management
- JSON backup/restore functionality for data resilience
- UUID-based entity identification
- Thread-safe database operations

### 🐳 **Containerization & Deployment**
- Docker containerization for easy deployment across environments
- Multi-stage Docker builds for optimized images
- Production-ready configuration
- One-click deployment to Fly.io

### 🧪 **Testing & Quality**
- Comprehensive integration tests for reliability
- Automated testing pipeline
- Custom error handling with detailed error types
- Structured logging throughout the system

## 🚀 Quick Start

**Full install, environment, Docker, and Fly.io steps:** [SETUP.md](SETUP.md).

### Prerequisites
- Rust 1.92 or later
- Docker (optional, for containerized deployment)

### Local Development

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd smart-loan-recovery
   ```

2. **Run the backend** (from the repository root)
   ```bash
   cargo run
   ```
   - API root: `http://127.0.0.1:3000` (JSON status at `GET /`)
   - **Web UI:** in another terminal:
     ```bash
     cd frontend
     cp .env.example .env.local   # already points API_URL at 127.0.0.1:3000
     pnpm install
     pnpm dev
     ```
     Open `http://127.0.0.1:3001`. The Next app proxies `/api/v1/*` to the Rust API (same-origin; no browser CORS). On Vercel, set the project **Root Directory** to `frontend`, plus `API_URL` (your API host) and `SESSION_SECRET`. More: [docs/FRONTEND_GUIDE.md](docs/FRONTEND_GUIDE.md).
   - Optional: `RUST_LOG=info cargo run` for request logs
   - Ensure `.env` does not set `SESSION_SECRET` to an empty value (or omit it to use the built-in dev default)

3. **Run with Docker**
   ```bash
   docker build -t smart-loan-recovery .
   docker run -p 3000:3000 smart-loan-recovery
   ```

## 📡 API Endpoints

Full product behaviour: [docs/ROLES_AND_DASHBOARDS.md](docs/ROLES_AND_DASHBOARDS.md).

### Authentication
- `POST /auth/register` — borrower email register (needs lender ID)
- `POST /auth/login` — borrower email login
- `POST /auth/google` — Google (role must match any existing link)
- `POST /auth/id-login` — 4-character ID (lender or borrower)
- `POST /auth/logout` — clear identity
- `POST /users` — self-register (session attached for lenders only)
- `POST /borrowers` — lender adds a borrower (does not switch session)

### Loans
- `GET /loans` — scoped to the signed-in user; includes health, schedule, installments
- `POST /loans` — create a loan (lenders; borrower must be on their book)
- `POST /loans/{id}/payments` — record a payment
- `POST /loans/{id}/signals` — borrower `can_pay_early` or `concern`
- `GET /signals` — signals for this book / this borrower

### Recovery
- `POST /overdues` — persist live overdue/default flags (lenders)
- `POST /recommend/{loan_id}` — same scoring snapshot as GET `/loans`

### System
- `GET /` — API information and available endpoints

## 🔧 Configuration

The application uses environment variables for configuration:

```bash
# Server Configuration
SERVER_HOST=0.0.0.0          # Host to bind to (0.0.0.0 for all interfaces)
SERVER_PORT=3000             # Port to listen on

# Database
DATABASE_URL=loans.db        # SQLite database file path

# Security
SESSION_SECRET=your-secret-key-here  # Session encryption key
```

## 🏗️ Architecture

```
src/
├── main.rs          # Application entry point
├── api.rs           # Web API routes and handlers
├── db.rs            # Database operations
├── user.rs          # User management
├── loan.rs          # Loan operations and payments
├── scoring.rs       # Live health score (0–100) and bands
├── recovery.rs      # Recovery action enum
├── models.rs        # Data structures
├── config.rs        # Configuration management
├── error.rs         # Error handling
└── lib.rs           # Library exports
```

### Key Components

- **Actix Web**: High-performance web framework
- **SQLite + Rusqlite**: Embedded database
- **Actix Identity**: Session management
- **Actix Session**: Secure cookie sessions
- **Serde**: Serialization/deserialization
- **UUID**: Unique identifier generation
- **Chrono**: Date/time handling

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

Run integration tests:

```bash
cargo test --test integration_tests
```

## 🐳 Docker Deployment

### Build the Image
```bash
docker build -t smart-loan-recovery .
```

### Run Locally
```bash
docker run -p 3000:3000 smart-loan-recovery
```

### Deploy to Fly.io
```bash
fly launch
fly deploy
```

## 📖 User Guide

Use the Next.js UI at `http://127.0.0.1:3001` (start `cargo run` and `cd frontend && pnpm dev`).

1. **Lender:** Register with company name → keep the 4-character ID → `/lender`. Add borrowers from that page, or share the ID so they self-join.
2. **Borrower:** Register with that lender ID (Google or email), **or** sign in with the ID the lender created for them → `/borrower`.
3. One Google account is one role. The lender’s Google will not open `/borrower`.
4. On `/borrower`, use the calendar and **Pay early** / **Need time**. Those show on the lender’s signal list; health scores refresh live.

Step-by-step identity, scoring, and API notes: [docs/ROLES_AND_DASHBOARDS.md](docs/ROLES_AND_DASHBOARDS.md). UI layout: [docs/FRONTEND_GUIDE.md](docs/FRONTEND_GUIDE.md).

## 🔒 Security Features

- **Session Management**: Secure cookie-based sessions
- **Authentication**: Required for sensitive operations
- **Authorization**: Role-based access control
- **Input Validation**: Comprehensive request validation
- **Error Handling**: Secure error responses without data leakage

## 📊 Recovery Actions

Live health (0–100) maps to actions in `src/scoring.rs`:

- **Healthy / watch (≥ 65)**: send reminder
- **At risk (≥ 45)**: renegotiate terms
- **Critical (< 45)**: escalate to collection

`risk_score` on JSON is `1 - health_score/100`. Details: [docs/ROLES_AND_DASHBOARDS.md](docs/ROLES_AND_DASHBOARDS.md#scoring-computed-on-every-loan-read).

## 🤝 Contributing

**Hackathon Participants**: We welcome contributions! Follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes and test thoroughly
4. Add integration tests for new features
5. Submit a pull request with a clear description

**Issues & Ideas**: Have suggestions? Open an issue on GitHub!

## 📄 License

This project is licensed under the APACHE License - see the LICENSE file for details.

## 🌟 Built with ❤️ for #RustAfricaHackathon

Special thanks to the Rust and African tech communities for the inspiration and support.

## 🙏 Acknowledgments

- Built with Rust and Actix Web
- SQLite for data persistence
- Docker for containerization
- Fly.io for hassle-free hosting
- The Rust community for amazing tools and frameworks