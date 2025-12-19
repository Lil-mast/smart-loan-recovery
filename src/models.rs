use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Borrower,
    Lender,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub role: UserRole,
    // for future use add more attributes like emails, phone numbers etc.
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
    pub id: Uuid,
    pub borrower_id: Uuid,
    pub lender_id: Uuid,
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
    fn calculate_risk_score(&self) -> f64 {
        // Simple rule: higher if overdue
        if let LoanStatus::Overdue = self.status {
            0.8 // High risk
        } else {
            0.2 // Low risk
        }
    }
}