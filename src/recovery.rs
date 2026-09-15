use crate::models::{Loan, RiskScorable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

    pub fn recommend_action(&self, risk_score: f64, missed_installments: usize) -> RecoveryAction {
        match (risk_score, missed_installments) {
            (score, missed) if score >= 0.65 || missed >= 3 => RecoveryAction::EscalateToCollection,
            (score, missed) if score >= 0.35 || missed >= 1 => RecoveryAction::RenegotiateTerms,
            _ => RecoveryAction::SendReminder,
        }
    }
}