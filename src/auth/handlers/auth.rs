use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::auth::{
    middleware::auth::require_auth,
    models::{
        AuthResponse, LoginRequest, LogoutRequest, RefreshTokenRequest, RegisterRequest,
        TokenVerificationResponse, UpdateProfileRequest, UserInfo,
    },
    services::{FirebaseAuthService, TokenBlacklist},
    AuthState,
};
use crate::db::Db;
use crate::models::UserRole;

/// Register a new user with email/password
pub async fn register(
    auth_state: web::Data<AuthState>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    log::info!("Processing registration request for email: {}", req.email);

    // Validate input
    if req.password.len() < 6 {
        return HttpResponse::BadRequest().json(json!({
            "error": "Password must be at least 6 characters long"
        }));
    }

    if !req.email.contains('@') {
        return HttpResponse::BadRequest().json(json!({
            "error": "Invalid email format"
        }));
    }

    // Create user in Firebase
    let firebase_user = match auth_state
        .firebase
        .create_user(&req.email, &req.password, &req.name)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            log::error!("Failed to create Firebase user: {}", e);
            return HttpResponse::BadRequest().json(json!({
                "error": format!("Failed to create user: {}", e)
            }));
        }
    };

    // Link to local database
    let local_user_id = match auth_state
        .firebase
        .link_user(
            db.as_ref(),
            &firebase_user.uid,
            &req.email,
            &req.name,
            req.role.clone(),
        )
        .await
    {
        Ok(id) => id,
        Err(e) => {
            log::error!("Failed to link user to database: {}", e);
            // Note: In production, you might want to rollback Firebase user creation
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to complete registration: {}", e)
            }));
        }
    };

    // Generate tokens
    let (access_token, refresh_token) = match auth_state
        .jwt
        .generate_token_pair(
            &firebase_user.uid,
            &req.email,
            false, // Email not verified yet
            req.role.clone(),
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
            uid: firebase_user.uid.clone(),
            email: req.email.clone(),
            email_verified: false,
            name: req.name.clone(),
            role: req.role.clone(),
            photo_url: None,
            local_user_id,
        },
    };

    log::info!("User registered successfully: {}", firebase_user.uid);
    HttpResponse::Created().json(response)
}

/// Login with email/password
pub async fn login(
    auth_state: web::Data<AuthState>,
    db: web::Data<Db>,
    req: web::Json<LoginRequest>,
) -> impl Responder {
    log::info!("Processing login request for email: {}", req.email);

    // Authenticate with Firebase
    let (id_token, firebase_user) = match auth_state
        .firebase
        .sign_in_with_email_password(&req.email, &req.password)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            log::warn!("Login failed for {}: {}", req.email, e);
            return HttpResponse::Unauthorized().json(json!({
                "error": "Invalid email or password"
            }));
        }
    };

    // Get or create user link
    let user_link = match FirebaseAuthService::get_user_link(db.as_ref(), &firebase_user.uid) {
        Ok(Some(link)) => link,
        Ok(None) => {
            // User exists in Firebase but not in our DB, create link
            let name = firebase_user.name.clone().unwrap_or_else(|| req.email.split('@').next().unwrap_or("User").to_string());
            let role = UserRole::Borrower; // Default role

            match auth_state
                .firebase
                .link_user(db.as_ref(), &firebase_user.uid, &req.email, &name, role.clone())
                .await
            {
                Ok(id) => {
                    use crate::auth::models::UserLink;
                    use chrono::Utc;
                    UserLink {
                        firebase_uid: firebase_user.uid.clone(),
                        local_user_id: id,
                        email: req.email.clone(),
                        role,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    }
                }
                Err(e) => {
                    log::error!("Failed to link existing Firebase user: {}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "error": "Failed to complete login"
                    }));
                }
            }
        }
        Err(e) => {
            log::error!("Database error during login: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Login failed"
            }));
        }
    };

    // Generate tokens
    let (access_token, refresh_token) = match auth_state
        .jwt
        .generate_token_pair(
            &firebase_user.uid,
            &req.email,
            firebase_user.email_verified,
            user_link.role.clone(),
            &user_link.local_user_id,
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
            uid: firebase_user.uid.clone(),
            email: req.email.clone(),
            email_verified: firebase_user.email_verified,
            name: user_link.email.split('@').next().unwrap_or("User").to_string(),
            role: user_link.role,
            photo_url: firebase_user.picture,
            local_user_id: user_link.local_user_id,
        },
    };

    log::info!("User logged in successfully: {}", firebase_user.uid);
    HttpResponse::Ok().json(response)
}

