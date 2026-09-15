use crate::models::{Loan, LoanPayment, LoanStatus};
use crate::db::Db;
use crate::scoring;
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
        borrower_id: String,
        lender_id: String,
        principal: f64,
        interest_rate: f64,
        duration_months: i64,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let mut schedule = Vec::new();
        for m in 1..=duration_months {
            schedule.push(now + Duration::days(30 * m));
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
            payments: Vec::new(),
        };
        self.db.save_loan(&loan)?;
        Ok(id)
    }

    pub fn record_payment(&self, loan_id: Uuid, amount: f64, note: Option<String>) -> Result<Loan> {
        if amount <= 0.0 {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let mut loan = self.db.load_loan(loan_id)?
            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;
        let now = Utc::now();
        loan.payments.push(LoanPayment {
            amount,
            paid_at: now,
            note,
        });
        loan.last_repayment_date = Some(now);
        let health = scoring::evaluate(&loan, now);
        loan.status = scoring::live_status(&health);
        self.db.save_loan(&loan)?;
        Ok(loan)
    }

    pub fn refresh_status(&self, loan: &mut Loan) -> Result<bool> {
        let health = scoring::evaluate(loan, Utc::now());
        let next = scoring::live_status(&health);
        if next != loan.status {
            loan.status = next;
            self.db.save_loan(loan)?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn get_loan(&self, loan_id: Uuid) -> Result<Option<Loan>> {
        self.db.load_loan(loan_id)
    }

    pub fn get_all_loans(&self) -> Result<Vec<Loan>> {
        self.db.load_all_loans()
    }

    pub fn flag_overdues(&self) -> Result<usize> {
        let loans = self.db.load_all_loans()?;
        let now = Utc::now();
        let mut flagged_count = 0;

        for mut loan in loans {
            if matches!(loan.status, LoanStatus::Repaid) {
                continue;
            }
            let health = scoring::evaluate(&loan, now);
            let next = scoring::live_status(&health);
            if next != loan.status {
                loan.status = next;
                self.db.save_loan(&loan)?;
                flagged_count += 1;
            }
        }
        Ok(flagged_count)
    }
}

