use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Borrower,
    Lender,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String, // 4-char alphanumeric A-Z0-9a-z
    pub name: String,
    pub role: UserRole,
    pub email: Option<String>,
    /// For borrowers: the lender they chose
    pub lender_id: Option<String>,
    /// For lenders: the organization they belong to
    pub organization: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoanStatus {
    Active,
    Overdue,
    Defaulted,
    Repaid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    pub id: uuid::Uuid,
    pub borrower_id: uuid::Uuid,
    pub lender_id: uuid::Uuid,
    pub principal: f64,
    pub interest_rate: f64, // Annual interest rate in percentage
    pub disbursement_date: DateTime<Utc>,
    pub repayment_schedule: Vec<DateTime<Utc>>,
    pub start_date: DateTime<Utc>,
    pub last_repayment_date: Option<DateTime<Utc>>,
    pub status: LoanStatus,
}

pub trait RiskScorable {
    fn calculate_risk_score(&self) -> f64;
}

impl RiskScorable for Loan {
    /// Score in [0, 1]: higher means higher predicted default / recovery difficulty.
    fn calculate_risk_score(&self) -> f64 {
        let status_base: f64 = match self.status {
            LoanStatus::Defaulted => 0.92,
            LoanStatus::Overdue => 0.78,
            LoanStatus::Repaid => 0.06,
            LoanStatus::Active => 0.22,
        };
        // Slightly lift risk for high coupon active loans (demo heuristic)
        let rate_bump: f64 = if matches!(self.status, LoanStatus::Active) && self.interest_rate > 15.0 {
            0.12
        } else {
            0.0
        };
        f64::min(status_base + rate_bump, 0.99)
    }
}

