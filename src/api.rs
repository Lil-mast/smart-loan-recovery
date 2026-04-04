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
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RegisterUserReq {
    name: String,
    role: String, // "borrower" or "lender"
    #[serde(default)]
    email: Option<String>,
}

#[derive(Deserialize)]
pub struct UsersQuery {
    #[serde(default)]
    email: Option<String>,
}

#[derive(Deserialize)]
pub struct LoansQuery {
    #[serde(default)]
    borrower_id: Option<String>,
}

/// Shape expected by `frontend/borrowers.html` and `frontend/lenders.html`.
#[derive(Serialize)]
struct LoanApiJson {
    id: Uuid,
    borrower_id: Uuid,
    lender_id: Uuid,
    principal: f64,
    amount: f64,
    interest_rate: f64,
    status: String,
    recovery_status: f64,
    outstanding_amount: f64,
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
    id: Uuid,
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
    if matches!(role, UserRole::Borrower) && email.is_none() {
        return Err(AppError::InvalidInput(
            "Borrower registration requires an email".to_string(),
        ));
    }

    if let Some(ref em) = email {
        if db
            .find_user_by_email_ci(em)
            .map_err(AppError::Database)?
            .is_some()
        {
            return Err(AppError::InvalidInput(
                "Email already registered".to_string(),
            ));
        }
    }

    let user_id = mgr
        .register_user(data.name.clone(), email.clone(), role)
        .map_err(AppError::Database)?;

    let user = mgr
        .get_user(user_id)
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("User not found after insert".to_string()))?;

    Ok(Ok(HttpResponse::Ok().json(user)))
}

#[derive(Deserialize)]
pub struct LoginReq {
    name: String,
}

pub async fn login(
    req: HttpRequest,
    data: web::Json<LoginReq>,
    _identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let mgr = UserManager::new(&db);

    // Find user by name
    let users = mgr.get_all_users()
        .map_err(|e| AppError::Database(e))?;

    let user = users.into_iter()
        .find(|u| u.name == data.name)
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Log the user in by storing their ID in the session
    let _identity = Identity::login(&req.extensions(), user.id.to_string())?;

    Ok(Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Login successful",
        "user_id": user.id,
        "role": user.role
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
    if let Some(user_id) = identity.id().ok() {
        let uuid = Uuid::parse_str(&user_id)
            .map_err(|_| AppError::InvalidInput("Invalid session".to_string()))?;

        let mgr = UserManager::new(&db);
        if let Some(user) = mgr.get_user(uuid)
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

    Ok(Ok(HttpResponse::Ok().json(users)))
}

async fn create_loan(
    data: web::Json<CreateLoanReq>,
    identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    // Check if user is authenticated and is a lender
    let user_id = identity.id()
        .map_err(|_| AppError::AuthRequired)?;

    let uuid = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::InvalidInput("Invalid session".to_string()))?;

    let mgr = UserManager::new(&db);
    let user = mgr.get_user(uuid)
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::AuthRequired)?;

    // Only lenders can create loans
    if !matches!(user.role, UserRole::Lender) {
        return Err(AppError::InsufficientPermissions);
    }

    let tracker = LoanTracker::new(&db);

    let borrower_uuid = Uuid::parse_str(&data.borrower_id)
        .map_err(|_| AppError::InvalidInput("Invalid borrower UUID format".to_string()))?;

    let lender_uuid = Uuid::parse_str(&data.lender_id)
        .map_err(|_| AppError::InvalidInput("Invalid lender UUID format".to_string()))?;

    let loan_id = tracker.create_loan(borrower_uuid, lender_uuid, data.principal, data.interest_rate, data.months)
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
        if !b.is_empty() && b != "all" {
            let uuid = Uuid::parse_str(b)
                .map_err(|_| AppError::InvalidInput("Invalid borrower_id (expected UUID)".to_string()))?;
            loans.retain(|l| l.borrower_id == uuid);
        }
    }

    let payload: Vec<LoanApiJson> = loans.iter().map(loan_api_json).collect();
    Ok(Ok(HttpResponse::Ok().json(payload)))
}

async fn flag_overdues(
    identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    // Check if user is authenticated and is a lender
    let user_id = identity.id()
        .map_err(|_| AppError::AuthRequired)?;

    let uuid = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::InvalidInput("Invalid session".to_string()))?;

    let mgr = UserManager::new(&db);
    let user = mgr.get_user(uuid)
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::AuthRequired)?;

    // Only lenders can flag overdues
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

// Similar endpoints for create_loan, flag_overdues, recommend_action
async fn recommend_action(
    path: web::Path<Uuid>,
    identity: Identity,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    // Check if user is authenticated
    let _user_id = identity.id()
        .map_err(|_| AppError::AuthRequired)?;

    let tracker = LoanTracker::new(&db);
    let recovery = RecoveryEngine;

    let loan = tracker.get_loan(path.into_inner())
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::NotFound("Loan not found".to_string()))?;

    let risk = recovery.predict_default(&loan);
    let action = recovery.recommend_action(risk, 0);  // Mock history

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
        // Create a new DB connection for each worker
        let db = match Db::new_with_path(&_config_clone.database_url) {
            Ok(db) => db,
            Err(e) => {
                log::error!("Failed to create database connection: {}", e);
                panic!("Database connection failed");
            }
        };

        // Create session middleware for each worker
        let key = Key::from(&_config_clone.session_secret.as_bytes()); // Use configured session secret
        let session_middleware = SessionMiddleware::builder(
            CookieSessionStore::default(),
            key,
        )
        .cookie_secure(false) // Set to true in production with HTTPS
        .build();

        App::new()
            .app_data(web::Data::new(db))
            .wrap(IdentityMiddleware::default())
            .wrap(session_middleware)
            .wrap(Logger::default())
            // Outermost: must run first so OPTIONS / cross-origin work before session touches the request
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            // Public routes
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
            // Auth routes
            .service(
                web::scope("/auth")
                    .route("/login", web::post().to(login))
                    .route("/logout", web::post().to(logout))
                    .route("/me", web::get().to(get_current_user))
            )
            // Protected routes
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