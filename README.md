# Smart Loan Recovery System

## Overview
An AI-enhanced loan recovery system in Rust that predicts potential defaults and recommends recovery strategies.

## Features
- **Database Persistence**: SQLite-backed storage with automatic schema management
- **Command-Line Interface**: Full CLI support for all operations using clap
- **User Management**: Support for borrowers and lenders with UUID-based identification and database persistence
- **Loan Tracking**: Comprehensive loan models with status tracking (Active, Overdue, Defaulted, Repaid)
- **Risk Scoring**: Built-in trait for calculating loan risk scores based on status and repayment history
- **Repayment Monitoring**: Tracks disbursement dates, repayment schedules, and last repayment dates
- **JSON Backup System**: Export/import functionality for data portability and backup
- **Rule-Based AI Recovery**: Intelligent recovery action recommendations based on risk assessment
- **Modular Architecture**: Well-structured codebase with clear separation of concerns

## Current Modules
- **`db.rs`**: SQLite database integration with JSON fallback support
- **`user.rs`**: Database-backed user management system
- **`models.rs`**: Core data models including User, Loan, UserRole, and LoanStatus enums, with RiskScorable trait
- **`loan.rs`**: Database-integrated loan tracking and management
- **`recovery.rs`**: Rule-based AI engine for predictive recovery actions
- **`main.rs`**: Application entry point with comprehensive demo

## Setup
1. Clone the repo: `git clone https://github.com/yourusername/smart-loan-recovery.git`
2. Build: `cargo build`
3. Run: `cargo run`

**Note**: The application will automatically create a `loans.db` SQLite database file and JSON backup files (`users_backup.json`, `loans_backup.json`) in the project directory.

## CLI Usage

The system supports both interactive demo mode and command-line interface:

### Demo Mode (Default)
```bash
cargo run
```
Runs the complete system demonstration with sample data.

### CLI Commands

#### Register a User
```bash
cargo run -- register-user --name "Alice Johnson" --role borrower
cargo run -- register-user --name "Bob Smith" --role lender
```

#### Create a Loan
```bash
cargo run -- create-loan --borrower-id <UUID> --lender-id <UUID> --principal 10000.0 --interest-rate 5.5 --months 12
```

#### Flag Overdue Loans
```bash
cargo run -- flag-overdues
```

#### Get Recovery Recommendation
```bash
cargo run -- recommend --loan-id <UUID>
```

#### Run Demo
```bash
cargo run -- demo
```

#### Help
```bash
cargo run -- --help
cargo run -- register-user --help
```
✅ **Database Integration Complete**: System now uses SQLite for persistent storage instead of in-memory data structures.

✅ **JSON Backup Support**: Implemented fallback JSON export/import functionality for data portability.

✅ **Functional Demo**: The system includes a working demo that showcases:
- User registration with database persistence
- Loan creation and management with SQLite storage
- Repayment processing and status updates
- Risk score calculation based on loan status
- Automatic JSON backup creation

## Data Persistence Architecture

### SQLite Database (`loans.db`)
- **Users Table**: Stores borrower and lender information with UUID identification
- **Loans Table**: Comprehensive loan data with JSON-encoded repayment schedules
- **Automatic Schema Creation**: Database tables are created automatically on first run

### JSON Backup System
- **Export Functionality**: `save_to_json()` method exports all data to human-readable JSON files
- **Fallback Support**: Alternative storage mechanism for data portability
- **Backup Files**: `users_backup.json` and `loans_backup.json` created automatically

## Database Integration Details

### UserManager Refactoring
```rust
// Before: In-memory storage
pub struct UserManager {
    users: Vec<User>,
}

// After: Database-backed
pub struct UserManager<'a> {
    db: &'a Db,
}
```

### LoanTracker Refactoring
```rust
// Before: In-memory storage
pub struct LoanTracker {
    loans: Vec<Loan>,
}

// After: Database-backed
pub struct LoanTracker<'a> {
    db: &'a Db,
}
```

### Key Database Operations
- **CRUD Operations**: Full Create, Read, Update, Delete support for users and loans
- **UUID Handling**: Proper serialization/deserialization of UUID fields
- **DateTime Management**: RFC3339 string conversion for timestamp fields
- **JSON Serialization**: Complex data structures stored as JSON blobs

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