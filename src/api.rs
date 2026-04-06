use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, HttpRequest, HttpMessage, Result as ActixResult, middleware::Logger};
use actix_identity::{Identity, IdentityMiddleware};
use actix_web::cookie::Key;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use crate::db::Db;
use crate::user::UserManager;
use crate::loan::LoanTracker;
use crate::recovery::RecoveryEngine;
use crate::models::{Loan, LoanStatus, UserRole};
use crate::config::Config;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

fn is_valid_4char_id(id: &str) -> bool {
    id.len() == 4 && id.chars().all(|c| c.is_alphanumeric())
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
struct LoanApiJson {
    id: uuid::Uuid,
    borrower_id: uuid::Uuid,
    lender_id: uuid::Uuid,
    principal: f64,
    amount: f64,
    interest_rate: f64,
    status: String,
    recovery_status: f64,
    outstanding_amount: f64,
    risk_score: f64,
    ai_recommendation: String,
}

fn loan_api_json(loan: &Loan) -> LoanApiJson {
    let recovery_status = match loan.status {
        LoanStatus::Repaid => 100.0,
        LoanStatus::Active => 42.0,
        LoanStatus::Overdue => 28.0,
        LoanStatus::Defaulted => 12.0,
    };
    let amount = loan.principal;
    let outstanding_amount = amount * (1.0 - recovery_status / 100.0);
    let recovery = RecoveryEngine;
    let risk_score = recovery.predict_default(loan);
    let action = recovery.recommend_action(risk_score, 0);
    let ai_recommendation = match action {
        crate::recovery::RecoveryAction::SendReminder => "send_reminder",
        crate::recovery::RecoveryAction::RenegotiateTerms => "renegotiate_terms",
        crate::recovery::RecoveryAction::EscalateToCollection => "escalate_to_collection",
    }
    .to_string();
    LoanApiJson {
        id: loan.id,
        borrower_id: loan.borrower_id,
        lender_id: loan.lender_id,
        principal: loan.principal,
        amount,
        interest_rate: loan.interest_rate,
        status: format!("{:?}", loan.status).to_lowercase(),
        recovery_status,
        outstanding_amount,
        risk_score,
        ai_recommendation,
    }
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
) -> AppResult<ActixResult<HttpResponse>> {
    let mgr = UserManager::new(&db);
    let role = match data.role.as_str() {
        "borrower" => UserRole::Borrower,
        "lender" => UserRole::Lender,
        _ => return Err(AppError::InvalidInput("Role must be 'borrower' or 'lender'".to_string())),
    };

    let email = data.email.clone().filter(|e| !e.trim().is_empty());
    let lender_id = if let Some(ref ln) = data.lender_name {
        let lenders = mgr.get_all_users().map_err(AppError::Database)?;
        lenders.into_iter().find(|u| u.role == UserRole::Lender && u.name == *ln).map(|u| u.id)
    } else {
        None
    };
    let organization = data.organization.clone().filter(|o| !o.trim().is_empty());

    if role == UserRole::Borrower && lender_id.is_none() {
        return Err(AppError::InvalidInput("Borrowers must select a lender".to_string()));
    }

    if role == UserRole::Lender && organization.is_none() {
        return Err(AppError::InvalidInput("Lenders must specify an organization".to_string()));
    }

    let user_id = mgr
        .register_user(data.name.clone(), email, role, lender_id, organization)
        .map_err(AppError::Database)?;

    let user = mgr
        .get_user(&user_id)
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("User not found after insert".to_string()))?;

    Ok(Ok(HttpResponse::Ok().json(user)))
}

#[derive(Deserialize)]
pub struct LoginReq {
    /// User ID to login with (4-char alphanumeric)
    user_id: String,
}

pub async fn login(
    req: HttpRequest,
    data: web::Json<LoginReq>,
    _identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let mgr = UserManager::new(&db);

    let user_id = data.user_id.trim();
    if !is_valid_4char_id(user_id) {
        return Err(AppError::InvalidInput("User ID must be exactly 4 alphanumeric characters".to_string()));
    }

    let user = mgr
        .get_user(user_id)
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let _identity = Identity::login(&req.extensions(), user.id.clone())?;

    Ok(Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Login successful",
        "user_id": user.id,
        "role": user.role,
        "name": user.name
    }))))
}

pub async fn logout(identity: Identity) -> ActixResult<HttpResponse> {
    identity.logout();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logout successful"
    })))
}

async fn get_current_user(
    identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    if let Some(user_id_str) = identity.id().ok() {
        let mgr = UserManager::new(&db);
        if let Some(user) = mgr.get_user(&user_id_str)
            .map_err(|e| AppError::Database(e))? {
            return Ok(Ok(HttpResponse::Ok().json(&user)));
        }
    }

    Err(AppError::AuthRequired)
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
            users.retain(|u| u.lender_id.as_deref() == Some(l));
        }
    }

    Ok(Ok(HttpResponse::Ok().json(users)))
}

