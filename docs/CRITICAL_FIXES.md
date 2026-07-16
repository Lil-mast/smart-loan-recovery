# Critical Fixes — Smart Loan Recovery

This document catalogs every bug and vulnerability fixed during the July 2026 audit, organized by severity.

---

## 🔴 Blocked (App Would Not Function)

### 1. Loan Model Type Mismatch (Root Cause)

**Files**: `src/models.rs`, `src/db.rs`, `src/loan.rs`, `src/api.rs`, `src/main.rs`

**Problem**: `Loan.borrower_id` and `Loan.lender_id` were typed as `uuid::Uuid`, but the user system uses 4-character alphanumeric IDs (e.g. `"DEMO"`, `"BANK"`).  
- `loan.rs:create_loan()` called `Uuid::parse_str()` on valid 4-char IDs → **panic**
- `db.rs:load_loan()` / `load_all_loans()` called `Uuid::parse_str()` on DB values like `"DEMO"` → **crash on read**
- The demo seed loan `LOAN1` was inserted with `borrower_id = "DEMO"` but could never be read back

**Fix**: Changed `Loan.borrower_id` / `Loan.lender_id` to `String` throughout the entire stack:
- `models.rs:33-35` — struct field types
- `db.rs:save_loan()` — removed `.to_string()` call on already-String values
- `db.rs:load_loan()` / `load_all_loans()` — removed `Uuid::parse_str()` for borrower/lender; stored as raw strings
- `loan.rs:create_loan()` — removed UUID parsing, passed strings directly
- `api.rs:LoanApiJson` — changed fields to `String`
- `main.rs:CreateLoan` CLI — removed UUID parsing

**Also**: Extracted the duplicate 50-line SQL→Loan mapping closure into a shared `fn row_to_loan()` in `db.rs`, eliminating ~100 lines of identical code.

---

### 2. Loan Filters Were No-Ops

**File**: `src/api.rs:232-244`

**Problem**: Both `borrower_id` and `lender_id` query parameters were parsed then discarded:
```rust
loans.retain(|_| false);   // always empties the list
```

The frontend dashboards send `?borrower_id=X` and `?lender_id=X`, so they **always received empty arrays**.

**Fix**: Implemented real filtering:
```rust
loans.retain(|loan| loan.borrower_id == b);
loans.retain(|loan| loan.lender_id == l);
```

---

### 3. Frontend Login Could Never Succeed

**Files**: `frontend/index.html`, `frontend/lenders.html`, `src/auth/handlers/auth.rs`, `src/api.rs`

**Problem**: The frontend sends `{ "user_id": "DEMO" }` to `/auth/login`, but the Firebase auth handler expects `{ "email": "...", "password": "..." }`. No endpoint accepted the frontend's format → **login always failed**.

**Fix**: 
1. Added `POST /auth/demo-login` in `src/api.rs` — accepts `{ user_id }`, looks up user by 4-char ID, returns role + name
2. The handler creates an `actix-identity` session (`Identity::login()`) so subsequent authenticated calls (`POST /loans`, `POST /overdues`, etc.) work
3. Updated `frontend/index.html` login form to POST to `/auth/demo-login`
4. Updated `frontend/lenders.html` `ensureAuthSession()` to POST to `/auth/demo-login`

---

### 4. Demo IDs Mismatched Frontend vs Database

**Files**: `frontend/index.html`, `src/db.rs`

**Problem**: Database seeds `"DEMO"` (borrower) and `"BANK"` (lender), but the login modal showed `"ABCD"` and `"WXYZ"`. Users entering the displayed IDs would always get "User not found".

**Fix**: Updated the placeholder text in `frontend/index.html:337` to show the correct demo IDs (`DEMO` / `BANK`).

---

## 🟠 High Severity

### 5. CORS Allowed Any Origin

**File**: `src/api.rs:367-372`

**Problem**: 
```rust
Cors::default()
    .allow_any_origin()
    .allow_any_method()
    .allow_any_header()
```
This allowed any website to make authenticated requests to the API (no CORS protection).

