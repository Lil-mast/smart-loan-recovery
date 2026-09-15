use actix_cors::Cors;
use actix_web::{web, App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Result as ActixResult, middleware::Logger};
use actix_identity::{Identity, IdentityMiddleware};
use actix_web::cookie::{Key, SameSite};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use crate::db::Db;
use crate::user::UserManager;
use crate::loan::LoanTracker;
use crate::models::{Loan, LoanSignal, User, UserRole};
use crate::scoring;
use crate::recovery::RecoveryAction;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::auth::{config_auth_routes, init_auth_services, AuthState, middleware::auth::JwtAuth, services::TokenBlacklist};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

fn is_valid_4char_id(id: &str) -> bool {
    id.len() == 4 && id.chars().all(|c| c.is_alphanumeric())
}

fn require_lender_id(mgr: &UserManager<'_>, raw: &str) -> AppResult<String> {
    if !is_valid_4char_id(raw) {
        return Err(AppError::InvalidInput(
            "Enter the lender’s 4-character account ID".to_string(),
        ));
    }
    let user = mgr
        .get_user(raw)
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::InvalidInput("No lender found with that ID".to_string()))?;
    if user.role != UserRole::Lender {
        return Err(AppError::InvalidInput(
            "That ID is not a lender account".to_string(),
        ));
    }
    Ok(user.id)
}

fn cors_origin_allowed(origin: &str) -> bool {
    if origin.is_empty()
        || origin == "null"
        || origin == "http://127.0.0.1:3000"
        || origin == "http://localhost:3000"
        || origin == "http://127.0.0.1:3001"
        || origin == "http://localhost:3001"
        || origin == "http://127.0.0.1:5500"
        || origin.ends_with(".vercel.app")
        || origin.ends_with(".onrender.com")
        || origin == "https://lendwise-recovery.fly.dev"
    {
        return true;
    }
    if let Ok(fe) = std::env::var("FRONTEND_URL") {
        let fe = fe.trim().trim_end_matches('/');
        if !fe.is_empty() && origin == fe {
            return true;
        }
    }
    if let Ok(extra) = std::env::var("CORS_ORIGINS") {
        return extra.split(',').any(|o| o.trim() == origin);
    }
    false
}

#[derive(Deserialize)]
pub struct RegisterUserReq {
    name: String,
    role: String, // "borrower" or "lender"
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    lender_name: Option<String>, // for borrowers
    #[serde(default)]
    lender_id: Option<String>, // for borrowers — lender account ID
    #[serde(default)]
    organization: Option<String>, // for lenders
}

#[derive(Deserialize)]
pub struct UsersQuery {
    #[serde(default)]
    email: Option<String>,
    /// Optional filter: `borrower` or `lender` (case-insensitive).
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    lender_id: Option<String>,
}

#[derive(Deserialize)]
pub struct LoansQuery {
    #[serde(default)]
    borrower_id: Option<String>,
    #[serde(default)]
    lender_id: Option<String>,
}

#[derive(Serialize)]
struct InstallmentApiJson {
    due: String,
    expected: f64,
    covered: f64,
    status: String,
}

#[derive(Serialize)]
struct LoanApiJson {
    id: uuid::Uuid,
    borrower_id: String,
    lender_id: String,
    principal: f64,
    amount: f64,
    interest_rate: f64,
    status: String,
    recovery_status: f64,
    outstanding_amount: f64,
    paid_amount: f64,
    expected_paid: f64,
    monthly_payment: f64,
    risk_score: f64,
    health_score: f64,
    health_band: String,
    days_past_due: i64,
    missed_installments: usize,
    next_due: Option<String>,
    coverage_ratio: f64,
    ai_recommendation: String,
    repayment_schedule: Vec<String>,
    installments: Vec<InstallmentApiJson>,
    evaluated_at: String,
    days_until_due: i64,
    early_pay_offered: bool,
}

fn action_key(action: &RecoveryAction) -> &'static str {
    match action {
        RecoveryAction::SendReminder => "send_reminder",
        RecoveryAction::RenegotiateTerms => "renegotiate_terms",
        RecoveryAction::EscalateToCollection => "escalate_to_collection",
    }
}

