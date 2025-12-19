# Smart Loan Recovery System

## Overview
An AI-enhanced loan recovery system in Rust that predicts potential defaults and recommends recovery strategies.

## Features
- **User Management**: Support for borrowers and lenders with UUID-based identification.
- **Loan Tracking**: Comprehensive loan models with status tracking (Active, Overdue, Defaulted, Repaid).
- **Risk Scoring**: Built-in trait for calculating loan risk scores based on status and repayment history.
- **Repayment Monitoring**: Tracks disbursement dates, repayment schedules, and last repayment dates.
- **Modular Architecture**: Well-structured codebase with clear separation of concerns.

## Current Modules
- **`user.rs`**: User management system with support for borrowers and lenders.
- **`models.rs`**: Core data models including User, Loan, UserRole, and LoanStatus enums, with RiskScorable trait.
- **`main.rs`**: Application entry point.

## Setup
1. Clone the repo: `git clone https://github.com/yourusername/smart-loan-recovery.git`
2. Build: `cargo build`
3. Run: `cargo run`

## Current Status
✅ **Functional Demo**: The system now includes a working demo that showcases:
- User registration for borrowers and lenders
- Loan creation with repayment scheduling
- Repayment processing and status updates
- Risk score calculation based on loan status

## Architecture
- **Modules**: `user`, `models`, and main entry point.
- **Key Traits**: `RiskScorable` for risk calculation.
- **Best Practices**: Error handling with `Result`, modular design, UUID-based entity identification.

## Data Structures
- **User**: Borrower/Lender with UUID, name, and role.
- **Loan**: Comprehensive loan tracking with principal, interest rate, disbursement date, repayment schedule, and status.
- **UserRole**: Enum for user types (Borrower, Lender).
- **LoanStatus**: Enum for loan states (Active, Overdue, Defaulted, Repaid).

## Future Enhancements
- Implement database persistence with SQLite.
- Integrate real ML for predictions.
- Add REST API with Actix-Web or Axum.
- CLI interface for user interactions.
- Recovery strategy recommendations engine.