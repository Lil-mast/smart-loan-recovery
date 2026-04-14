//! Google Sign-In Handler
//!
//! Handles authentication via Google Sign-In through Firebase
//! - Verifies Google ID token from Firebase
//! - Creates or links user account
//! - Returns JWT tokens

#![allow(dead_code)]

use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use crate::auth::{
    models::{AuthResponse, GoogleSignInRequest, UserInfo},
    services::FirebaseAuthService,
    AuthState,
};
use crate::db::Db;
use crate::models::UserRole;

/// Handle Google Sign-In
pub async fn google_sign_in(
    auth_state: web::Data<AuthState>,
    db: web::Data<Db>,
    req: web::Json<GoogleSignInRequest>,
) -> impl Responder {
    log::info!("Processing Google Sign-In request");

    // Verify the Google ID token through Firebase
    let firebase_user = match auth_state
        .firebase
        .verify_id_token(&req.id_token)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            log::warn!("Google Sign-In failed: {}", e);
            return HttpResponse::Unauthorized().json(json!({
                "error": "Invalid or expired Google token"
            }));
        }
    };

    let email = firebase_user.email.clone().unwrap_or_default();
    let name = firebase_user.name.clone().unwrap_or_else(|| {
        email.split('@').next().unwrap_or("Google User").to_string()
    });

    log::info!(
        "Google Sign-In successful for user: {} ({})",
        firebase_user.uid,
        email
    );

    // Check if user already exists
    let (local_user_id, role) = match FirebaseAuthService::get_user_link(db.as_ref(), &firebase_user.uid) {
        Ok(Some(link)) => {
            log::info!("Existing user linked: {}", link.local_user_id);
            (link.local_user_id, link.role)
        }
        Ok(None) => {
            // New user - create link with default role
            log::info!("Creating new user from Google Sign-In");
            let default_role = UserRole::Borrower;
            
            match auth_state
                .firebase
                .link_user(db.as_ref(), &firebase_user.uid, &email, &name, default_role.clone())
                .await
            {
                Ok(id) => {
                    log::info!("New user created and linked: {}", id);
                    (id, default_role)
                }
                Err(e) => {
                    log::error!("Failed to link Google user: {}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "error": "Failed to complete registration"
                    }));
                }
            }
        }
        Err(e) => {
            log::error!("Database error during Google Sign-In: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Authentication failed"
            }));
        }
    };

    // Generate tokens
    let (access_token, refresh_token) = match auth_state
        .jwt
        .generate_token_pair(
            &firebase_user.uid,
            &email,
            firebase_user.email_verified,
            role.clone(),
            &local_user_id,
        ) {
        Ok(tokens) => tokens,
        Err(e) => {
            log::error!("Failed to generate tokens: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to generate authentication tokens"
            }));
        }
    };

    let response = AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: auth_state.jwt.get_token_expiry(),
        user: UserInfo {
            uid: firebase_user.uid,
            email,
            email_verified: firebase_user.email_verified,
            name,
            role,
            photo_url: firebase_user.picture,
            local_user_id,
        },
    };

    log::info!("Google Sign-In completed successfully");
    HttpResponse::Ok().json(response)
}

/// Verify Google token without creating session
/// Useful for checking if a Google token is valid before processing
pub async fn verify_google_token(
    auth_state: web::Data<AuthState>,
    req: web::Json<GoogleSignInRequest>,
) -> impl Responder {
    log::info!("Verifying Google token");

    match auth_state.firebase.verify_id_token(&req.id_token).await {
        Ok(user) => {
            HttpResponse::Ok().json(json!({
                "valid": true,
                "user": {
                    "uid": user.uid,
                    "email": user.email,
                    "name": user.name,
                    "email_verified": user.email_verified,
                    "picture": user.picture
                }
            }))
        }
        Err(e) => {
            log::warn!("Google token verification failed: {}", e);
            HttpResponse::Unauthorized().json(json!({
                "valid": false,
                "error": format!("{}", e)
            }))
        }
    }
}
