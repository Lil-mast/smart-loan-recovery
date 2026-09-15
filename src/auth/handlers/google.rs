//! Google Sign-In Handler
//!
//! Handles authentication via Google Sign-In through Firebase
//! - Verifies Google ID token from Firebase
//! - Creates or links user account
//! - Returns JWT tokens

use actix_identity::Identity;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use serde_json::json;

use crate::auth::{
    models::{AuthResponse, GoogleSignInRequest, UserInfo, UserLink},
    services::FirebaseAuthService,
    AuthState,
};
use crate::db::Db;
use crate::models::UserRole;
use crate::user::UserManager;

fn parse_intended_role(raw: Option<&str>) -> Option<UserRole> {
    match raw.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        Some("lender") => Some(UserRole::Lender),
        Some("borrower") => Some(UserRole::Borrower),
        _ => None,
    }
}

fn role_label(role: &UserRole) -> &'static str {
    match role {
        UserRole::Lender => "lender",
        UserRole::Borrower => "borrower",
        UserRole::Admin => "admin",
    }
}

fn resolve_lender_id(db: &Db, raw: Option<&str>) -> Result<String, String> {
    let id = raw.unwrap_or("").trim();
    if id.len() != 4 || !id.chars().all(|c| c.is_alphanumeric()) {
        return Err("Enter the lender’s 4-character account ID".to_string());
    }
    let mgr = UserManager::new(db);
    let user = mgr
        .get_user(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No lender found with that ID".to_string())?;
    if user.role != UserRole::Lender {
        return Err("That ID is not a lender account".to_string());
    }
    Ok(user.id)
}

/// Handle Google Sign-In
pub async fn google_sign_in(
    auth_state: web::Data<AuthState>,
    db: web::Data<Db>,
    http: HttpRequest,
    identity: Option<Identity>,
    req: web::Json<GoogleSignInRequest>,
) -> impl Responder {
    log::info!("Processing Google Sign-In request");

    let firebase_user = match auth_state.firebase.verify_id_token(&req.id_token).await {
        Ok(user) => user,
        Err(e) => {
            log::warn!("Google Sign-In failed: {}", e);
            return HttpResponse::Unauthorized().json(json!({
                "error": crate::auth::utils::firebase_auth_error_message(&e.to_string())
            }));
        }
    };

    let email = firebase_user.email.clone().unwrap_or_default();
    let name = firebase_user.name.clone().unwrap_or_else(|| {
        email
            .split('@')
            .next()
            .unwrap_or("Google User")
            .to_string()
    });

    let requested = parse_intended_role(req.role.as_deref()).unwrap_or(UserRole::Borrower);

    let (local_user_id, role) = match FirebaseAuthService::get_user_link(db.as_ref(), &firebase_user.uid)
    {
        Ok(Some(link)) => {
            if link.role != requested {
                return HttpResponse::Conflict().json(json!({
                    "error": format!(
                        "This Google account is already a {}. Sign in on that tab, or use a different Google account.",
                        role_label(&link.role)
                    ),
                    "existing_role": role_label(&link.role)
                }));
            }
            (link.local_user_id, link.role)
        }
        Ok(None) => {
            if req.link_existing.unwrap_or(false) {
                let Some(ref ident) = identity else {
                    return HttpResponse::BadRequest().json(json!({
                        "error": "Sign in with your account ID before linking Google."
                    }));
                };
                let Ok(existing_id) = ident.id() else {
                    return HttpResponse::BadRequest().json(json!({
                        "error": "Could not read current session"
                    }));
                };
                let mgr = UserManager::new(db.as_ref());
                match mgr.get_user(&existing_id) {
                    Ok(Some(user)) => {
                        let link = UserLink {
                            firebase_uid: firebase_user.uid.clone(),
                            local_user_id: user.id.clone(),
                            email: email.clone(),
                            role: user.role.clone(),
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        };
                        if let Err(e) = db.save_user_link(&link) {
                            log::error!("Failed to attach Google to account: {e}");
                            return HttpResponse::InternalServerError().json(json!({
                                "error": "Failed to link Google"
                            }));
                        }
                        (user.id, user.role)
                    }
                    Ok(None) => {
                        return HttpResponse::BadRequest().json(json!({
                            "error": "Account not found"
                        }));
                    }
                    Err(e) => {
                        log::error!("Database error linking Google: {e}");
                        return HttpResponse::InternalServerError().json(json!({
                            "error": "Authentication failed"
                        }));
                    }
                }
            } else {
                let lender_id = if requested == UserRole::Borrower {
                    match resolve_lender_id(db.as_ref(), req.lender_id.as_deref()) {
                        Ok(id) => Some(id),
                        Err(msg) => {
                            return HttpResponse::BadRequest().json(json!({ "error": msg }));
                        }
                    }
                } else {
                    None
                };

                let organization = if requested == UserRole::Lender {
                    let org = req
                        .organization
                        .as_deref()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if org.is_empty() {
                        return HttpResponse::BadRequest().json(json!({
                            "error": "Enter your company name"
                        }));
                    }
                    Some(org)
                } else {
                    None
                };

                match auth_state
                    .firebase
                    .link_user(
                        db.as_ref(),
                        &firebase_user.uid,
                        &email,
                        &name,
                        requested.clone(),
                        lender_id,
                        organization,
                    )
                    .await
                {
                    Ok(id) => (id, requested),
                    Err(e) => {
                        log::error!("Failed to link Google user: {}", e);
                        return HttpResponse::InternalServerError().json(json!({
                            "error": "Failed to complete registration"
                        }));
                    }
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

    if let Err(e) = Identity::login(&http.extensions(), local_user_id.clone()) {
        log::warn!("Could not attach session identity after Google sign-in: {e}");
    }

    let (access_token, refresh_token) = match auth_state.jwt.generate_token_pair(
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
pub async fn verify_google_token(
    auth_state: web::Data<AuthState>,
    req: web::Json<GoogleSignInRequest>,
) -> impl Responder {
    log::info!("Verifying Google token");

    match auth_state.firebase.verify_id_token(&req.id_token).await {
        Ok(user) => HttpResponse::Ok().json(json!({
            "valid": true,
            "user": {
                "uid": user.uid,
                "email": user.email,
                "name": user.name,
                "email_verified": user.email_verified,
                "picture": user.picture
            }
        })),
        Err(e) => {
            log::warn!("Google token verification failed: {}", e);
            HttpResponse::Unauthorized().json(json!({
                "valid": false,
                "error": format!("{}", e)
            }))
        }
    }
}
