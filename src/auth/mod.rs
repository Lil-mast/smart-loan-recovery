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

/// Initialize authentication services
pub async fn init_auth_services() -> Result<AuthState, Box<dyn std::error::Error>> {
    // Load Firebase configuration from .env.firebase
    dotenv::from_filename(".env.firebase").ok();
    
    let firebase = Arc::new(FirebaseAuthService::new().await?);
    let jwt = Arc::new(JwtService::new()?);
    
    Ok(AuthState { firebase, jwt })
}
