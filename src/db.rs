use rusqlite::{Connection, Result, params};
use crate::models::{User, UserRole, Loan, LoanStatus};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::fs;
use std::path::Path;

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("loans.db")?;
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
        Ok(())
    }

    // User operations
    pub fn save_user(&self, user: &User) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO users (id, name, role) VALUES (?1, ?2, ?3)",
            params![user.id.to_string(), user.name, format!("{:?}", user.role)],
        )?;
        Ok(())
    }

    pub fn load_user(&self, id: Uuid) -> Result<Option<User>> {
        let mut stmt = self.conn.prepare("SELECT id, name, role FROM users WHERE id = ?1")?;
        let mut rows = stmt.query_map(params![id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let role_str: String = row.get(2)?;

            let id = Uuid::parse_str(&id_str).map_err(|_| rusqlite::Error::InvalidColumnType(0, "UUID".to_string(), rusqlite::types::Type::Text))?;
            let role = match role_str.as_str() {
                "Borrower" => UserRole::Borrower,
                "Lender" => UserRole::Lender,
                _ => return Err(rusqlite::Error::InvalidColumnType(2, "UserRole".to_string(), rusqlite::types::Type::Text)),
            };

            Ok(User { id, name, role })
        })?;

        match rows.next() {
            Some(user) => Ok(Some(user?)),
            None => Ok(None),
        }
    }

    pub fn load_all_users(&self) -> Result<Vec<User>> {
        let mut stmt = self.conn.prepare("SELECT id, name, role FROM users")?;
        let users = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let role_str: String = row.get(2)?;

            let id = Uuid::parse_str(&id_str).map_err(|_| rusqlite::Error::InvalidColumnType(0, "UUID".to_string(), rusqlite::types::Type::Text))?;
            let role = match role_str.as_str() {
                "Borrower" => UserRole::Borrower,
                "Lender" => UserRole::Lender,
                _ => return Err(rusqlite::Error::InvalidColumnType(2, "UserRole".to_string(), rusqlite::types::Type::Text)),
            };

            Ok(User { id, name, role })
        })?;

        users.collect()
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

    pub fn load_from_json<P: AsRef<Path>>(users_path: P, loans_path: P) -> Result<(Vec<User>, Vec<Loan>)> {
        let users_json = fs::read_to_string(users_path)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
        let users: Vec<User> = serde_json::from_str(&users_json)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;

        let loans_json = fs::read_to_string(loans_path)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
        let loans: Vec<Loan> = serde_json::from_str(&loans_json)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;

        Ok((users, loans))
    }
}