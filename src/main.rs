mod models;
mod user;
mod loan;
mod db;
mod recovery;
mod api;
mod config;
mod error;
mod auth;

use crate::config::Config;
use crate::models::{UserRole, RiskScorable};
use crate::user::UserManager;
use crate::loan::LoanTracker;
use crate::recovery::RecoveryEngine;
use crate::db::Db;
use clap::{Parser, Subcommand};
use uuid::Uuid;
use actix_web;

#[derive(Parser)]
#[command(name = "smart-loan-recovery")]
#[command(about = "AI-enhanced loan recovery system")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Register a new user (borrower or lender)
    RegisterUser {
        /// User name
        #[arg(short, long)]
        name: String,
        /// User role (borrower or lender)
        #[arg(short, long)]
        role: String
    },
    /// Create a new loan
    CreateLoan {
        /// Borrower UUID
        #[arg(short, long)]
        borrower_id: String,
        /// Lender UUID
        #[arg(short, long)]
        lender_id: String,
        /// Loan principal amount
        #[arg(short, long)]
        principal: f64,
        /// Annual interest rate (percentage)
        #[arg(short = 'r', long)]
        interest_rate: f64,
        /// Loan duration in months
        #[arg(short, long)]
        months: i64
    },
    /// Flag overdue loans
    FlagOverdues,
    /// Get recovery recommendation for a loan
    Recommend {
        /// Loan UUID
        #[arg(short, long)]
        loan_id: String
    },
    /// Run the demo
    Demo,
}

fn run_cli(cli: Cli, db: Db) -> Result<(), Box<dyn std::error::Error>> {
    let user_manager = UserManager::new(&db);
    let loan_tracker = LoanTracker::new(&db);
    let recovery_engine = RecoveryEngine;

    match cli.command.unwrap() {
        Commands::RegisterUser { name, role } => {
            let user_role = match role.to_lowercase().as_str() {
                "borrower" => UserRole::Borrower,
                "lender" => UserRole::Lender,
                _ => {
                    eprintln!("❌ Invalid role. Use 'borrower' or 'lender'");
                    return Ok(());
                }
            };

            match user_manager.register_user(name.clone(), None, user_role, None, None) {
                Ok(user_id) => println!("✅ Registered {} as {} with ID: {}", name, role, user_id),
                Err(e) => eprintln!("❌ Failed to register user: {}", e),
            }
        }

        Commands::CreateLoan { borrower_id, lender_id, principal, interest_rate, months } => {
            let borrower_uuid = Uuid::parse_str(&borrower_id)
                .map_err(|_| "Invalid borrower UUID format")?;
            let lender_uuid = Uuid::parse_str(&lender_id)
                .map_err(|_| "Invalid lender UUID format")?;

            match loan_tracker.create_loan(borrower_uuid.to_string(), lender_uuid.to_string(), principal, interest_rate, months) {
                Ok(loan_id) => println!("✅ Created loan with ID: {}", loan_id),
                Err(e) => eprintln!("❌ Failed to create loan: {}", e),
            }
        }

        Commands::FlagOverdues => {
            match loan_tracker.flag_overdues() {
                Ok(count) => println!("✅ Overdue loans flagged successfully: {} loans flagged", count),
                Err(e) => eprintln!("❌ Failed to flag overdues: {}", e),
            }
        }

        Commands::Recommend { loan_id } => {
            let loan_uuid = Uuid::parse_str(&loan_id)
                .map_err(|_| "Invalid loan UUID format")?;

            match loan_tracker.get_loan(loan_uuid) {
                Ok(Some(loan)) => {
                    let risk_score = recovery_engine.predict_default(&loan);
                    let action = recovery_engine.recommend_action(risk_score, 0); // Simplified: assume 0 missed payments for demo
                    println!("📊 Loan {} - Risk Score: {:.2}", loan_id, risk_score);
                    println!("💡 Recommended Action: {:?}", action);
                }
                Ok(None) => eprintln!("❌ Loan not found"),
                Err(e) => eprintln!("❌ Failed to load loan: {}", e),
            }
        }

        Commands::Demo => {
            run_demo(db);
        }
    }

    Ok(())
}

