use rusqlite::{Connection, Result, params, Row};
use crate::models::{User, UserRole, Loan, LoanPayment, LoanSignal, LoanStatus};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::fs;
use std::path::Path;

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new_with_path(database_path: &str) -> Result<Self> {
        let conn = Connection::open(database_path)?;
        Self::init_tables(&conn)?;
        Ok(Db { conn })
    }

    fn init_tables(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                role TEXT NOT NULL,
                email TEXT,
                lender_id TEXT,
                organization TEXT
            )",
            [],
        )?;

        Self::migrate_users_email_column(conn)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS loans (
                id TEXT PRIMARY KEY,
                borrower_id TEXT NOT NULL,
                lender_id TEXT NOT NULL,
                principal REAL NOT NULL,
                interest_rate REAL NOT NULL,
                disbursement_date TEXT NOT NULL,
                start_date TEXT NOT NULL,
                last_repayment_date TEXT,
                status TEXT NOT NULL,
                repayment_schedule TEXT NOT NULL,
                payments TEXT NOT NULL DEFAULT '[]'
            )",
            [],
        )?;

        let _ = conn.execute("ALTER TABLE loans ADD COLUMN payments TEXT", []);

        conn.execute(
            "CREATE TABLE IF NOT EXISTS loan_signals (
                id TEXT PRIMARY KEY,
                loan_id TEXT NOT NULL,
                borrower_id TEXT NOT NULL,
                lender_id TEXT NOT NULL,
                kind TEXT NOT NULL,
                message TEXT NOT NULL,
                proposed_date TEXT,
                created_at TEXT NOT NULL,
                status TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS firebase_user_links (
                firebase_uid TEXT PRIMARY KEY,
                local_user_id TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL,
                role TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS loan_notes (
                id TEXT PRIMARY KEY,
                loan_id TEXT NOT NULL,
                author_id TEXT NOT NULL,
                kind TEXT NOT NULL,
                message TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        Self::purge_legacy_demo_data(conn)?;

        Ok(())
    }

    /// Remove hardcoded demo companies/users/loans from older databases.
    fn purge_legacy_demo_data(conn: &Connection) -> Result<()> {
        const DEMO_USER_IDS: &[&str] = &["MSHW", "BRCH", "TALA", "EAZZ", "KCBP", "DEMO", "BANK"];
        for id in DEMO_USER_IDS {
            conn.execute("DELETE FROM firebase_user_links WHERE local_user_id = ?1 COLLATE NOCASE", params![id])?;
            conn.execute("DELETE FROM users WHERE id = ?1 COLLATE NOCASE", params![id])?;
        }
        conn.execute("DELETE FROM loans WHERE id = ?1", params!["LOAN1"])?;
        conn.execute(
            "DELETE FROM loans WHERE borrower_id IN ('DEMO', 'BANK') OR lender_id IN ('DEMO', 'BANK', 'MSHW', 'BRCH', 'TALA', 'EAZZ', 'KCBP')",
            [],
        )?;
        Ok(())
    }

    fn migrate_users_email_column(conn: &Connection) -> Result<()> {
        let _ = conn.execute("ALTER TABLE users ADD COLUMN email TEXT", []);
        let _ = conn.execute("ALTER TABLE users ADD COLUMN lender_id TEXT", []);
        let _ = conn.execute("ALTER TABLE users ADD COLUMN organization TEXT", []);
        Ok(())
    }

    fn row_to_user(row: &rusqlite::Row<'_>) -> Result<User> {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let role_str: String = row.get(2)?;
        let email: Option<String> = row.get(3)?;
        let lender_id: Option<String> = row.get(4)?;
        let organization: Option<String> = row.get(5)?;

        let role = match role_str.as_str() {
            "Borrower" => UserRole::Borrower,
            "Lender" => UserRole::Lender,
            _ => return Err(rusqlite::Error::InvalidColumnType(2, "UserRole".to_string(), rusqlite::types::Type::Text)),
        };

        Ok(User {
            id,
            name,
            role,
            email,
            lender_id,
            organization,
        })
    }

    // User operations
    pub fn save_user(&self, user: &User) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO users (id, name, role, email, lender_id, organization) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &user.id,
                &user.name,
                format!("{:?}", user.role),
                &user.email,
                &user.lender_id,
                &user.organization
            ],
        )?;
        Ok(())
    }

    pub fn load_user(&self, id: &str) -> Result<Option<User>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, role, email, lender_id, organization FROM users WHERE id = ?1 COLLATE NOCASE",
        )?;
        let mut rows = stmt.query_map(params![id.trim()], Self::row_to_user)?;

        match rows.next() {
            Some(user) => Ok(Some(user?)),
            None => Ok(None),
        }
    }

    pub fn load_all_users(&self) -> Result<Vec<User>> {
        let mut stmt = self.conn.prepare("SELECT id, name, role, email, lender_id, organization FROM users")?;
        let users = stmt.query_map([], Self::row_to_user)?;
        users.collect()
    }

    #[allow(dead_code)]
    pub fn find_user_by_email_ci(&self, email: &str) -> Result<Option<User>> {
        let needle = email.trim().to_lowercase();
        if needle.is_empty() {
            return Ok(None);
        }
        let users = self.load_all_users()?;
        Ok(users.into_iter().find(|u| {
            u.email
                .as_ref()
                .map(|e| e.trim().to_lowercase() == needle)
                .unwrap_or(false)
        }))
    }

    fn row_to_loan(row: &Row<'_>) -> Result<Loan> {
        let id_str: String = row.get(0)?;
        let borrower_id: String = row.get(1)?;
        let lender_id: String = row.get(2)?;
        let principal: f64 = row.get(3)?;
        let interest_rate: f64 = row.get(4)?;
        let disbursement_date_str: String = row.get(5)?;
        let start_date_str: String = row.get(6)?;
        let last_repayment_date_str: Option<String> = row.get(7)?;
        let status_str: String = row.get(8)?;
        let repayment_schedule_json: String = row.get(9)?;
        let payments_json: String = row.get::<_, Option<String>>(10)?.unwrap_or_else(|| "[]".to_string());

        let id = Uuid::parse_str(&id_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "UUID".to_string(), rusqlite::types::Type::Text))?;

        let disbursement_date = DateTime::parse_from_rfc3339(&disbursement_date_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(5, "DateTime".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);
        let start_date = DateTime::parse_from_rfc3339(&start_date_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(6, "DateTime".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);
        let last_repayment_date = match last_repayment_date_str {
            Some(date_str) => Some(DateTime::parse_from_rfc3339(&date_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(7, "DateTime".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc)),
            None => None,
        };

        let status = match status_str.as_str() {
            "Active" => LoanStatus::Active,
            "Overdue" => LoanStatus::Overdue,
            "Defaulted" => LoanStatus::Defaulted,
            "Repaid" => LoanStatus::Repaid,
            _ => return Err(rusqlite::Error::InvalidColumnType(8, "LoanStatus".to_string(), rusqlite::types::Type::Text)),
        };

        let repayment_schedule: Vec<DateTime<Utc>> = serde_json::from_str(&repayment_schedule_json)
            .map_err(|_| rusqlite::Error::InvalidColumnType(9, "JSON".to_string(), rusqlite::types::Type::Text))?;
        let payments: Vec<LoanPayment> = serde_json::from_str(&payments_json).unwrap_or_default();

        Ok(Loan {
            id,
            borrower_id,
            lender_id,
            principal,
            interest_rate,
            disbursement_date,
            start_date,
            last_repayment_date,
            status,
            repayment_schedule,
            payments,
        })
    }

    // Loan operations
    pub fn save_loan(&self, loan: &Loan) -> Result<()> {
        let repayment_schedule_json = serde_json::to_string(&loan.repayment_schedule)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "JSON".to_string(), rusqlite::types::Type::Text))?;
        let payments_json = serde_json::to_string(&loan.payments)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "JSON".to_string(), rusqlite::types::Type::Text))?;

        self.conn.execute(
            "INSERT OR REPLACE INTO loans (id, borrower_id, lender_id, principal, interest_rate, disbursement_date, start_date, last_repayment_date, status, repayment_schedule, payments)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                loan.id.to_string(),
                loan.borrower_id,
                loan.lender_id,
                loan.principal,
                loan.interest_rate,
                loan.disbursement_date.to_rfc3339(),
                loan.start_date.to_rfc3339(),
                loan.last_repayment_date.map(|dt| dt.to_rfc3339()),
                format!("{:?}", loan.status),
                repayment_schedule_json,
                payments_json
            ],
        )?;
        Ok(())
    }

    pub fn load_loan(&self, id: Uuid) -> Result<Option<Loan>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, borrower_id, lender_id, principal, interest_rate, disbursement_date, start_date, last_repayment_date, status, repayment_schedule, COALESCE(payments, '[]')
             FROM loans WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![id.to_string()], Self::row_to_loan)?;

        match rows.next() {
            Some(loan) => Ok(Some(loan?)),
            None => Ok(None),
        }
    }

    pub fn load_all_loans(&self) -> Result<Vec<Loan>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, borrower_id, lender_id, principal, interest_rate, disbursement_date, start_date, last_repayment_date, status, repayment_schedule, COALESCE(payments, '[]')
             FROM loans"
        )?;
        let loans = stmt.query_map([], Self::row_to_loan)?;
        loans.collect()
    }

    // JSON fallback methods
    pub fn save_to_json<P: AsRef<Path>>(&self, users_path: P, loans_path: P) -> Result<()> {
        let users = self.load_all_users()?;
        let users_json = serde_json::to_string_pretty(&users)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;

        let loans = self.load_all_loans()?;
        let loans_json = serde_json::to_string_pretty(&loans)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;

        fs::write(users_path, users_json)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
        fs::write(loans_path, loans_json)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
        Ok(())
    }

    // Firebase user link methods
    pub fn create_linked_user(
        &self,
        name: String,
        email: Option<String>,
        role: UserRole,
        lender_id: Option<String>,
        organization: Option<String>,
        _firebase_uid: String,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO users (id, name, role, email, lender_id, organization) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name, format!("{:?}", role), email, lender_id, organization],
        )?;
        Ok(id)
    }

    pub fn save_user_link(&self, link: &crate::auth::models::UserLink) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO firebase_user_links (firebase_uid, local_user_id, email, role, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                link.firebase_uid,
                link.local_user_id,
                link.email,
                format!("{:?}", link.role),
                link.created_at.to_rfc3339(),
                link.updated_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn get_user_link(&self, firebase_uid: &str) -> Result<Option<crate::auth::models::UserLink>> {
        let mut stmt = self.conn.prepare(
            "SELECT firebase_uid, local_user_id, email, role, created_at, updated_at 
             FROM firebase_user_links WHERE firebase_uid = ?1"
        )?;

        let result = stmt.query_row(params![firebase_uid], |row| {
            use chrono::DateTime;
            let role_str: String = row.get(3)?;
            let role = match role_str.as_str() {
                "Borrower" => UserRole::Borrower,
                "Lender" => UserRole::Lender,
                "Admin" => UserRole::Admin,
                _ => UserRole::Borrower,
            };

            Ok(crate::auth::models::UserLink {
                firebase_uid: row.get(0)?,
                local_user_id: row.get(1)?,
                email: row.get(2)?,
                role,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4, rusqlite::types::Type::Text, Box::new(e)
                    ))?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        5, rusqlite::types::Type::Text, Box::new(e)
                    ))?.with_timezone(&Utc),
            })
        });

        match result {
            Ok(link) => Ok(Some(link)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_user_link_by_local_id(&self, local_user_id: &str) -> Result<Option<crate::auth::models::UserLink>> {
        let mut stmt = self.conn.prepare(
            "SELECT firebase_uid, local_user_id, email, role, created_at, updated_at 
             FROM firebase_user_links WHERE local_user_id = ?1"
        )?;

        let result = stmt.query_row(params![local_user_id], |row| {
            use chrono::DateTime;
            let role_str: String = row.get(3)?;
            let role = match role_str.as_str() {
                "Borrower" => UserRole::Borrower,
                "Lender" => UserRole::Lender,
                "Admin" => UserRole::Admin,
                _ => UserRole::Borrower,
            };

            Ok(crate::auth::models::UserLink {
                firebase_uid: row.get(0)?,
                local_user_id: row.get(1)?,
                email: row.get(2)?,
                role,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4, rusqlite::types::Type::Text, Box::new(e)
                    ))?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        5, rusqlite::types::Type::Text, Box::new(e)
                    ))?.with_timezone(&Utc),
            })
        });

        match result {
            Ok(link) => Ok(Some(link)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn find_unclaimed_borrower(&self, email: &str, lender_id: &str) -> Result<Option<User>> {
        let Some(user) = self.find_user_by_email_ci(email)? else {
            return Ok(None);
        };
        if user.role != UserRole::Borrower {
            return Ok(None);
        }
        let same_book = user
            .lender_id
            .as_deref()
            .map(|id| id.eq_ignore_ascii_case(lender_id))
            .unwrap_or(false);
        if !same_book {
            return Ok(None);
        }
        if self.get_user_link_by_local_id(&user.id)?.is_some() {
            return Ok(None);
        }
        Ok(Some(user))
    }

    pub fn save_signal(&self, signal: &LoanSignal) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO loan_signals (id, loan_id, borrower_id, lender_id, kind, message, proposed_date, created_at, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                signal.id,
                signal.loan_id.to_string(),
                signal.borrower_id,
                signal.lender_id,
                signal.kind,
                signal.message,
                signal.proposed_date.map(|d| d.to_rfc3339()),
                signal.created_at.to_rfc3339(),
                signal.status
            ],
        )?;
        Ok(())
    }

    pub fn load_signals(
        &self,
        borrower_id: Option<&str>,
        lender_id: Option<&str>,
        loan_id: Option<&str>,
    ) -> Result<Vec<LoanSignal>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, loan_id, borrower_id, lender_id, kind, message, proposed_date, created_at, status
             FROM loan_signals ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let loan_id_str: String = row.get(1)?;
            let proposed: Option<String> = row.get(6)?;
            let created: String = row.get(7)?;
            Ok(LoanSignal {
                id: row.get(0)?,
                loan_id: Uuid::parse_str(&loan_id_str).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e))
                })?,
                borrower_id: row.get(2)?,
                lender_id: row.get(3)?,
                kind: row.get(4)?,
                message: row.get(5)?,
                proposed_date: match proposed {
                    Some(s) => Some(
                        DateTime::parse_from_rfc3339(&s)
                            .map_err(|e| {
                                rusqlite::Error::FromSqlConversionFailure(
                                    6,
                                    rusqlite::types::Type::Text,
                                    Box::new(e),
                                )
                            })?
                            .with_timezone(&Utc),
                    ),
                    None => None,
                },
                created_at: DateTime::parse_from_rfc3339(&created)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e))
                    })?
                    .with_timezone(&Utc),
                status: row.get(8)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            let s = row?;
            if let Some(b) = borrower_id {
                if !s.borrower_id.eq_ignore_ascii_case(b) {
                    continue;
                }
            }
            if let Some(l) = lender_id {
                if !s.lender_id.eq_ignore_ascii_case(l) {
                    continue;
                }
            }
            if let Some(id) = loan_id {
                if !s.loan_id.to_string().eq_ignore_ascii_case(id) {
                    continue;
                }
            }
            out.push(s);
        }
        Ok(out)
    }

    pub fn loan_has_open_kind(&self, loan_id: &str, kind: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare(
            "SELECT 1 FROM loan_signals WHERE loan_id = ?1 AND kind = ?2 AND status = 'open' LIMIT 1",
        )?;
        match stmt.query_row(params![loan_id, kind], |_| Ok(())) {
            Ok(()) => Ok(true),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

