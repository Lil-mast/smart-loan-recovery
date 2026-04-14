//! Authentication Models
//!
//! Data structures for Firebase authentication

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::UserRole;

/// Firebase User Claims stored in JWT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseClaims {
    /// User ID (Firebase UID)
    pub sub: String,
    /// Email address
    pub email: Option<String>,
    /// Email verification status
    pub email_verified: bool,
    /// Token issuer (should be Firebase)
    pub iss: String,
    /// Audience (Firebase project ID)
    pub aud: String,
    /// Issued at timestamp
    pub iat: i64,
    /// Expiration timestamp
    pub exp: i64,
    /// Custom claims - user role
    pub role: Option<UserRole>,
    /// Custom claims - local database user ID
    pub local_user_id: Option<String>,
}

/// Login request payload
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Registration request payload
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: UserRole,
}

/// Google Sign-In request
#[derive(Debug, Deserialize)]
pub struct GoogleSignInRequest {
    /// Google ID Token from Firebase
    pub id_token: String,
}

/// Token refresh request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Authentication response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

/// User information returned in auth responses
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub uid: String,
    pub email: String,
    pub email_verified: bool,
    pub name: String,
    pub role: UserRole,
    pub photo_url: Option<String>,
    pub local_user_id: String,
}

/// Token verification response
#[derive(Debug, Serialize)]
pub struct TokenVerificationResponse {
    pub valid: bool,
    pub user: Option<UserInfo>,
    pub error: Option<String>,
}

/// Logout request
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
}

/// Firebase token decoded from Admin SDK
#[derive(Debug, Clone)]
pub struct DecodedFirebaseToken {
    pub uid: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub _claims: serde_json::Value,
}

/// Link between Firebase user and local database user
#[derive(Debug, Serialize, Deserialize)]
pub struct UserLink {
    pub firebase_uid: String,
    pub local_user_id: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Profile update request
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}
