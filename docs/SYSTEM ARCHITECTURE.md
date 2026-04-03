# Smart Loan Recovery — Mermaid diagrams

These diagrams match this repository: **Actix Web** API, **rusqlite** / **SQLite**, **actix-session** cookie sessions, **frontend** (and curl/Postman) as HTTP clients, and **Docker** (`Dockerfile` multi-stage build, port **3000**).

---

## Figure 1 — System architecture

```mermaid
graph TB
  subgraph clients[HTTP clients]
    FE[Web frontend]
    CURL[curl or Postman]
  end
  subgraph docker[Docker Debian binary]
    subgraph actix[Actix Web server]
      API[Routes auth users loans overdues recommend]
      SESS[actix-identity session cookie]
      BL[UserManager LoanTracker RecoveryEngine]
    end
    DBF[SQLite rusqlite]
  end
  FE -->|HTTP JSON| API
  CURL -->|HTTP JSON| API
  API --> SESS
  API --> BL
  BL --> DBF
```

---

## Figure 2 — Use case diagram

Mermaid has no standard UML use-case shape; this flowchart lists actors and use cases as rectangles. (Figure 2 revision: no percent-comment line, plain graph TB.)

```mermaid
graph TB
  B[Borrower]
  L[Lender]
  UC_REG[Register user]
  UC_LOGIN[Log in and session]
  UC_PROFILE[View profile via GET auth-me]
  UC_B_LOANS[Borrower loan list client-filtered from loans API]
  UC_L_CREATE[Create loan via POST loans]
  UC_L_ALL[View all loans via GET loans]
  UC_L_FLAG[Flag overdue loans via POST overdues]
  UC_L_REC[Recovery recommendation via POST recommend loan id]

  B --> UC_REG
  B --> UC_LOGIN
  B --> UC_PROFILE
  B --> UC_B_LOANS

  L --> UC_REG
  L --> UC_LOGIN
  L --> UC_PROFILE
  L --> UC_L_CREATE
  L --> UC_L_ALL
  L --> UC_L_FLAG
  L --> UC_L_REC
```

---

## Figure 3 — Sequence: loan creation

Aligned with `api::create_loan` and `LoanTracker::create_loan`: lender must be authenticated with role **Lender**; loan is saved as **Active** with a **repayment schedule** derived from `months` (approx. 30-day steps).

```mermaid
sequenceDiagram
  actor Lender
  participant API as Actix API\nPOST /loans
  participant LT as LoanTracker
  participant DB as Db / SQLite

  Lender->>API: JSON body (borrower_id, lender_id,\nprincipal, interest_rate, months)
  API->>API: Validate session + Lender role
  API->>API: Parse borrower_id / lender_id as UUIDs
  API->>LT: create_loan(borrower, lender, principal,\nrate, months)
  LT->>LT: Build Loan (Active) +\nrepayment_schedule from months
  LT->>DB: save_loan(loan)
  DB->>DB: INSERT OR REPLACE loans
  DB-->>LT: Ok
  LT-->>API: loan_id (Uuid)
  API-->>Lender: 200 JSON { id }
```

---

## Figure 4 — Sequence: recovery recommendation

Aligned with `api::recommend_action` and `RecoveryEngine`: loads the loan, computes **risk_score** via `RiskScorable::calculate_risk_score`, then **rule-based** `recommend_action(risk_score, history)`.

```mermaid
sequenceDiagram
  actor Lender
  participant API as Actix API\nPOST /recommend/{loan_id}
  participant LT as LoanTracker
  participant DB as Db / SQLite
  participant RE as RecoveryEngine

  Lender->>API: loan_id (path) + session cookie
  API->>API: Validate session
  API->>LT: get_loan(loan_id)
  LT->>DB: load_loan(loan_id)
  DB-->>LT: Option<Loan>
  LT-->>API: Loan
  API->>RE: predict_default(&loan)\n(uses status-based rules)
  RE-->>API: risk_score (f64)
  API->>RE: recommend_action(risk_score, history)
  RE-->>API: RecoveryAction enum
  API-->>Lender: JSON { loan_id, risk_score,\nrecommended_action }
```

---

## Figure 5 — Class diagram

Rust types from `models.rs`, `recovery.rs`, and API response shape. The API returns recommendation fields as JSON rather than a named `struct` in code.

