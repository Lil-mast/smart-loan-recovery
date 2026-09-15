# Roles, dashboards, and live loan health

Lenders and borrowers are **different accounts with different workspaces**. A borrower never lands on the lender book, and adding a borrower never steals the lender’s session.

| Role | Dashboard | How they get there |
|------|-----------|--------------------|
| Lender | `/lender` | Company name → 4-character account ID, then ID login (optional Google later) |
| Borrower | `/borrower` | Join with the **lender’s** ID, or the lender adds them and shares **their** ID |

`frontend/src/proxy.ts` enforces this: a borrower hitting `/lender` is sent to `/borrower`, and the reverse.

## Identity rules (do not skip)

A Google account is bound to **one** local role. If you already linked Google to a lender workspace, borrower Google with that same account is rejected. Use a different Google, email, or the borrower’s 4-character ID.

Leftover lender cookies are cleared before a new Google or email login. Linking Google on an existing lender is explicit (`link_existing`), not inferred from a stale session.

`POST /users` creates a lender (self-register) or a borrower (join). It only opens a session for **lenders**. Lender-created borrowers go through `POST /borrowers` and do not change who is signed in.

## How a borrower gets an account

1. **Self-join** — `/register` as Borrower, enter the lender’s 4-character ID, then Google or email. Continue to `/borrower`.
2. **Lender adds them** — on `/lender`, **Add borrower** (name required; email and loan optional). Share the new 4-character ID. They sign in on `/login` → Borrower → **Account ID**.

If the lender already added someone by email, a later Google/email register with the same email and lender ID **claims that row** instead of creating a duplicate.

Borrower ID login is the fastest path after a lender adds someone who has no Google yet.

## Lender dashboard (`/lender`)

- Book totals: outstanding, loan count, stressed (at-risk + critical), open borrower signals.
- **Add borrower** form: name, optional email, optional principal / rate / months (creates the loan in the same submit).
- **Issue loan** against an existing borrower on this book only.
- Table sorted by **health score** (worst first): outstanding, band, days past due, next due, recommended action.
- **Live borrower signals**: `can_pay_early` and `concern` from the borrower workspace.
- Refreshes about every 10 seconds while the tab is visible.

## Borrower dashboard (`/borrower`)

- Alerts: overdue installments, due today, due within 7 days, plus confirmation of signals they already sent.
- Month calendar of the repayment schedule (overdue / due today / upcoming / paid).
- Per loan: outstanding, health band, next due.
- **Pay early** / **Need time** — posts a signal the lender sees immediately. An open `can_pay_early` slightly **raises** health so the book is not treated as silent risk.

## Scoring (computed on every loan read)

Implementation: `src/scoring.rs`. Pure function of `(loan, now)`. Payments apply FIFO to the contractual schedule.

Monthly payment is standard amortization:

\[
M = P \cdot \frac{i(1+i)^n}{(1+i)^n - 1}
\]

where \(i\) is the monthly rate (`annual % / 1200`) and \(n\) is the number of installments. Zero coupon is \(P / n\).

Health score is **0–100 (higher is healthier)**:

| Term | Weight | Meaning |
|------|--------|---------|
| Coverage | 40 | Amount paid vs amount that should already have been paid |
| Timeliness | 30 | Exponential decay with days past due (≈45-day time constant) |
| Consistency | 20 | Consecutive missed installments (full penalty at 3) |
| Vintage | 10 | Calendar progress vs principal actually paid (lag on the clock) |

Bands:

| Score | Band | Default recovery action |
|-------|------|-------------------------|
| ≥ 80 | `healthy` | Send reminder |
| ≥ 65 | `watch` | Send reminder |
| ≥ 45 | `at_risk` | Renegotiate terms |
| < 45 | `critical` | Escalate to collection |

Live status (also persisted when it changes):

- Paid in full → `repaid`
- ≥ 90 days past due or ≥ 3 consecutive misses → `defaulted`
- Any unpaid installment already due → `overdue`
- Else → `active`

`risk_score` on the API is `1 - health_score/100` (higher = worse), so older recovery code still has a 0–1 risk.

Open `can_pay_early` adds 8 points (`apply_early_intent`) and re-bands.

GET `/loans` is scoped to the signed-in user: lenders see their book, borrowers see only their loans. Scores are evaluated at request time (`evaluated_at`).

## Signals and payments

| Method | Path | Who | Purpose |
|--------|------|-----|---------|
| POST | `/borrowers` | Lender | Add borrower (optional loan). Does **not** switch session. |
| GET | `/loans` | Either | Live health + installment schedule |
| POST | `/loans` | Lender | Issue a loan to a borrower on this book |
| POST | `/loans/{id}/payments` | Lender or owning borrower | Record a payment |
| POST | `/loans/{id}/signals` | Borrower | `can_pay_early` or `concern` |
| GET | `/signals` | Either | Open notes for this book / this borrower |
| POST | `/recommend/{id}` | Owner | Same scoring snapshot as GET `/loans` |

Signal kinds: `can_pay_early`, `concern`. Status starts as `open`.

## UI routes

```
/                 marketing
/register         pick role (borrower needs lender ID)
/login            borrower: Google / email / account ID
                  lender: account ID (optional Google if linked)
/lender           lender only
/borrower         borrower only
```

Session cookie: httpOnly `lw_session` set by the Next BFF (`frontend/src/app/api/v1/[...path]/route.ts`) on `/auth/login`, `/auth/register`, `/auth/google`, `/auth/id-login`. Creating users with `POST /users` or `POST /borrowers` does not attach that cookie (lenders then call `/auth/id-login`).

## Check it yourself

1. Register a lender → note the 4-character ID → `/lender`.
2. Add a borrower (or register a **different** Google/email with that lender ID).
3. Sign in as the borrower with **their** ID → `/borrower`. Visiting `/lender` must bounce back.
4. Use **Pay early**; it should appear under Live borrower signals on `/lender`.
5. Same Google as the lender, on the borrower tab, must error — not open the lender book.