fn loan_api_json(db: &Db, loan: &Loan) -> LoanApiJson {
    let now = chrono::Utc::now();
    let mut health = scoring::evaluate(loan, now);
    let early_pay_offered = db
        .loan_has_open_kind(&loan.id.to_string(), "can_pay_early")
        .unwrap_or(false);
    if early_pay_offered {
        health = scoring::apply_early_intent(health);
    }
    let live = scoring::live_status(&health);
    let recovery_status = if health.paid_in_full {
        100.0
    } else if health.monthly_payment * loan.repayment_schedule.len().max(1) as f64 > 0.01 {
        let total = health.monthly_payment * loan.repayment_schedule.len() as f64;
        (health.paid_amount / total * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    let days_until_due = health.next_due.map(|d| (d - now).num_days()).unwrap_or(0);
    LoanApiJson {
        id: loan.id,
        borrower_id: loan.borrower_id.clone(),
        lender_id: loan.lender_id.clone(),
        principal: loan.principal,
        amount: loan.principal,
        interest_rate: loan.interest_rate,
        status: format!("{:?}", live).to_lowercase(),
        recovery_status,
        outstanding_amount: health.outstanding_amount,
        paid_amount: health.paid_amount,
        expected_paid: health.expected_paid,
        monthly_payment: health.monthly_payment,
        risk_score: health.risk_score,
        health_score: health.score,
        health_band: match health.band {
            scoring::HealthBand::Healthy => "healthy",
            scoring::HealthBand::Watch => "watch",
            scoring::HealthBand::AtRisk => "at_risk",
            scoring::HealthBand::Critical => "critical",
        }
        .to_string(),
        days_past_due: health.days_past_due,
        missed_installments: health.missed_installments,
        next_due: health.next_due.map(|d| d.to_rfc3339()),
        coverage_ratio: health.coverage_ratio,
        ai_recommendation: action_key(&health.recommendation).to_string(),
        repayment_schedule: loan
            .repayment_schedule
            .iter()
            .map(|d| d.to_rfc3339())
            .collect(),
        installments: health
            .installments
            .iter()
            .map(|i| InstallmentApiJson {
                due: i.due.to_rfc3339(),
                expected: i.expected,
                covered: i.covered,
                status: i.status.to_string(),
            })
            .collect(),
        evaluated_at: now.to_rfc3339(),
        days_until_due,
        early_pay_offered,
    }
}

fn current_user(identity: &Identity, db: &Db) -> AppResult<User> {
    let user_id = identity.id().map_err(|_| AppError::AuthRequired)?;
    UserManager::new(db)
        .get_user(&user_id)
        .map_err(AppError::Database)?
        .ok_or(AppError::AuthRequired)
}

#[derive(Deserialize)]
struct CreateLoanReq {
    borrower_id: String,
    lender_id: String,
    principal: f64,
    interest_rate: f64,
    months: i64,
}

#[derive(Serialize)]
struct CreateLoanRes {
    id: uuid::Uuid,
}

pub async fn register_user(
    data: web::Json<RegisterUserReq>,
    db: web::Data<Db>,
    http: HttpRequest,
) -> AppResult<ActixResult<HttpResponse>> {
    let mgr = UserManager::new(&db);
    let role = match data.role.as_str() {
        "borrower" => UserRole::Borrower,
        "lender" => UserRole::Lender,
        _ => return Err(AppError::InvalidInput("Role must be 'borrower' or 'lender'".to_string())),
    };

    let email = if role == UserRole::Lender {
        None
    } else {
        data.email.clone().filter(|e| !e.trim().is_empty())
    };

    let lender_id = if role == UserRole::Borrower {
        let raw = data
            .lender_id
            .as_deref()
            .or(data.lender_name.as_deref())
            .unwrap_or("")
            .trim();
        Some(require_lender_id(&mgr, raw)?)
    } else {
        None
    };
    let organization = data.organization.clone().filter(|o| !o.trim().is_empty());

    if role == UserRole::Lender && organization.is_none() {
        return Err(AppError::InvalidInput("Lenders must specify an organization".to_string()));
    }

    let user_id = mgr
        .register_user(data.name.clone(), email, role.clone(), lender_id, organization)
        .map_err(AppError::Database)?;

    let user = mgr
        .get_user(&user_id)
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("User not found after insert".to_string()))?;

    if matches!(role, UserRole::Lender) {
        if let Err(e) = Identity::login(&http.extensions(), user.id.clone()) {
            log::warn!("Could not attach session identity after registration: {e}");
        }
    }

    Ok(Ok(HttpResponse::Ok().json(user)))
}

pub async fn get_users(
    query: web::Query<UsersQuery>,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let mgr = UserManager::new(&db);
    let mut users = mgr.get_all_users().map_err(AppError::Database)?;

    if let Some(ref em) = query.email {
        let needle = em.trim();
        if !needle.is_empty() {
            users.retain(|u| {
                u.email
                    .as_ref()
                    .map(|e| e.trim().eq_ignore_ascii_case(needle))
                    .unwrap_or(false)
            });
        }
    }

    if let Some(ref r) = query.role {
        let rl = r.trim().to_ascii_lowercase();
        if !rl.is_empty() {
            users.retain(|u| match rl.as_str() {
                "borrower" => matches!(u.role, UserRole::Borrower),
                "lender" => matches!(u.role, UserRole::Lender),
                _ => true,
            });
        }
    }

    if let Some(ref lid) = query.lender_id {
        let l = lid.trim();
        if !l.is_empty() && is_valid_4char_id(l) {
            users.retain(|u| {
                u.lender_id
                    .as_deref()
                    .map(|id| id.eq_ignore_ascii_case(l))
                    .unwrap_or(false)
            });
        }
    }

    Ok(Ok(HttpResponse::Ok().json(users)))
}

async fn create_loan(
    data: web::Json<CreateLoanReq>,
    identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let user = current_user(&identity, &db)?;
    if !matches!(user.role, UserRole::Lender) {
        return Err(AppError::InsufficientPermissions);
    }

    let borrower_id = data.borrower_id.trim();
    let lender_id = data.lender_id.trim();
    if !is_valid_4char_id(borrower_id) || !is_valid_4char_id(lender_id) || lender_id != user.id {
        return Err(AppError::InvalidInput("Invalid borrower/lender ID format".to_string()));
    }
    if data.principal <= 0.0 || data.months < 1 || data.months > 120 {
        return Err(AppError::InvalidInput(
            "Principal must be positive and term 1–120 months".to_string(),
        ));
    }

    let borrower = UserManager::new(&db)
        .get_user(borrower_id)
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::InvalidInput("Borrower not found".to_string()))?;
    if borrower.role != UserRole::Borrower {
        return Err(AppError::InvalidInput("That ID is not a borrower".to_string()));
    }
    if borrower.lender_id.as_deref() != Some(user.id.as_str()) {
        return Err(AppError::InvalidInput(
            "That borrower is not on your book".to_string(),
        ));
    }

    let tracker = LoanTracker::new(&db);
    let loan_id = tracker
        .create_loan(
            borrower_id.to_string(),
            lender_id.to_string(),
            data.principal,
            data.interest_rate,
            data.months,
        )
        .map_err(AppError::Database)?;

    Ok(Ok(HttpResponse::Ok().json(CreateLoanRes { id: loan_id })))
}

