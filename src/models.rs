use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Borrower,
    Lender,
    Admin,
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
pub struct LoanPayment {
    pub amount: f64,
    pub paid_at: DateTime<Utc>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanSignal {
    pub id: String,
    pub loan_id: uuid::Uuid,
    pub borrower_id: String,
    pub lender_id: String,
    /// `can_pay_early` | `concern`
    pub kind: String,
    pub message: String,
    #[serde(default)]
    pub proposed_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    /// `open` | `seen`
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    pub id: uuid::Uuid,
    pub borrower_id: String,
    pub lender_id: String,
    pub principal: f64,
    pub interest_rate: f64, // Annual interest rate in percentage
    pub disbursement_date: DateTime<Utc>,
    pub repayment_schedule: Vec<DateTime<Utc>>,
    pub start_date: DateTime<Utc>,
    pub last_repayment_date: Option<DateTime<Utc>>,
    pub status: LoanStatus,
    #[serde(default)]
    pub payments: Vec<LoanPayment>,
}

pub trait RiskScorable {
    fn calculate_risk_score(&self) -> f64;
}

#[derive(Debug, Clone)]
pub struct LoanSnapshot {
    pub risk_score: f64,
    pub health: &'static str,
    pub missed_installments: usize,
    pub next_due: Option<DateTime<Utc>>,
    pub days_until_due: i64,
    pub live_status: LoanStatus,
}

impl Loan {
    fn unpaid_dues(&self) -> Vec<DateTime<Utc>> {
        self.repayment_schedule
            .iter()
            .copied()
            .filter(|due| match self.last_repayment_date {
                Some(paid) => *due > paid,
                None => true,
            })
            .collect()
    }

    /// Deterministic book health in [0, 1]. Higher = worse.
    /// Combines missed share, lateness, coupon, and optional early-pay signal.
    pub fn snapshot_at(&self, now: DateTime<Utc>, early_pay: bool) -> LoanSnapshot {
        let unpaid = self.unpaid_dues();
        let missed = unpaid.iter().filter(|d| **d < now).count();
        let next_due = unpaid
            .iter()
            .filter(|d| **d >= now)
            .min()
            .copied()
            .or_else(|| unpaid.iter().max().copied());
        let days_until_due = next_due
            .map(|d| (d - now).num_days())
            .unwrap_or(0);

        let n = self.repayment_schedule.len().max(1) as f64;
        let missed_ratio = missed as f64 / n;
        let lateness = if missed > 0 {
            ((-days_until_due) as f64 / 90.0).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let coupon = (self.interest_rate / 40.0).clamp(0.0, 0.25);
        let early_relief = if early_pay { 0.18 } else { 0.0 };

        let mut risk = (0.10 + 0.50 * missed_ratio + 0.28 * lateness + coupon - early_relief)
            .clamp(0.02, 0.99);

        let live_status = if matches!(self.status, LoanStatus::Repaid) {
            risk = 0.04;
            LoanStatus::Repaid
        } else if missed >= 3 || (missed >= 1 && -days_until_due >= 90) {
            risk = risk.max(0.82);
            LoanStatus::Defaulted
        } else if missed >= 1 {
            risk = risk.max(0.45);
            LoanStatus::Overdue
        } else {
            LoanStatus::Active
        };

        let health = if risk < 0.35 {
            "healthy"
        } else if risk < 0.65 {
            "watch"
        } else {
            "critical"
        };

        LoanSnapshot {
            risk_score: risk,
            health,
            missed_installments: missed,
            next_due,
            days_until_due,
            live_status,
        }
    }
}

impl RiskScorable for Loan {
    fn calculate_risk_score(&self) -> f64 {
        crate::scoring::evaluate(self, Utc::now()).risk_score
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanNote {
    pub id: String,
    pub loan_id: String,
    pub author_id: String,
    pub kind: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}


