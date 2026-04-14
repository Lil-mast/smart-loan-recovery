//! Firebase Authentication Module
//! 
//! This module provides:
//! - Firebase Admin SDK integration
//! - JWT token verification and generation
//! - Role-based access control (RBAC)
//! - Email/Password and Google Sign-In support
//! - Logout and token refresh endpoints

pub mod handlers;
pub mod middleware;
pub mod models;
pub mod services;
pub mod utils;

use actix_web::web;
use services::{firebase::FirebaseAuthService, jwt::JwtService};
use std::sync::Arc;

/// Application data shared across handlers
pub struct AuthState {
    pub firebase: Arc<FirebaseAuthService>,
    pub jwt: Arc<JwtService>,
}

/// Configure authentication routes
pub fn config_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            // Public: Firebase configuration for frontend
            .route("/config", web::get().to(firebase_config_handler))
            // Authentication endpoints
            .route("/login", web::post().to(handlers::auth::login))
            .route("/register", web::post().to(handlers::auth::register))
            .route("/logout", web::post().to(handlers::auth::logout))
            .route("/refresh", web::post().to(handlers::auth::refresh_token))
            .route("/verify", web::post().to(handlers::auth::verify_token))
            // Google Sign-In
            .route("/google", web::post().to(handlers::google::google_sign_in))
            // User profile
            .route("/me", web::get().to(handlers::auth::get_current_user))
            .route("/me", web::put().to(handlers::auth::update_profile))
    );
}

/// Handler to serve Firebase configuration to frontend
async fn firebase_config_handler() -> impl actix_web::Responder {
    use actix_web::HttpResponse;
    use serde_json::json;
    
    // Load config from environment
    let api_key = std::env::var("FIREBASE_API_KEY").unwrap_or_default();
    let project_id = std::env::var("FIREBASE_PROJECT_ID").unwrap_or_default();
    let auth_domain = std::env::var("FIREBASE_AUTH_DOMAIN").unwrap_or_default();
    
    if api_key.is_empty() || project_id.is_empty() {
        return HttpResponse::ServiceUnavailable().json(json!({
            "error": "Firebase configuration not available"
        }));
    }
    
    HttpResponse::Ok().json(json!({
        "apiKey": api_key,
        "authDomain": auth_domain,
        "projectId": project_id,
        "storageBucket": std::env::var("FIREBASE_STORAGE_BUCKET").unwrap_or_default(),
        "messagingSenderId": std::env::var("FIREBASE_MESSAGING_SENDER_ID").unwrap_or_default(),
        "appId": std::env::var("FIREBASE_APP_ID").unwrap_or_default(),
    }))
}

/// Initialize authentication services
pub async fn init_auth_services() -> Result<AuthState, Box<dyn std::error::Error>> {
    // Load Firebase configuration from .env.local
    dotenv::from_filename(".env.local").ok();
    
    let firebase = Arc::new(FirebaseAuthService::new().await?);
    let jwt = Arc::new(JwtService::new()?);
    
    Ok(AuthState { firebase, jwt })
}