async fn get_loans(
    query: web::Query<LoansQuery>,
    identity: Option<Identity>,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let tracker = LoanTracker::new(&db);
    let mut loans = tracker.get_all_loans().map_err(AppError::Database)?;

    if let Some(ref ident) = identity {
        if let Ok(user) = current_user(ident, &db) {
            match user.role {
                UserRole::Lender => loans.retain(|loan| loan.lender_id == user.id),
                UserRole::Borrower => loans.retain(|loan| loan.borrower_id == user.id),
                UserRole::Admin => {}
            }
        }
    } else {
        if let Some(ref bid) = query.borrower_id {
            let b = bid.trim();
            if !b.is_empty() && b != "all" {
                loans.retain(|loan| loan.borrower_id == b);
            }
        }
        if let Some(ref lid) = query.lender_id {
            let l = lid.trim();
            if !l.is_empty() {
                loans.retain(|loan| loan.lender_id == l);
            }
        }
    }

    for loan in &mut loans {
        let _ = tracker.refresh_status(loan);
    }

    let mut payload: Vec<LoanApiJson> = loans.iter().map(|loan| loan_api_json(&db, loan)).collect();
    payload.sort_by(|a, b| a.health_score.partial_cmp(&b.health_score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(Ok(HttpResponse::Ok().json(payload)))
}

#[derive(Deserialize)]
struct AddBorrowerReq {
    name: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    principal: Option<f64>,
    #[serde(default)]
    interest_rate: Option<f64>,
    #[serde(default)]
    months: Option<i64>,
}

#[derive(Serialize)]
struct AddBorrowerRes {
    id: String,
    name: String,
    email: Option<String>,
    loan_id: Option<uuid::Uuid>,
}

async fn add_borrower(
    data: web::Json<AddBorrowerReq>,
    identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let lender = current_user(&identity, &db)?;
    if !matches!(lender.role, UserRole::Lender) {
        return Err(AppError::InsufficientPermissions);
    }
    let name = data.name.trim();
    if name.is_empty() {
        return Err(AppError::InvalidInput("Enter the borrower’s name".to_string()));
    }
    let email = data
        .email
        .as_deref()
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty());
    if let Some(ref em) = email {
        if !em.contains('@') {
            return Err(AppError::InvalidInput("Enter a valid email".to_string()));
        }
        if let Some(existing) = db.find_user_by_email_ci(em).map_err(AppError::Database)? {
            return Err(AppError::InvalidInput(format!(
                "A user with email {em} already exists (ID {})",
                existing.id
            )));
        }
    }

    let mgr = UserManager::new(&db);
    let borrower_id = mgr
        .register_user(
            name.to_string(),
            email.clone(),
            UserRole::Borrower,
            Some(lender.id.clone()),
            None,
        )
        .map_err(AppError::Database)?;

    let mut loan_id = None;
    if let Some(principal) = data.principal {
        if principal > 0.0 {
            let months = data.months.unwrap_or(12).clamp(1, 120);
            let rate = data.interest_rate.unwrap_or(8.5);
            let tracker = LoanTracker::new(&db);
            loan_id = Some(
                tracker
                    .create_loan(
                        borrower_id.clone(),
                        lender.id.clone(),
                        principal,
                        rate,
                        months,
                    )
                    .map_err(AppError::Database)?,
            );
        }
    }

    Ok(Ok(HttpResponse::Ok().json(AddBorrowerRes {
        id: borrower_id,
        name: name.to_string(),
        email,
        loan_id,
    })))
}

#[derive(Deserialize)]
struct PaymentReq {
    amount: f64,
    #[serde(default)]
    note: Option<String>,
}

async fn record_payment(
    path: web::Path<uuid::Uuid>,
    data: web::Json<PaymentReq>,
    identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let user = current_user(&identity, &db)?;
    let tracker = LoanTracker::new(&db);
    let loan = tracker
        .get_loan(path.into_inner())
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Loan not found".to_string()))?;

    let allowed = match user.role {
        UserRole::Lender => loan.lender_id == user.id,
        UserRole::Borrower => loan.borrower_id == user.id,
        UserRole::Admin => true,
    };
    if !allowed {
        return Err(AppError::InsufficientPermissions);
    }
    if data.amount <= 0.0 {
        return Err(AppError::InvalidInput("Payment amount must be positive".to_string()));
    }

    let updated = tracker
        .record_payment(loan.id, data.amount, data.note.clone())
        .map_err(AppError::Database)?;
    Ok(Ok(HttpResponse::Ok().json(loan_api_json(&db, &updated))))
}

#[derive(Deserialize)]
struct SignalReq {
    kind: String,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    proposed_date: Option<String>,
}

#[derive(Deserialize)]
struct SignalsQuery {
    #[serde(default)]
    borrower_id: Option<String>,
    #[serde(default)]
    lender_id: Option<String>,
    #[serde(default)]
    loan_id: Option<String>,
}

async fn create_signal(
    path: web::Path<uuid::Uuid>,
    data: web::Json<SignalReq>,
    identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let user = current_user(&identity, &db)?;
    if !matches!(user.role, UserRole::Borrower) {
        return Err(AppError::InsufficientPermissions);
    }
    let kind = data.kind.trim().to_ascii_lowercase();
    if kind != "can_pay_early" && kind != "concern" {
        return Err(AppError::InvalidInput(
            "Signal must be can_pay_early or concern".to_string(),
        ));
    }
    let tracker = LoanTracker::new(&db);
    let loan = tracker
        .get_loan(path.into_inner())
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Loan not found".to_string()))?;
    if loan.borrower_id != user.id {
        return Err(AppError::InsufficientPermissions);
    }

    let proposed_date = match data.proposed_date.as_deref() {
        Some(raw) if !raw.trim().is_empty() => {
            let raw = raw.trim();
            let parsed = chrono::DateTime::parse_from_rfc3339(raw)
                .map(|d| d.with_timezone(&chrono::Utc))
                .or_else(|_| {
                    chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d").map(|d| {
                        d.and_hms_opt(12, 0, 0)
                            .expect("noon")
                            .and_utc()
                    })
                })
                .map_err(|_| AppError::InvalidInput("Invalid proposed date".to_string()))?;
            Some(parsed)
        }
        _ => None,
    };

    let message = data
        .message
        .as_deref()
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| {
            if kind == "can_pay_early" {
                "I can pay this loan earlier than scheduled.".to_string()
            } else {
                "I may have trouble making the next payment.".to_string()
            }
        });

    let signal = LoanSignal {
        id: uuid::Uuid::new_v4().to_string(),
        loan_id: loan.id,
        borrower_id: user.id.clone(),
        lender_id: loan.lender_id.clone(),
        kind,
        message,
        proposed_date,
        created_at: chrono::Utc::now(),
        status: "open".to_string(),
    };
    db.save_signal(&signal).map_err(AppError::Database)?;

    Ok(Ok(HttpResponse::Ok().json(signal)))
}

