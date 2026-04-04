use rusqlite::{Connection, Result, params};
use crate::models::{User, UserRole, Loan, LoanStatus};
use chrono::{DateTime, Duration, Utc};
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
                role TEXT NOT NULL
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
                repayment_schedule TEXT NOT NULL
            )",
            [],
        )?;

        Self::seed_demo_if_no_loans(conn)?;

        Ok(())
    }

    /// One sample borrower + lender + loan so `/app/` login works without a separate lender API flow.
    fn seed_demo_if_no_loans(conn: &Connection) -> Result<()> {
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM loans", [], |r| r.get(0))?;
        if n > 0 {
            return Ok(());
        }

        let now = Utc::now();
        let schedule = serde_json::to_string(&vec![now + Duration::days(30), now + Duration::days(60)])
            .map_err(|e| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(e))
            })?;

        conn.execute(
            "INSERT OR IGNORE INTO users (id, name, role, email) VALUES (?1, ?2, ?3, ?4)",
            params![
                "11111111-1111-4111-8111-111111111111",
                "Demo Borrower",
                "Borrower",
                "demo.borrower@lendwise.test"
            ],
        )?;

        conn.execute(
            "INSERT OR IGNORE INTO users (id, name, role, email) VALUES (?1, ?2, ?3, ?4)",
            params![
                "22222222-2222-4222-8222-222222222222",
                "Demo Lender",
                "Lender",
                "demo.lender@lendwise.test"
            ],
        )?;

        conn.execute(
            "INSERT OR IGNORE INTO loans (id, borrower_id, lender_id, principal, interest_rate, disbursement_date, start_date, last_repayment_date, status, repayment_schedule)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8, ?9)",
            params![
                "aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa",
                "11111111-1111-4111-8111-111111111111",
                "22222222-2222-4222-8222-222222222222",
                24_850.0_f64,
                8.4_f64,
                now.to_rfc3339(),
                now.to_rfc3339(),
                "Active",
                schedule
            ],
        )?;

        Ok(())
    }

    fn migrate_users_email_column(conn: &Connection) -> Result<()> {
        let _ = conn.execute("ALTER TABLE users ADD COLUMN email TEXT", []);
        Ok(())
    }

    fn row_to_user(row: &rusqlite::Row<'_>) -> Result<User> {
        let id_str: String = row.get(0)?;
        let name: String = row.get(1)?;
        let role_str: String = row.get(2)?;
        let email: Option<String> = row.get(3)?;

        let id = Uuid::parse_str(&id_str).map_err(|_| {
            rusqlite::Error::InvalidColumnType(0, "UUID".to_string(), rusqlite::types::Type::Text)
        })?;
        let role = match role_str.as_str() {
            "Borrower" => UserRole::Borrower,
            "Lender" => UserRole::Lender,
            _ => {
                return Err(rusqlite::Error::InvalidColumnType(
                    2,
                    "UserRole".to_string(),
                    rusqlite::types::Type::Text,
                ))
            }
        };

        Ok(User {
            id,
            name,
            role,
            email,
        })
    }

    // User operations
    pub fn save_user(&self, user: &User) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO users (id, name, role, email) VALUES (?1, ?2, ?3, ?4)",
            params![
                user.id.to_string(),
                user.name,
                format!("{:?}", user.role),
                user.email
            ],
        )?;
        Ok(())
    }

    pub fn load_user(&self, id: Uuid) -> Result<Option<User>> {
        let mut stmt = self.conn.prepare("SELECT id, name, role, email FROM users WHERE id = ?1")?;
        let mut rows = stmt.query_map(params![id.to_string()], Self::row_to_user)?;

        match rows.next() {
            Some(user) => Ok(Some(user?)),
            None => Ok(None),
        }
    }

    pub fn load_all_users(&self) -> Result<Vec<User>> {
        let mut stmt = self.conn.prepare("SELECT id, name, role, email FROM users")?;
        let users = stmt.query_map([], Self::row_to_user)?;
        users.collect()
    }

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

    // Loan operations
    pub fn save_loan(&self, loan: &Loan) -> Result<()> {
        let repayment_schedule_json = serde_json::to_string(&loan.repayment_schedule)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "JSON".to_string(), rusqlite::types::Type::Text))?;

        self.conn.execute(
            "INSERT OR REPLACE INTO loans (id, borrower_id, lender_id, principal, interest_rate, disbursement_date, start_date, last_repayment_date, status, repayment_schedule)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                loan.id.to_string(),
                loan.borrower_id.to_string(),
                loan.lender_id.to_string(),
                loan.principal,
                loan.interest_rate,
                loan.disbursement_date.to_rfc3339(),
                loan.start_date.to_rfc3339(),
                loan.last_repayment_date.map(|dt| dt.to_rfc3339()),
                format!("{:?}", loan.status),
                repayment_schedule_json
            ],
        )?;
        Ok(())
    }

    pub fn load_loan(&self, id: Uuid) -> Result<Option<Loan>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, borrower_id, lender_id, principal, interest_rate, disbursement_date, start_date, last_repayment_date, status, repayment_schedule
             FROM loans WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let borrower_id_str: String = row.get(1)?;
            let lender_id_str: String = row.get(2)?;
            let principal: f64 = row.get(3)?;
            let interest_rate: f64 = row.get(4)?;
            let disbursement_date_str: String = row.get(5)?;
            let start_date_str: String = row.get(6)?;
            let last_repayment_date_str: Option<String> = row.get(7)?;
            let status_str: String = row.get(8)?;
            let repayment_schedule_json: String = row.get(9)?;

            let id = Uuid::parse_str(&id_str).map_err(|_| rusqlite::Error::InvalidColumnType(0, "UUID".to_string(), rusqlite::types::Type::Text))?;
            let borrower_id = Uuid::parse_str(&borrower_id_str).map_err(|_| rusqlite::Error::InvalidColumnType(1, "UUID".to_string(), rusqlite::types::Type::Text))?;
            let lender_id = Uuid::parse_str(&lender_id_str).map_err(|_| rusqlite::Error::InvalidColumnType(2, "UUID".to_string(), rusqlite::types::Type::Text))?;

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
            })
        })?;

        match rows.next() {
            Some(loan) => Ok(Some(loan?)),
            None => Ok(None),
        }
    }

    pub fn load_all_loans(&self) -> Result<Vec<Loan>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, borrower_id, lender_id, principal, interest_rate, disbursement_date, start_date, last_repayment_date, status, repayment_schedule
             FROM loans"
        )?;
        let loans = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let borrower_id_str: String = row.get(1)?;
            let lender_id_str: String = row.get(2)?;
            let principal: f64 = row.get(3)?;
            let interest_rate: f64 = row.get(4)?;
            let disbursement_date_str: String = row.get(5)?;
            let start_date_str: String = row.get(6)?;
            let last_repayment_date_str: Option<String> = row.get(7)?;
            let status_str: String = row.get(8)?;
            let repayment_schedule_json: String = row.get(9)?;

            let id = Uuid::parse_str(&id_str).map_err(|_| rusqlite::Error::InvalidColumnType(0, "UUID".to_string(), rusqlite::types::Type::Text))?;
            let borrower_id = Uuid::parse_str(&borrower_id_str).map_err(|_| rusqlite::Error::InvalidColumnType(1, "UUID".to_string(), rusqlite::types::Type::Text))?;
            let lender_id = Uuid::parse_str(&lender_id_str).map_err(|_| rusqlite::Error::InvalidColumnType(2, "UUID".to_string(), rusqlite::types::Type::Text))?;

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
            })
        })?;

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
}