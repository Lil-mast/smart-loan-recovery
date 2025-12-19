use crate::models::{Loan, LoanStatus};
use chrono::{Duration, Utc};
use uuid::Uuid;

pub struct LoanTracker {
    loans: Vec<Loan>,
}

impl LoanTracker {
    pub fn new() -> Self {
        LoanTracker { loans: Vec::new() }
    }

    pub fn create_loan(
        &mut self,
        borrower_id: Uuid,
        lender_id: Uuid,
        principal: f64,
        interest_rate: f64,
        duration_months: i64,
    ) -> Result<Uuid, String> {
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
        self.loans.push(loan);
        Ok(id)
    }

    pub fn update_repayment(&mut self, loan_id: Uuid) -> Result<(), String> {
        if let Some(loan) = self.loans.iter_mut().find(|l| l.id == loan_id) {
            loan.last_repayment_date = Some(Utc::now());
            loan.status = if Utc::now() > *loan.repayment_schedule.last().unwrap() {
                LoanStatus::Repaid
            } else {
                LoanStatus::Active
            };
            Ok(())
        } else {
            Err("Loan not found".to_string())
        }
    }

    pub fn get_loan(&self, loan_id: Uuid) -> Option<&Loan> {
        self.loans.iter().find(|l| l.id == loan_id)
    }

    pub fn flag_overdues(&mut self) {
        let now = Utc::now();
        for loan in &mut self.loans {
            if loan.status == LoanStatus::Active && now > loan.repayment_schedule[0] { // Check next due
                loan.status = LoanStatus::Overdue;
            }
        }
    }
}