async fn get_signals(
    query: web::Query<SignalsQuery>,
    identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let user = current_user(&identity, &db)?;
    let (borrower_id, lender_id) = match user.role {
        UserRole::Lender => (None, Some(user.id.as_str())),
        UserRole::Borrower => (Some(user.id.as_str()), None),
        UserRole::Admin => (
            query.borrower_id.as_deref(),
            query.lender_id.as_deref(),
        ),
    };
    let signals = db
        .load_signals(borrower_id, lender_id, query.loan_id.as_deref())
        .map_err(AppError::Database)?;
    Ok(Ok(HttpResponse::Ok().json(signals)))
}

async fn flag_overdues(
    identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let user = current_user(&identity, &db)?;
    if !matches!(user.role, UserRole::Lender) {
        return Err(AppError::InsufficientPermissions);
    }

    let tracker = LoanTracker::new(&db);
    let flagged_count = tracker.flag_overdues().map_err(AppError::Database)?;

    Ok(Ok(HttpResponse::Ok().json(serde_json::json!({
        "flagged_count": flagged_count
    }))))
}

async fn recommend_action(
    path: web::Path<uuid::Uuid>,
    identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let user = current_user(&identity, &db)?;
    let tracker = LoanTracker::new(&db);
    let loan = tracker
        .get_loan(path.into_inner())
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Loan not found".to_string()))?;

    let allowed = match user.role {
        UserRole::Lender => loan.lender_id == user.id,
        UserRole::Borrower => loan.borrower_id == user.id,
        UserRole::Admin => true,
    };
    if !allowed {
        return Err(AppError::InsufficientPermissions);
    }

    let health = scoring::evaluate(&loan, chrono::Utc::now());
    Ok(Ok(HttpResponse::Ok().json(serde_json::json!({
        "loan_id": loan.id,
        "risk_score": health.risk_score,
        "health_score": health.score,
        "health_band": health.band,
        "days_past_due": health.days_past_due,
        "recommended_action": action_key(&health.recommendation)
    }))))
}

