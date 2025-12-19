use crate::models::{Loan, RiskScorable};

#[derive(Debug)]
pub enum RecoveryAction {
    SendReminder,
    RenegotiateTerms,
    EscalateToCollection,
}

pub struct RecoveryEngine;

impl RecoveryEngine {
    pub fn predict_default(&self, loan: &Loan) -> f64 {
        loan.calculate_risk_score() // From trait
    }

    pub fn recommend_action(&self, risk_score: f64, repayment_history: usize) -> RecoveryAction { // History: e.g., missed payments
        match (risk_score, repayment_history) {
            (score, hist) if score > 0.7 || hist > 2 => RecoveryAction::EscalateToCollection,
            (score, hist) if score > 0.4 || hist > 0 => RecoveryAction::RenegotiateTerms,
            _ => RecoveryAction::SendReminder,
        }
    }
}