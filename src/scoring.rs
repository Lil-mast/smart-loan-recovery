//! Live loan health scoring.
//!
//! Higher `score` = healthier book. The model is deterministic given (loan, now):
//!   1. Amortize the contractual monthly payment.
//!   2. Apply recorded payments FIFO against the installment schedule.
//!   3. Penalize delinquency (days past due), coverage shortfall, and consecutive misses.
//!   4. Band the result so a lender can scan the book in one glance.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::models::{Loan, LoanPayment, LoanStatus};
use crate::recovery::RecoveryAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthBand {
    Healthy,
    Watch,
    AtRisk,
    Critical,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallmentView {
    pub due: DateTime<Utc>,
    pub expected: f64,
    pub covered: f64,
    pub status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoanHealth {
    pub score: f64,
    pub band: HealthBand,
    pub risk_score: f64,
    pub days_past_due: i64,
    pub missed_installments: usize,
    pub due_installments: usize,
    pub paid_amount: f64,
    pub expected_paid: f64,
    pub outstanding_amount: f64,
    pub next_due: Option<DateTime<Utc>>,
    pub monthly_payment: f64,
    pub paid_in_full: bool,
    pub coverage_ratio: f64,
    pub consecutive_missed: usize,
    pub recommendation: RecoveryAction,
    pub installments: Vec<InstallmentView>,
}

pub fn monthly_payment(principal: f64, annual_rate_pct: f64, months: usize) -> f64 {
    if months == 0 || principal <= 0.0 {
        return 0.0;
    }
    let i = annual_rate_pct / 1200.0;
    if i.abs() < 1e-12 {
        return principal / months as f64;
    }
    let pow = (1.0 + i).powi(months as i32);
    principal * i * pow / (pow - 1.0)
}

fn clamp(x: f64, lo: f64, hi: f64) -> f64 {
    x.max(lo).min(hi)
}

/// Score a loan as of `now`. Pure function — no I/O.
pub fn evaluate(loan: &Loan, now: DateTime<Utc>) -> LoanHealth {
    let n = loan.repayment_schedule.len().max(1);
    let monthly = monthly_payment(loan.principal, loan.interest_rate, n);
    let total_contract = monthly * n as f64;
    let paid_amount: f64 = loan.payments.iter().map(|p| p.amount.max(0.0)).sum();
    let paid_in_full = paid_amount + 0.01 >= total_contract && total_contract > 0.0
        || matches!(loan.status, LoanStatus::Repaid);

    let mut remaining_cover = paid_amount;
    let mut installments = Vec::with_capacity(n);
    let mut missed = 0usize;
    let mut due_count = 0usize;
    let mut consecutive_missed = 0usize;
    let mut streak = 0usize;
    let mut first_unpaid_due: Option<DateTime<Utc>> = None;
    let mut next_due: Option<DateTime<Utc>> = None;

    for due in &loan.repayment_schedule {
        let covered = remaining_cover.min(monthly);
        remaining_cover -= covered;
        let is_due = now >= *due;
        if is_due {
            due_count += 1;
        }
        let fully_covered = covered + 0.01 >= monthly;
        let status = if fully_covered {
            streak = 0;
            "paid"
        } else if is_due {
            missed += 1;
            streak += 1;
            consecutive_missed = consecutive_missed.max(streak);
            if first_unpaid_due.is_none() {
                first_unpaid_due = Some(*due);
            }
            "overdue"
        } else {
            streak = 0;
            if next_due.is_none() {
                next_due = Some(*due);
            }
            "upcoming"
        };
        if status == "paid" && !is_due {
            // paid ahead of schedule
        }
        if next_due.is_none() && !fully_covered && !is_due {
            next_due = Some(*due);
        }
        installments.push(InstallmentView {
            due: *due,
            expected: monthly,
            covered,
            status,
        });
    }

    if next_due.is_none() {
        next_due = loan
            .repayment_schedule
            .iter()
            .find(|d| now < **d)
            .copied();
    }

    let days_past_due = match first_unpaid_due {
        Some(due) if now > due => (now - due).num_days().max(0),
        _ => 0,
    };

    let expected_paid = monthly * due_count as f64;
    let coverage_ratio = if expected_paid > 0.01 {
        paid_amount / expected_paid
    } else if paid_in_full {
        1.0
    } else {
        // Nothing due yet: on-track unless they already defaulted.
        1.0
    };

    let outstanding = if paid_in_full {
        0.0
    } else {
        (total_contract - paid_amount).max(0.0)
    };

    // Weighted score in [0, 100].
    // Coverage (40): shortfall vs what should have been paid by now.
    // Timeliness (30): exponential decay with days past due (half-life ~31d).
    // Consistency (20): consecutive missed installments.
    // Vintage (10): remaining term vs remaining balance (are they falling behind the clock).
    let coverage_term = 40.0 * clamp(coverage_ratio, 0.0, 1.0);
    let timeliness = 30.0 * (-(days_past_due as f64) / 45.0).exp();
    let consistency = 20.0 * clamp(1.0 - consecutive_missed as f64 / 3.0, 0.0, 1.0);

    let elapsed = {
        let start = loan.start_date;
        let end = loan
            .repayment_schedule
            .last()
            .copied()
            .unwrap_or(start);
        let span = (end - start).num_seconds().max(1) as f64;
        clamp((now - start).num_seconds() as f64 / span, 0.0, 1.0)
    };
    let paid_frac = if total_contract > 0.01 {
        clamp(paid_amount / total_contract, 0.0, 1.0)
    } else {
        1.0
    };
    let lag = (elapsed - paid_frac).max(0.0);
    let vintage = 10.0 * (1.0 - lag);

    let mut score = coverage_term + timeliness + consistency + vintage;
    if paid_in_full {
        score = 100.0;
    }
    match loan.status {
        LoanStatus::Defaulted if !paid_in_full => score = score.min(25.0),
        LoanStatus::Overdue if days_past_due == 0 => {}
        _ => {}
    }
    score = clamp(score, 0.0, 100.0);

    let band = band_for(score);
    let recommendation = recommendation_for(band);

    LoanHealth {
        score,
        band,
        risk_score: clamp(1.0 - score / 100.0, 0.0, 1.0),
        days_past_due,
        missed_installments: missed,
        due_installments: due_count,
        paid_amount,
        expected_paid,
        outstanding_amount: outstanding,
        next_due,
        monthly_payment: monthly,
        paid_in_full,
        coverage_ratio,
        consecutive_missed,
        recommendation,
        installments,
    }
}

fn band_for(score: f64) -> HealthBand {
    if score >= 80.0 {
        HealthBand::Healthy
    } else if score >= 65.0 {
        HealthBand::Watch
    } else if score >= 45.0 {
        HealthBand::AtRisk
    } else {
        HealthBand::Critical
    }
}

fn recommendation_for(band: HealthBand) -> RecoveryAction {
    match band {
        HealthBand::Critical => RecoveryAction::EscalateToCollection,
        HealthBand::AtRisk => RecoveryAction::RenegotiateTerms,
        HealthBand::Watch => RecoveryAction::SendReminder,
        HealthBand::Healthy => RecoveryAction::SendReminder,
    }
}

/// Borrower signaled they can pay early — lift health slightly so the book isn't treated as silent risk.
pub fn apply_early_intent(mut health: LoanHealth) -> LoanHealth {
    if health.paid_in_full {
        return health;
    }
    health.score = clamp(health.score + 8.0, 0.0, 100.0);
    health.risk_score = clamp(1.0 - health.score / 100.0, 0.0, 1.0);
    health.band = band_for(health.score);
    health.recommendation = recommendation_for(health.band);
    health
}

pub fn live_status(health: &LoanHealth) -> LoanStatus {
    if health.paid_in_full {
        LoanStatus::Repaid
    } else if health.days_past_due >= 90 || health.consecutive_missed >= 3 {
        LoanStatus::Defaulted
    } else if health.days_past_due > 0 {
        LoanStatus::Overdue
    } else {
        LoanStatus::Active
    }
}

pub fn total_paid(payments: &[LoanPayment]) -> f64 {
    payments.iter().map(|p| p.amount.max(0.0)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Loan;
    use chrono::Duration;
    use uuid::Uuid;

    fn loan_with(months: i64, payments: Vec<LoanPayment>, start_offset_days: i64) -> Loan {
        let start = Utc::now() - Duration::days(start_offset_days);
        let mut schedule = Vec::new();
        for m in 1..=months {
            schedule.push(start + Duration::days(30 * m));
        }
        Loan {
            id: Uuid::new_v4(),
            borrower_id: "B001".into(),
            lender_id: "L001".into(),
            principal: 12_000.0,
            interest_rate: 12.0,
            disbursement_date: start,
            repayment_schedule: schedule,
            start_date: start,
            last_repayment_date: None,
            status: LoanStatus::Active,
            payments,
        }
    }

    #[test]
    fn new_loan_nothing_due_is_healthy() {
        let loan = loan_with(12, vec![], 0);
        let h = evaluate(&loan, Utc::now());
        assert!(h.score >= 80.0, "score={}", h.score);
        assert_eq!(h.band, HealthBand::Healthy);
        assert_eq!(h.days_past_due, 0);
    }

    #[test]
    fn two_missed_installments_is_stressed() {
        let loan = loan_with(12, vec![], 75);
        let h = evaluate(&loan, Utc::now());
        assert!(h.missed_installments >= 2, "missed={}", h.missed_installments);
        assert!(h.score < 80.0, "score={}", h.score);
        assert!(h.days_past_due > 0);
    }

    #[test]
    fn fully_paid_is_healthy() {
        let mut loan = loan_with(6, vec![], 200);
        let monthly = monthly_payment(loan.principal, loan.interest_rate, 6);
        loan.payments = vec![LoanPayment {
            amount: monthly * 6.0,
            paid_at: Utc::now(),
            note: None,
        }];
        let h = evaluate(&loan, Utc::now());
        assert!(h.paid_in_full);
        assert_eq!(h.band, HealthBand::Healthy);
        assert_eq!(h.outstanding_amount, 0.0);
    }

    #[test]
    fn monthly_payment_zero_rate_is_principal_over_term() {
        let m = monthly_payment(1200.0, 0.0, 12);
        assert!((m - 100.0).abs() < 1e-9);
    }

    #[test]
    fn early_pay_intent_raises_health_score() {
        let loan = loan_with(12, vec![], 40);
        let base = evaluate(&loan, Utc::now());
        let eased = apply_early_intent(base.clone());
        assert!(eased.score > base.score);
        assert!(eased.risk_score < base.risk_score);
    }
}