async fn create_loan(
    data: web::Json<CreateLoanReq>,
    identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let user_id = identity.id()
        .map_err(|_| AppError::AuthRequired)?;

    let mgr = UserManager::new(&db);
    let user = mgr.get_user(&user_id)
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::AuthRequired)?;

    if !matches!(user.role, UserRole::Lender) {
        return Err(AppError::InsufficientPermissions);
    }

    let borrower_id = data.borrower_id.trim();
    let lender_id = data.lender_id.trim();
    if !is_valid_4char_id(borrower_id) || !is_valid_4char_id(lender_id) || lender_id != user_id {
        return Err(AppError::InvalidInput("Invalid borrower/lender ID format".to_string()));
    }

    let tracker = LoanTracker::new(&db);
    let loan_id = tracker.create_loan(borrower_id.to_string(), lender_id.to_string(), data.principal, data.interest_rate, data.months)
        .map_err(|e| AppError::Database(e))?;

    Ok(Ok(HttpResponse::Ok().json(CreateLoanRes { id: loan_id })))
}

async fn get_loans(
    query: web::Query<LoansQuery>,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let tracker = LoanTracker::new(&db);
    let mut loans = tracker.get_all_loans().map_err(AppError::Database)?;

    if let Some(ref bid) = query.borrower_id {
        let b = bid.trim();
        if !b.is_empty() && b != "all" && is_valid_4char_id(b) {
            loans.retain(|_| false); // No filter impl for now
        }
    }

    if let Some(ref lid) = query.lender_id {
        let l = lid.trim();
        if !l.is_empty() && is_valid_4char_id(l) {
            loans.retain(|_loan| false); // No filter
        }
    }

    let payload: Vec<LoanApiJson> = loans.iter().map(loan_api_json).collect();
    Ok(Ok(HttpResponse::Ok().json(payload)))
}

async fn flag_overdues(
    identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let user_id = identity.id()
        .map_err(|_| AppError::AuthRequired)?;

    let mgr = UserManager::new(&db);
    let user = mgr.get_user(&user_id)
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::AuthRequired)?;

    if !matches!(user.role, UserRole::Lender) {
        return Err(AppError::InsufficientPermissions);
    }

    let tracker = LoanTracker::new(&db);
    let flagged_count = tracker.flag_overdues()
        .map_err(|e| AppError::Database(e))?;

    Ok(Ok(HttpResponse::Ok().json(serde_json::json!({
        "flagged_count": flagged_count
    }))))
}

async fn recommend_action(
    path: web::Path<uuid::Uuid>,
    identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let _user_id = identity.id()
        .map_err(|_| AppError::AuthRequired)?;

    let tracker = LoanTracker::new(&db);
    let recovery = RecoveryEngine;

    let loan = tracker.get_loan(path.into_inner())
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::NotFound("Loan not found".to_string()))?;

    let risk = recovery.predict_default(&loan);
    let action = recovery.recommend_action(risk, 0);

    Ok(Ok(HttpResponse::Ok().json(serde_json::json!({
        "loan_id": loan.id,
        "risk_score": risk,
        "recommended_action": action
    }))))
}

pub async fn run_server(config: Config) -> std::io::Result<()> {
    log::info!("🚀 Smart Loan Recovery Server starting at http://{}", config.server_addr());
    log::info!(
        "Web UI (same-origin, avoids CORS): http://{}/app/",
        config.server_addr()
    );

    let fe = std::path::Path::new(&config.frontend_dir);
    if !fe.is_dir() {
        log::warn!(
            "FRONTEND_DIR {:?} is not a directory — GET /app/ will fail. Run `cargo run` from the repo root or set FRONTEND_DIR.",
            fe
        );
    }

    log::info!("Server configured successfully");

    let _config_clone = config.clone();
    let frontend_dir = _config_clone.frontend_dir.clone();
    HttpServer::new(move || {
        let db = match Db::new_with_path(&_config_clone.database_url) {
            Ok(db) => db,
            Err(e) => {
                log::error!("Failed to create database connection: {}", e);
                panic!("Database connection failed");
            }
        };

        let key = Key::from(&_config_clone.session_secret.as_bytes());
        let session_middleware = SessionMiddleware::builder(
            CookieSessionStore::default(),
            key,
        )
        .cookie_secure(false)
        .build();

        App::new()
            .app_data(web::Data::new(db))
            .wrap(IdentityMiddleware::default())
            .wrap(session_middleware)
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .route("/", web::get().to(|| async {
                Ok::<_, AppError>(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Smart Loan Recovery API is running!",
                    "version": "1.0.0",
                    "endpoints": {
                        "ui": ["/app/", "/app/index.html"],
                        "auth": ["/auth/login", "/auth/logout", "/auth/me"],
                        "users": ["/users"],
                        "loans": ["/loans"],
                        "recovery": ["/overdues", "/recommend/{loan_id}"]
                    }
                })))
            }))
            .route("/test", web::post().to(|| async { HttpResponse::Ok().body("POST test successful!") }))
            .service(
                Files::new("/app", frontend_dir.clone())
                    .index_file("index.html")
                    .prefer_utf8(true),
            )
            .service(
                web::scope("/auth")
                    .route("/login", web::post().to(login))
                    .route("/logout", web::post().to(logout))
                    .route("/me", web::get().to(get_current_user))
            )
            .route("/users", web::get().to(get_users))
            .route("/users", web::post().to(register_user))
            .route("/loans", web::get().to(get_loans))
            .route("/loans", web::post().to(create_loan))
            .route("/overdues", web::post().to(flag_overdues))
            .route("/recommend/{loan_id}", web::post().to(recommend_action))
    })
    .bind(config.server_addr())?
    .run()
    .await
}