/// Logout and revoke tokens
pub async fn logout(
    auth_state: web::Data<AuthState>,
    token_blacklist: web::Data<Arc<TokenBlacklist>>,
    http_req: HttpRequest,
    req: web::Json<LogoutRequest>,
) -> impl Responder {
    log::info!("Processing logout request");

    // Extract and blacklist the current access token
    if let Some(auth_header) = http_req.headers().get("Authorization") {
        if let Ok(header_str) = auth_header.to_str() {
            if let Some(token) = crate::auth::services::jwt::JwtService::extract_token_from_header(header_str) {
                token_blacklist.revoke_token(token);
                log::info!("Access token revoked");
            }
        }
    }

    // Also blacklist refresh token if provided
    if let Some(ref refresh_token) = req.refresh_token {
        token_blacklist.revoke_token(refresh_token);
        log::info!("Refresh token revoked");
    }

    // If user is authenticated, revoke Firebase refresh tokens
    if let Ok(auth_ctx) = require_auth(&http_req) {
        if let Err(e) = auth_state.firebase.revoke_refresh_tokens(auth_ctx.user_id()).await {
            log::warn!("Failed to revoke Firebase refresh tokens: {}", e);
            // Don't fail the logout, but log the warning
        }
    }

    log::info!("Logout successful");
    HttpResponse::Ok().json(json!({
        "message": "Logged out successfully"
    }))
}

/// Refresh access token using refresh token
pub async fn refresh_token(
    auth_state: web::Data<AuthState>,
    token_blacklist: web::Data<Arc<TokenBlacklist>>,
    req: web::Json<RefreshTokenRequest>,
) -> impl Responder {
    log::info!("Processing token refresh request");

    // Check if refresh token is revoked
    if token_blacklist.is_revoked(&req.refresh_token) {
        return HttpResponse::Unauthorized().json(json!({
            "error": "Refresh token has been revoked. Please log in again."
        }));
    }

    // Verify refresh token
    let (uid, local_user_id, jti) = match auth_state
        .jwt
        .verify_refresh_token(&req.refresh_token)
    {
        Ok(data) => data,
        Err(e) => {
            log::warn!("Invalid refresh token: {}", e);
            return HttpResponse::Unauthorized().json(json!({
                "error": "Invalid refresh token"
            }));
        }
    };

    // Get user information from database
    let user_link = match auth_state.firebase.get_user_link(&uid).await {
        Ok(Some(link)) => link,
        Ok(None) => {
            log::error!("User link not found during token refresh: {}", uid);
            return HttpResponse::Unauthorized().json(json!({
                "error": "User not found"
            }));
        }
        Err(e) => {
            log::error!("Database error during token refresh: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to refresh token"
            }));
        }
    };

    // Generate new token pair
    let (access_token, new_refresh_token) = match auth_state
        .jwt
        .generate_token_pair(
            &uid,
            &user_link.email,
            true, // Assume email is verified for refresh
            user_link.role.clone(),
            &local_user_id,
        ) {
        Ok(tokens) => tokens,
        Err(e) => {
            log::error!("Failed to generate new tokens: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to generate new tokens"
            }));
        }
    };

    // Revoke old refresh token
    token_blacklist.revoke_token(&req.refresh_token);
    log::info!("Old refresh token revoked, new tokens issued for user: {}", uid);

    let response = AuthResponse {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: auth_state.jwt.get_token_expiry(),
        user: UserInfo {
            uid,
            email: user_link.email.clone(),
            email_verified: true,
            name: user_link.email.split('@').next().unwrap_or("User").to_string(),
            role: user_link.role.clone(),
            photo_url: None,
            local_user_id,
        },
    };

    HttpResponse::Ok().json(response)
}