#[actix_web::main]  // Use actix runtime
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init();

    // Load main application configuration
    let config = Config::from_env().expect("Failed to load configuration");

    // Load Firebase authentication configuration (optional, for server mode)
    if dotenv::from_filename(".env.local").is_ok() {
        log::info!("🔐 Loaded Firebase configuration from .env.local");
    } else {
        log::info!("ℹ️  No .env.local file found. Firebase auth may not be available.");
    }

    let cli = Cli::parse();

    // Check if running in CLI mode or server mode
    if let Some(_) = cli.command {
        // CLI mode
        let db = match Db::new_with_path(&config.database_url) {
            Ok(db) => db,
            Err(e) => {
                eprintln!("❌ Failed to initialize database: {}", e);
                return Ok(());
            }
        };
        if let Err(e) = run_cli(cli, db) {
            eprintln!("❌ CLI Error: {}", e);
        }
        Ok(())
    } else {
        // Server mode (no subcommand provided)
        crate::api::run_server(config).await
    }
}

fn run_demo(db: Db) {
    println!("🚀 Smart Loan Recovery System Starting...");

    // Initialize system components with database
    let user_manager = UserManager::new(&db);
    let loan_tracker = LoanTracker::new(&db);

    // Demo: Register users
    println!(" 📝 Registering users...");

    let borrower_id = match user_manager.register_user(
        "Alice Johnson".to_string(),
        None,
        UserRole::Borrower,
        None,
        None
    ) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("❌ Failed to register borrower: {}", e);
            return;
        }
    };

    let lender_id = match user_manager.register_user(
        "Bob Smith".to_string(),
        None,
        UserRole::Lender,
        None,
        Some("Demo Bank".to_string())
    ) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("❌ Failed to register lender: {}", e);
            return;
        }
    };

    println!("✅ Registered borrower: {}", borrower_id);
    println!("✅ Registered lender: {}", lender_id);

    // Demo: Create a loan
    println!(" 💰 Creating a loan...");

    let loan_id = match loan_tracker.create_loan(
        borrower_id.clone(),
        lender_id,
        10000.0,  // $10,000 principal
        5.5,      // 5.5% interest rate
        12        // 12 months duration
    ) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("❌ Failed to create loan: {}", e);
            return;
        }
    };

    println!("✅ Created loan: {} for borrower {}", loan_id, borrower_id);

    // Demo: Get loan details
    match loan_tracker.get_loan(loan_id) {
        Ok(Some(loan)) => {
            println!(" 📊 Loan Details:");
            println!("   ID: {}", loan.id);
            println!("   Principal: ${:.2}", loan.principal);
            println!("   Interest Rate: {:.1}%", loan.interest_rate);
            println!("   Status: {:?}", loan.status);
            println!("   Risk Score: {:.2}", loan.calculate_risk_score());
        }
        Ok(None) => println!("❌ Loan not found"),
        Err(e) => eprintln!("❌ Failed to load loan: {}", e),
    }

    // Demo: Update repayment
    println!(" 💳 Processing repayment...");
    if let Err(e) = loan_tracker.update_repayment(loan_id) {
        eprintln!("❌ Failed to update repayment: {}", e);
    } else {
        println!("✅ Repayment updated successfully");
    }

    // Demo: Check updated loan status
    match loan_tracker.get_loan(loan_id) {
        Ok(Some(loan)) => {
            println!(" 📈 Updated Loan Status: {:?}", loan.status);
            println!("   Risk Score: {:.2}", loan.calculate_risk_score());
        }
        Ok(None) => println!("❌ Loan not found"),
        Err(e) => eprintln!("❌ Failed to load loan: {}", e),
    }

    // Demo: Save to JSON backup
    println!(" 💾 Creating JSON backup...");
    if let Err(e) = db.save_to_json("users_backup.json", "loans_backup.json") {
        eprintln!("❌ Failed to create JSON backup: {}", e);
    } else {
        println!("✅ JSON backup created successfully");
    }

    println!(" 🎉 Smart Loan Recovery System Demo Complete!");
    println!("💡 Data is now persisted in SQLite database 'loans.db'");
}

