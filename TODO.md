# Smart Loan Recovery - Debug Errors TODO

Status: Approved plan - Frontend first, then backend compiles, test stack.

## Step 1: [✅ COMPLETED] Fix Frontend HTML Syntax
- Edit `frontend/index.html`: Added missing `>` to hero img tag.
- Verified: Diff applied cleanly, hero img now valid.

## Step 2: [PENDING] Fix Backend Compile TODOs (main.rs)
- Edit `src/main.rs`:
  - CLI CreateLoan (~line 97): Add `.to_string()` to borrower/lender UUIDs.
  - Demo (~line 209): `loan_tracker.create_loan(borrower_id.clone(), ...`
- Run `cargo check`.

## Step 3: [PENDING] Remove Debug Prints (if looping)
- Search/edit config.rs/api.rs: Remove H1/H3 hypothesis logs.
- Test `cargo run`.

## Step 4: [PENDING] Full Stack Test
- `cargo run`
- Open http://127.0.0.1:3000/app/
- Test login (DEMO borrower, BANK lender), /users, /loans.

## Step 5: [PENDING] Complete & Cleanup
- Update TODO.md ✅ marks.
- `attempt_completion`