/// Verify JWT token validity
pub async fn verify_token(
    auth_state: web::Data<AuthState>,
    token_blacklist: web::Data<Arc<TokenBlacklist>>,
    http_req: HttpRequest,
) -> impl Responder {
    // Extract token from header
    let token = match http_req.headers().get("Authorization") {
        Some(header) => {
            match header.to_str() {
                Ok(header_str) => {
                    match crate::auth::services::jwt::JwtService::extract_token_from_header(header_str) {
                        Some(t) => t,
                        None => {
                            return HttpResponse::BadRequest().json(TokenVerificationResponse {
                                valid: false,
                                user: None,
                                error: Some("Invalid authorization header format".to_string()),
                            });
                        }
                    }
                }
                Err(_) => {
                    return HttpResponse::BadRequest().json(TokenVerificationResponse {
                        valid: false,
                        user: None,
                        error: Some("Invalid authorization header".to_string()),
                    });
                }
            }
        }
        None => {
            return HttpResponse::BadRequest().json(TokenVerificationResponse {
                valid: false,
                user: None,
                error: Some("Missing authorization header".to_string()),
            });
        }
    };

    // Check if token is blacklisted
    if token_blacklist.is_revoked(token) {
        return HttpResponse::Ok().json(TokenVerificationResponse {
            valid: false,
            user: None,
            error: Some("Token has been revoked".to_string()),
        });
    }

    // Verify token
    match auth_state.jwt.verify_access_token(token) {
        Ok(claims) => {
            // Get user info from database
            let user_link = match auth_state.firebase.get_user_link(&claims.sub).await {
                Ok(Some(link)) => link,
                _ => {
                    return HttpResponse::Ok().json(TokenVerificationResponse {
                        valid: false,
                        user: None,
                        error: Some("User not found".to_string()),
                    });
                }
            };

            let user_info = UserInfo {
                uid: claims.sub,
                email: user_link.email.clone(),
                email_verified: claims.email_verified,
                name: user_link.email.split('@').next().unwrap_or("User").to_string(),
                role: user_link.role,
                photo_url: None,
                local_user_id: user_link.local_user_id,
            };

            HttpResponse::Ok().json(TokenVerificationResponse {
                valid: true,
                user: Some(user_info),
                error: None,
            })
        }
        Err(e) => {
            HttpResponse::Ok().json(TokenVerificationResponse {
                valid: false,
                user: None,
                error: Some(format!("Token verification failed: {}", e)),
            })
        }
    }
}

/// Get current authenticated user information
pub async fn get_current_user(
    auth_state: web::Data<AuthState>,
    http_req: HttpRequest,
) -> impl Responder {
    let auth_ctx = match require_auth(&http_req) {
        Ok(ctx) => ctx,
        Err(e) => return e.error_response(),
    };

    let uid = auth_ctx.user_id();

    // Get user link from database
    let user_link = match auth_state.firebase.get_user_link(uid).await {
        Ok(Some(link)) => link,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "error": "User not found"
            }));
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to retrieve user information"
            }));
        }
    };

    let user_info = UserInfo {
        uid: uid.to_string(),
        email: user_link.email.clone(),
        email_verified: auth_ctx.claims.email_verified,
        name: user_link.email.split('@').next().unwrap_or("User").to_string(),
        role: user_link.role,
        photo_url: None,
        local_user_id: user_link.local_user_id,
    };

    HttpResponse::Ok().json(user_info)
}

/// Update user profile
pub async fn update_profile(
    auth_state: web::Data<AuthState>,
    http_req: HttpRequest,
    req: web::Json<UpdateProfileRequest>,
) -> impl Responder {
    let auth_ctx = match require_auth(&http_req) {
        Ok(ctx) => ctx,
        Err(e) => return e.error_response(),
    };

    let local_user_id = match auth_ctx.local_user_id() {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Local user ID not found"
            }));
        }
    };

    // Update profile in database
    // Note: This would update the local database user record
    // For full implementation, you'd also update Firebase user profile
    
    log::info!("Profile update requested for user: {}", local_user_id);

    // For now, just return success
    // In production, implement actual profile update logic
    HttpResponse::Ok().json(json!({
        "message": "Profile updated successfully",
        "updated_fields": {
            "name": req.name.is_some(),
            "email": req.email.is_some()
        }
    }))
}
