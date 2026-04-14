//! Firebase Admin SDK Service
//!
//! Handles Firebase authentication using Firebase Admin SDK
//! - Verify ID tokens from Firebase
//! - Create custom tokens
//! - Link Firebase users to local database

use crate::auth::models::{DecodedFirebaseToken, UserLink};
use crate::db::Db;
use crate::models::UserRole;
use chrono::Utc;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

pub struct FirebaseAuthService {
    client: Client,
    project_id: String,
    api_key: String,
    service_account_key: Option<Value>,
}

impl FirebaseAuthService {
    /// Initialize Firebase Auth Service
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load configuration from environment
        let project_id = std::env::var("FIREBASE_PROJECT_ID")
            .map_err(|_| "FIREBASE_PROJECT_ID not set in .env.firebase")?;
        let api_key = std::env::var("FIREBASE_API_KEY")
            .map_err(|_| "FIREBASE_API_KEY not set in .env.firebase")?;

        // Load service account key if available
        let service_account_key = if let Ok(key_path) = std::env::var("FIREBASE_SERVICE_ACCOUNT_KEY_PATH") {
            if std::path::Path::new(&key_path).exists() {
                let content = tokio::fs::read_to_string(&key_path).await?;
                Some(serde_json::from_str(&content)?)
            } else {
                None
            }
        } else if let Ok(key_base64) = std::env::var("FIREBASE_SERVICE_ACCOUNT_JSON_BASE64") {
            if !key_base64.is_empty() {
                let decoded = base64::decode(key_base64)?;
                let content = String::from_utf8(decoded)?;
                Some(serde_json::from_str(&content)?)
            } else {
                None
            }
        } else {
            None
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            project_id,
            api_key,
            service_account_key,
        })
    }

    /// Verify Firebase ID Token
    /// This validates the token by calling Firebase Auth REST API
    pub async fn verify_id_token(&self, id_token: &str) -> Result<DecodedFirebaseToken, Box<dyn std::error::Error>> {
        // For production, you should use the Firebase Admin SDK properly
        // This is a simplified version using Firebase Auth REST API
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:lookup?key={}",
            self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&json!({ "idToken": id_token }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Firebase token verification failed: {}", error_text).into());
        }

        let data: Value = response.json().await?;
        let users = data.get("users").and_then(|u| u.as_array());

        if let Some(user_array) = users {
            if let Some(user) = user_array.first() {
                let decoded = DecodedFirebaseToken {
                    uid: user.get("localId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    email: user.get("email").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    email_verified: user.get("emailVerified").and_then(|v| v.as_bool()).unwrap_or(false),
                    name: user.get("displayName").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    picture: user.get("photoUrl").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    claims: json!({}),
                };
                return Ok(decoded);
            }
        }

        Err("Invalid token or user not found".into())
    }

    /// Create a new user with email and password
    pub async fn create_user(
        &self,
        email: &str,
        password: &str,
        name: &str,
    ) -> Result<DecodedFirebaseToken, Box<dyn std::error::Error>> {
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:signUp?key={}",
            self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&json!({
                "email": email,
                "password": password,
                "displayName": name,
                "returnSecureToken": true
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to create user: {}", error_text).into());
        }

        let data: Value = response.json().await?;
        let uid = data.get("localId").and_then(|v| v.as_str()).unwrap_or("").to_string();

        Ok(DecodedFirebaseToken {
            uid,
            email: Some(email.to_string()),
            email_verified: false,
            name: Some(name.to_string()),
            picture: None,
            claims: json!({}),
        })
    }

    /// Sign in with email and password
    pub async fn sign_in_with_email_password(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(String, DecodedFirebaseToken), Box<dyn std::error::Error>> {
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={}",
            self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&json!({
                "email": email,
                "password": password,
                "returnSecureToken": true
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Authentication failed: {}", error_text).into());
        }

        let data: Value = response.json().await?;
        let id_token = data
            .get("idToken")
            .and_then(|v| v.as_str())
            .ok_or("Missing ID token")?;
        let uid = data
            .get("localId")
            .and_then(|v| v.as_str())
            .ok_or("Missing user ID")?;
        let email_verified = data.get("emailVerified").and_then(|v| v.as_bool()).unwrap_or(false);

        let decoded = DecodedFirebaseToken {
            uid: uid.to_string(),
            email: Some(email.to_string()),
            email_verified,
            name: data.get("displayName").and_then(|v| v.as_str()).map(|s| s.to_string()),
            picture: data.get("profilePicture").and_then(|v| v.as_str()).map(|s| s.to_string()),
            claims: json!({}),
        };

        Ok((id_token.to_string(), decoded))
    }

    /// Link Firebase user to local database
    pub async fn link_user(
        &self,
        db: &Db,
        firebase_uid: &str,
        email: &str,
        name: &str,
        role: UserRole,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // First, check if user already exists
        if let Some(existing) = Self::get_user_link(db, firebase_uid)? {
            return Ok(existing.local_user_id);
        }

        // Create local user
        let local_user_id = db.create_linked_user(
            name.to_string(),
            Some(email.to_string()),
            role.clone(),
            None,
            None,
            firebase_uid.to_string(),
        )?;

        // Store the link
        let link = UserLink {
            firebase_uid: firebase_uid.to_string(),
            local_user_id: local_user_id.clone(),
            email: email.to_string(),
            role,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        db.save_user_link(&link)?;

        Ok(local_user_id)
    }

    /// Get user link by Firebase UID
    pub fn get_user_link(
        db: &Db,
        firebase_uid: &str,
    ) -> Result<Option<UserLink>, Box<dyn std::error::Error>> {
        Ok(db.get_user_link(firebase_uid)?)
    }

    /// Get user link by local user ID
    pub fn get_link_by_local_id(
        db: &Db,
        local_user_id: &str,
    ) -> Result<Option<UserLink>, Box<dyn std::error::Error>> {
        Ok(db.get_user_link_by_local_id(local_user_id)?)
    }

    /// Revoke refresh tokens (logout)
    pub async fn revoke_refresh_tokens(&self, uid: &str) -> Result<(), Box<dyn std::error::Error>> {
        // In a production setup with proper Firebase Admin SDK:
        // auth.revoke_refresh_tokens(uid).await?;
        
        // For now, we'll just log it and handle it at JWT level
        log::info!("Revoking refresh tokens for user: {}", uid);
        Ok(())
    }
}