**Fix**: Restricted to:
- Empty origin (same-origin requests)
- `http://127.0.0.1:3000`
- `http://localhost:3000`
- `null` (for file:/// development)
- Added `.supports_credentials()` so session cookies work

---

### 6. Server Panics on Startup Failure

**File**: `src/api.rs:314-319`

**Problem**: Two `panic!()` calls:
- Firebase init failure → `panic!("Firebase auth initialization failed...")`
- DB connection failure → `panic!("Database connection failed")`

**Fix**: 
- Firebase failure now creates a fallback `AuthState` with empty `FirebaseAuthService` + demo `JwtService`
- DB failure left as `panic!` (can't operate without a database) but with improved logging
- Added `FirebaseAuthService::default()` impl for the fallback path
- Added `JwtService::from_secret()` for creating JWT service without environment variables

---

### 7. Weak Password Validation

**File**: `src/auth/handlers/auth.rs:26-29`

**Problem**: Registration only checked `password.len() < 6`. The existing `validate_password_strength()` in `src/auth/utils/mod.rs` required 8+ chars, uppercase, lowercase, digit, and special char — but was **never called**.

**Fix**: Replaced the weak check with a call to the existing `validate_password_strength()` function.

---

### 8. Mutex unwrap() Could Poison-Panic

**File**: `src/auth/services/mod.rs:27-34`

**Problem**: `self.revoked_tokens.lock().unwrap()` would panic if another thread poisoned the mutex, crashing the server.

**Fix**: Changed to `if let Ok(mut tokens) = self.revoked_tokens.lock()` and `.map(|tokens| ...).unwrap_or(false)` — graceful degradation on poison.

---

## 🟡 Medium Severity

### 9. Session Cookie Not Secure in Production

**File**: `src/api.rs:352`

**Problem**: `.cookie_secure(false)` was hardcoded — cookies sent over HTTP even in production.

**Fix**: Changed to `.cookie_secure(is_production)` where `is_production` is derived from `RUST_ENV=production`.

---

### 10. Test DB File Collision

**File**: `tests/integration_tests.rs`

**Problem**: All 4 integration tests shared `test_loans.db` with no cleanup — data leaked between tests and files persisted across runs.

**Fix**: Each test now creates a unique temp file via `std::env::temp_dir()` + atomic counter. Files are cleaned up on test completion.

---

### 11. Secrets Could Leak via Docker Build

**File**: `.dockerignore`

**Problem**: Only ignored `target/` and `.git/`. Environment files (`*.env*`), database files (`*.db`), and JSON backups were included in the Docker build context.

**Fix**: Added `.env*`, `*.db*`, `users_backup.json`, `loans_backup.json`, and `target/debug/` to `.dockerignore`.

---

## Summary of Changes by File

| File | Changes |
|---|---|
| `src/models.rs` | `Loan.borrower_id` / `Loan.lender_id`: `Uuid` → `String` |
| `src/db.rs` | Extracted `row_to_loan()`, removed UUID parsing, removed `.to_string()` on string fields |
| `src/loan.rs` | Removed UUID parsing, fixed `unwrap()` on empty schedule |
| `src/api.rs` | Fixed loan filters, tightened CORS, added `/auth/demo-login`, fixed panic on Firebase failure, secure session cookie |
| `src/main.rs` | Removed UUID parsing in CLI `CreateLoan` |
| `src/auth/mod.rs` | — |
| `src/auth/handlers/auth.rs` | Uses `validate_password_strength()` |
| `src/auth/services/mod.rs` | Removed `unwrap()` on mutex lock |
| `src/auth/services/jwt.rs` | Added `from_secret()` constructor |
| `src/auth/services/firebase.rs` | Added `Default` impl |
| `frontend/index.html` | Fixed demo IDs (`DEMO`/`BANK`), login endpoint → `/auth/demo-login` |
| `frontend/lenders.html` | Login endpoint → `/auth/demo-login`, removed dead `/auth/logout` call |
| `tests/integration_tests.rs` | Unique temp DB files per test |
| `.dockerignore` | Added secret/db file exclusions |
| `TODO.md` | Updated status |
