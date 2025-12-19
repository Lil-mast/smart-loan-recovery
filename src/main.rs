mod models;
mod user;
mod loan;

use crate::models::{UserRole, RiskScorable};
use crate::user::UserManager;
use crate::loan::LoanTracker;

fn main() {
    println!("🚀 Smart Loan Recovery System Starting...");

    // Initialize system components
    let mut user_manager = UserManager::new();
    let mut loan_tracker = LoanTracker::new();

    // Demo: Register users
    println!("\n📝 Registering users...");

    let borrower_id = user_manager.register_user(
        "Alice Johnson".to_string(),
        UserRole::Borrower
    ).expect("Failed to register borrower");

    let lender_id = user_manager.register_user(
        "Bob Smith".to_string(),
        UserRole::Lender
    ).expect("Failed to register lender");

    println!("✅ Registered borrower: {}", borrower_id);
    println!("✅ Registered lender: {}", lender_id);

    // Demo: Create a loan
    println!("\n💰 Creating a loan...");

    let loan_id = loan_tracker.create_loan(
        borrower_id,
        lender_id,
        10000.0,  // $10,000 principal
        5.5,      // 5.5% interest rate
        12        // 12 months duration
    ).expect("Failed to create loan");

    println!("✅ Created loan: {} for borrower {}", loan_id, borrower_id);

    // Demo script: Get loan details
    if let Some(loan) = loan_tracker.get_loan(loan_id) {
        println!("\n📊 Loan Details:");
        println!("   ID: {}", loan.id);
        println!("   Principal: ${:.2}", loan.principal);
        println!("   Interest Rate: {:.1}%", loan.interest_rate);
        println!("   Status: {:?}", loan.status);
        println!("   Risk Score: {:.2}", loan.calculate_risk_score());
    }

    // Demo: Update repayment
    println!("\n💳 Processing repayment...");
    if let Err(e) = loan_tracker.update_repayment(loan_id) {
        println!("❌ Failed to update repayment: {}", e);
    } else {
        println!("✅ Repayment updated successfully");
    }

    // Demo: Check updated loan status
    if let Some(loan) = loan_tracker.get_loan(loan_id) {
        println!("\n📈 Updated Loan Status: {:?}", loan.status);
        println!("   Risk Score: {:.2}", loan.calculate_risk_score());
    }

    println!("\n🎉 Smart Loan Recovery System Demo Complete!");
}
