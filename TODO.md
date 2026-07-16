# Smart Loan Recovery - Fix Status

## COMPLETED (this session)

### HIGH Priority
- [x] **Loan model type fix**: `borrower_id`/`lender_id` changed from `uuid::Uuid` to `String` to match 4-char user ID system
- [x] **db.rs**: Extracted shared `row_to_loan()` function (removed 50-line duplication), updated save/load for String IDs
- [x] **loan.rs**: Removed UUID parsing in `create_loan()`, fixed `unwrap()` on empty repayment schedule
- [x] **api.rs**: Fixed loan filters (were `retain(|_| false)` no-ops), fixed `LoanApiJson` types, added CORS origin restriction (was `allow_any_origin()`), added `demo_login` endpoint, fixed `panic!` at Firebase init failure
- [x] **main.rs**: Removed UUID parsing in CLI `CreateLoan` command
- [x] **auth**: Password validation now uses `validate_password_strength()` (was `len < 6`), `Mutex::unwrap()` fixed to handle poison, `AuthState` fallback on Firebase init failure, `JwtService::from_secret()` added for non-env-var usage
- [x] **FirebaseAuthService**: Added `Default` impl for fallback mode
- [x] **Frontend**: Fixed demo IDs (`DEMO`/`BANK` not `ABCD`/`WXYZ`), login endpoint changed to `/auth/demo-login`, session identity set on demo login

### MEDIUM Priority
- [x] **Session cookie**: `cookie_secure` now based on `RUST_ENV=production`
- [x] **Dockerignore**: Added `*.db`, `.env*`, `*.json` backup files
- [x] **Tests**: Fixed shared DB file collision using temp files with unique names

## PENDING
- [ ] Frontend rebuild (see architecture suggestion)
- [ ] Rate limiting on auth endpoints
- [ ] Token blacklist persistence (currently in-memory only)
- [ ] https-only enforcement in production