pub async fn run_server(config: Config) -> std::io::Result<()> {
    log::info!("🚀 Smart Loan Recovery API starting at http://{}", config.server_addr());
    log::info!("Next.js UI: cd frontend && pnpm dev → http://127.0.0.1:3001");

    // Initialize Firebase authentication services
    log::info!("🔐 Initializing Firebase authentication...");
    let auth_state: web::Data<AuthState> = match init_auth_services().await {
        Ok(state) => {
            log::info!("✅ Firebase authentication initialized successfully");
            web::Data::new(state)
        }
        Err(e) => {
            log::error!("❌ Failed to initialize Firebase authentication: {}", e);
            log::warn!("⚠️  Starting server WITHOUT Firebase authentication - only demo mode will work");
            // Fallback: create minimal AuthState for demo mode without Firebase
            let fallback_jwt = Arc::new(
                crate::auth::services::jwt::JwtService::from_secret(
                    &std::env::var("JWT_SECRET").unwrap_or_else(|_| "insecure-demo-secret".to_string()),
                    24,
                    7,
                )
            );
            let fallback_firebase = Arc::new(crate::auth::services::firebase::FirebaseAuthService::default());
            web::Data::new(AuthState {
                firebase: fallback_firebase,
                jwt: fallback_jwt,
            })
        }
    };

    // Initialize token blacklist for logout functionality
    let token_blacklist = web::Data::new(Arc::new(TokenBlacklist::new()));

    log::info!("Server configured successfully");

    let _config_clone = config.clone();
    let is_production = std::env::var("RUST_ENV").map(|v| v == "production").unwrap_or(false);
    
    HttpServer::new(move || {
        let db = match Db::new_with_path(&_config_clone.database_url) {
            Ok(db) => db,
            Err(e) => {
                log::error!("Failed to create database connection: {}", e);
                panic!("Database connection failed");
            }
        };

        let key = Key::derive_from(&_config_clone.session_secret.as_bytes());
        let session_middleware = SessionMiddleware::builder(
            CookieSessionStore::default(),
            key,
        )
        .cookie_secure(is_production)
        .cookie_same_site(if is_production { SameSite::None } else { SameSite::Lax })
        .build();

        // Initialize JWT auth middleware
        let token_blacklist_arc = Arc::new(TokenBlacklist::new());
        let jwt_auth = JwtAuth::new(auth_state.jwt.clone(), token_blacklist_arc);

        App::new()
            .app_data(web::Data::new(db))
            .app_data(auth_state.clone())
            .app_data(token_blacklist.clone())
            .wrap(IdentityMiddleware::default())
            .wrap(session_middleware)
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin_fn(|origin, _| {
                        cors_origin_allowed(origin.to_str().unwrap_or(""))
                    })
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials()
                    .max_age(3600),
            )
            .route("/", web::get().to(|| async {
                Ok::<_, AppError>(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Smart Loan Recovery API is running!",
                    "version": "1.0.0",
                    "features": {
                        "firebase_auth": true,
                        "jwt_tokens": true,
                        "role_based_access": true,
                        "google_signin": true
                    },
                    "endpoints": {
                        "auth": [
                            "/auth/register",
                            "/auth/login",
                            "/auth/logout",
                            "/auth/refresh",
                            "/auth/verify",
                            "/auth/me",
                            "/auth/google",
                            "/auth/id-login",
                            "/auth/demo-login",
                        ],
                        "users": ["/users", "/borrowers"],
                        "loans": ["/loans", "/loans/{id}/payments", "/loans/{id}/signals"],
                        "recovery": ["/overdues", "/recommend/{loan_id}", "/signals"]
                    }
                })))
            }))
            .route("/test", web::post().to(|| async { HttpResponse::Ok().body("POST test successful!") }))
            // Firebase authentication routes (no JWT required)
            .configure(config_auth_routes)
            // Public routes used by the Next.js BFF (no JWT)
            .route("/users", web::get().to(get_users))
            .route("/users", web::post().to(register_user))
            .route("/borrowers", web::post().to(add_borrower))
            .route("/loans", web::get().to(get_loans))
            .route("/loans", web::post().to(create_loan))
            .route("/loans/{loan_id}/payments", web::post().to(record_payment))
            .route("/loans/{loan_id}/signals", web::post().to(create_signal))
            .route("/signals", web::get().to(get_signals))
            .route("/overdues", web::post().to(flag_overdues))
            .route("/recommend/{loan_id}", web::post().to(recommend_action))
            // Protected routes with JWT authentication
            .service(
                web::scope("/api")
                    .wrap(jwt_auth.clone())
                    .route("/users", web::get().to(get_users))
                    .route("/users", web::post().to(register_user))
                    .route("/borrowers", web::post().to(add_borrower))
                    .route("/loans", web::get().to(get_loans))
                    .route("/loans", web::post().to(create_loan))
                    .route("/loans/{loan_id}/payments", web::post().to(record_payment))
                    .route("/loans/{loan_id}/signals", web::post().to(create_signal))
                    .route("/signals", web::get().to(get_signals))
                    .route("/overdues", web::post().to(flag_overdues))
                    .route("/recommend/{loan_id}", web::post().to(recommend_action))
            )
    })
    .bind(config.server_addr())?
    .run()
    .await
}