```mermaid
classDiagram
  class User {
    +Uuid id
    +String name
    +UserRole role
  }

  class UserRole {
    <<enumeration>>
    Borrower
    Lender
  }

  class Loan {
    +Uuid id
    +Uuid borrower_id
    +Uuid lender_id
    +f64 principal
    +f64 interest_rate
    +DateTime~Utc~ disbursement_date
    +Vec~DateTime~ repayment_schedule
    +DateTime~Utc~ start_date
    +Option~DateTime~ last_repayment_date
    +LoanStatus status
    +calculate_risk_score() f64
  }

  class LoanStatus {
    <<enumeration>>
    Active
    Overdue
    Defaulted
    Repaid
  }

  class RecoveryEngine {
    +predict_default(loan) f64
    +recommend_action(risk, history) RecoveryAction
  }

  class RecoveryAction {
    <<enumeration>>
    SendReminder
    RenegotiateTerms
    EscalateToCollection
  }

  class RecoveryRecommendation {
    +Uuid loan_id
    +f64 risk_score
    +RecoveryAction recommended_action
    <<API JSON DTO>>
  }

  User "1" --> "*" Loan : borrower_id
  User "1" --> "*" Loan : lender_id
  User --> UserRole
  Loan --> LoanStatus
  RecoveryEngine ..> Loan : reads
  RecoveryEngine ..> RecoveryAction : produces
  RecoveryRecommendation ..> RecoveryAction
```

---

## Figure 6 — Entity-relationship diagram

Logical model: `users` and `loans` in SQLite (`db.rs`). Foreign keys are conceptual (schema uses `TEXT` IDs; no `REFERENCES` clause in the current migration).

```mermaid
erDiagram
  users ||--o{ loans : "borrower_id"
  users ||--o{ loans : "lender_id"

  users {
    text id PK
    text name
    text role
  }

  loans {
    text id PK
    text borrower_id FK
    text lender_id FK
    real principal
    real interest_rate
    text disbursement_date
    text start_date
    text last_repayment_date
    text status
    text repayment_schedule
  }
```

---

## Figure 7 — Database schema

Logical relationships: each loan references two users (borrower and lender). DDL is defined in `Db::init_tables` (`src/db.rs`); SQLite does not declare `FOREIGN KEY` constraints in this project.

```mermaid
flowchart TB
  subgraph users_tbl["Table: users"]
    direction TB
    U1["id — TEXT PRIMARY KEY (UUID)"]
    U2["name — TEXT NOT NULL"]
    U3["role — TEXT NOT NULL (Borrower | Lender)"]
  end

  subgraph loans_tbl["Table: loans"]
    direction TB
    L1["id — TEXT PRIMARY KEY (UUID)"]
    L2["borrower_id — TEXT NOT NULL → users.id"]
    L3["lender_id — TEXT NOT NULL → users.id"]
    L4["principal — REAL NOT NULL"]
    L5["interest_rate — REAL NOT NULL"]
    L6["disbursement_date — TEXT NOT NULL (RFC3339)"]
    L7["start_date — TEXT NOT NULL (RFC3339)"]
    L8["last_repayment_date — TEXT NULL (RFC3339)"]
    L9["status — TEXT NOT NULL"]
    L10["repayment_schedule — TEXT NOT NULL (JSON array of datetimes)"]
  end

  users_tbl -->|"1 : N (as borrower)"| loans_tbl
  users_tbl -->|"1 : N (as lender)"| loans_tbl
```

### Column reference (SQLite)

| Table   | Column               | Type (SQLite) | Notes                                      |
|---------|----------------------|---------------|--------------------------------------------|
| `users` | `id`                 | TEXT (PK)     | UUID string                                |
| `users` | `name`               | TEXT          | NOT NULL                                   |
| `users` | `role`               | TEXT          | `Borrower` / `Lender` (debug string)       |
| `loans` | `id`                 | TEXT (PK)     | UUID string                                |
| `loans` | `borrower_id`        | TEXT          | NOT NULL, → `users.id` (logical FK)        |
| `loans` | `lender_id`          | TEXT          | NOT NULL, → `users.id` (logical FK)        |
| `loans` | `principal`          | REAL          | NOT NULL                                   |
| `loans` | `interest_rate`      | REAL          | NOT NULL (annual % in app)                 |
| `loans` | `disbursement_date`  | TEXT          | NOT NULL, RFC3339                          |
| `loans` | `start_date`         | TEXT          | NOT NULL, RFC3339                          |
| `loans` | `last_repayment_date`| TEXT          | nullable, RFC3339                          |
| `loans` | `status`             | TEXT          | Active / Overdue / Defaulted / Repaid      |
| `loans` | `repayment_schedule` | TEXT          | NOT NULL, JSON array of RFC3339 datetimes  |

