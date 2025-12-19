use crate::models::{Loan, LoanStatus};
use crate::db::Db;
use chrono::{Duration, Utc};
use uuid::Uuid;
use rusqlite::Result;

pub struct LoanTracker<'a> {
    db: &'a Db,
}

impl<'a> LoanTracker<'a> {
    pub fn new(db: &'a Db) -> Self {
        LoanTracker { db }
    }

    pub fn create_loan(
        &self,
        borrower_id: Uuid,
        lender_id: Uuid,
        principal: f64,
        interest_rate: f64,
        duration_months: i64,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let mut schedule = Vec::new();
        for m in 1..=duration_months {
            schedule.push(now + Duration::days(30 * m)); // Approximate monthly
        }
        let loan = Loan {
            id,
            borrower_id,
            lender_id,
            principal,
            interest_rate,
            disbursement_date: now,
            repayment_schedule: schedule,
            start_date: now,
            last_repayment_date: None,
            status: LoanStatus::Active,
        };
        self.db.save_loan(&loan)?;
        Ok(id)
    }

    pub fn update_repayment(&self, loan_id: Uuid) -> Result<()> {
        let mut loan = self.db.load_loan(loan_id)?
            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;

        loan.last_repayment_date = Some(Utc::now());
        loan.status = if Utc::now() > *loan.repayment_schedule.last().unwrap() {
            LoanStatus::Repaid
        } else {
            LoanStatus::Active
        };

        self.db.save_loan(&loan)?;
        Ok(())
    }

    pub fn get_loan(&self, loan_id: Uuid) -> Result<Option<Loan>> {
        self.db.load_loan(loan_id)
    }

    pub fn get_all_loans(&self) -> Result<Vec<Loan>> {
        self.db.load_all_loans()
    }

    pub fn flag_overdues(&self) -> Result<()> {
        let loans = self.db.load_all_loans()?;
        let now = Utc::now();

        for mut loan in loans {
            if loan.status == LoanStatus::Active && now > loan.repayment_schedule[0] {
                loan.status = LoanStatus::Overdue;
                self.db.save_loan(&loan)?;
            }
        }
        Ok(())
    }
}