---

## Figure 8 — Flowchart: loan status update process

How the codebase assigns **Active**, **Overdue**, and **Repaid**. **`Defaulted`** exists in `LoanStatus` and in the DB loader but is not set by the current automatic paths below (reserved for future rules or manual updates).

```mermaid
flowchart TB
  subgraph create["A) New loan — LoanTracker::create_loan"]
    C1[Build repayment_schedule from months]
    C2[status = Active]
    C1 --> C2
  end

  subgraph flag["B) Lender POST /overdues — LoanTracker::flag_overdues"]
    F1[Load all loans]
    F2{status == Active?}
    F3{Any installment due date in the past?}
    F4[status = Overdue, save]
    F5[Skip loan]
    F1 --> F2
    F2 -->|no| F5
    F2 -->|yes| F3
    F3 -->|yes| F4
    F3 -->|no| F5
  end

  subgraph repay["C) Repayment — LoanTracker::update_repayment (e.g. CLI in main.rs)"]
    R1[Set last_repayment_date = now]
    R2{Current time after final schedule date?}
    R3[status = Repaid]
    R4[status = Active]
    R1 --> R2
    R2 -->|yes| R3
    R2 -->|no| R4
  end
```

These are **three separate triggers** in the codebase (not one linear pipeline).

---

## Figure 9 — Flowchart: recovery recommendation process

Matches `RecoveryEngine` in `recovery.rs` and `POST /recommend/{loan_id}` in `api.rs`. The HTTP handler currently passes **repayment_history = 0**; the branches that depend only on `history` apply when that value is wired to real data later.

```mermaid
flowchart TB
  A([Lender requests recommendation]) --> B[Load loan from DB]
  B --> C[predict_default: RiskScorable::calculate_risk_score]
  C --> D{Loan status == Overdue?}
  D -->|yes| E[risk_score = 0.8]
  D -->|no| F[risk_score = 0.2]
  E --> G[recommend_action risk_score, repayment_history]
  F --> G
  G --> H{"risk_score > 0.7 OR history > 2 ?"}
  H -->|yes| I[EscalateToCollection]
  H -->|no| J{"risk_score > 0.4 OR history > 0 ?"}
  J -->|yes| K[RenegotiateTerms]
  J -->|no| L[SendReminder]
  I --> M([Return JSON: loan_id, risk_score, recommended_action])
  K --> M
  L --> M
```

---

## Figure 10 — Mockup design (future lender dashboard)

Conceptual UI when the API is fronted by a web app: list loans, show status chips, and surface recovery for overdue rows (aligns with `frontend/index.html` direction).

```mermaid
flowchart TB
  subgraph page["Lender dashboard — wireframe"]
    direction TB
    NAV["Top bar: logo · Smart Loan Recovery · user menu · Log out"]
    TITLE["Heading: My loans"]
    ACTIONS["Toolbar: Refresh · Flag overdues (POST /overdues)"]
    subgraph table["Loan list (table)"]
      direction TB
      H["Columns: ID · Borrower · Principal · Rate · Status · Actions"]
      R1["Row: … · Active · —"]
      R2["Row: … · Overdue · [Get recommendation]"]
      R3["Row: … · Repaid · —"]
    end
    PANEL["Optional side / modal: recommendation result (risk score + suggested action)"]
    H --> R1
    R1 --> R2
    R2 --> R3
  end

  NAV --> TITLE
  TITLE --> ACTIONS
  ACTIONS --> table
  R2 -.->|click| PANEL
```

**Design notes (for documentation):**

- **Status indicators**: color-coded badges (e.g. green Active, amber Overdue, grey Repaid) mapped from `LoanStatus`.
- **Get recommendation**: visible only when `status === Overdue` (or always enabled with server-side validation); calls `POST /recommend/{loan_id}` and shows `risk_score` and `recommended_action`.
- **Data**: table populated from `GET /loans` (lender view may show all loans; borrower view filters client-side by `borrower_id`).

---

## Objectives traceability (documentation)

Project goals and how they tie to code: **[OBJECTIVES_TRACEABILITY.md](OBJECTIVES_TRACEABILITY.md)**. **Figure 1** shows runtime architecture; **Figure 4** shows the recovery recommendation sequence aligned with `RecoveryEngine` and `POST /recommend/{loan_id}`.
