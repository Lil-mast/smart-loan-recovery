//! JWT Service
//!
//! Handles JWT token generation, verification, and refresh
//! - Stateless authentication using JWT
//! - Role-based claims
//! - Token expiration and refresh logic

use crate::auth::models::FirebaseClaims;
use crate::models::UserRole;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// JWT Service for token operations
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    jwt_secret: String,
    access_token_expiry_hours: i64,
    refresh_token_expiry_days: i64,
}

/// Refresh token data
#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenData {
    pub uid: String,
    pub local_user_id: String,
    pub exp: i64,
    pub jti: String, // JWT ID for revocation
}

impl JwtService {
    /// Initialize JWT service from environment variables
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET not set in .env.firebase")?;

        let access_token_expiry_hours = std::env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<i64>()?;

        let refresh_token_expiry_days = std::env::var("REFRESH_TOKEN_EXPIRATION_DAYS")
            .unwrap_or_else(|_| "7".to_string())
            .parse::<i64>()?;

        let encoding_key = EncodingKey::from_secret(jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());

        Ok(Self {
            encoding_key,
            decoding_key,
            jwt_secret,
            access_token_expiry_hours,
            refresh_token_expiry_days,
        })
    }

    /// Generate access token for authenticated user
    pub fn generate_access_token(
        &self,
        uid: &str,
        email: &str,
        email_verified: bool,
        role: UserRole,
        local_user_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.access_token_expiry_hours);

        let claims = FirebaseClaims {
            sub: uid.to_string(),
            email: Some(email.to_string()),
            email_verified,
            iss: "lendwise-recovery".to_string(),
            aud: "lendwise-api".to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            role: Some(role),
            local_user_id: Some(local_user_id.to_string()),
        };

        let token = encode(&Header::new(Algorithm::HS256), &claims, &self.encoding_key)?;
        Ok(token)
    }

    /// Generate refresh token
    pub fn generate_refresh_token(
        &self,
        uid: &str,
        local_user_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let now = Utc::now();
        let exp = now + Duration::days(self.refresh_token_expiry_days);

        // Generate unique JWT ID for revocation support
        let jti = format!("{}-{}", uid, generate_random_string(16));

        let claims = RefreshTokenData {
            uid: uid.to_string(),
            local_user_id: local_user_id.to_string(),
            exp: exp.timestamp(),
            jti,
        };

        let token = encode(&Header::new(Algorithm::HS256), &claims, &self.encoding_key)?;
        Ok(token)
    }

    /// Verify and decode access token
    pub fn verify_access_token(&self, token: &str) -> Result<FirebaseClaims, Box<dyn std::error::Error>> {
        let validation = Validation::new(Algorithm::HS256);

        let decoded = decode::<FirebaseClaims>(token, &self.decoding_key, &validation)?;

        // Check expiration
        let now = Utc::now().timestamp();
        if decoded.claims.exp < now {
            return Err("Token expired".into());
        }

        Ok(decoded.claims)
    }

    /// Verify refresh token and return the data
    pub fn verify_refresh_token(&self, token: &str) -> Result<(String, String, String), Box<dyn std::error::Error>> {
        let validation = Validation::new(Algorithm::HS256);

        let decoded = decode::<RefreshTokenData>(token, &self.decoding_key, &validation)?;

        // Check expiration
        let now = Utc::now().timestamp();
        if decoded.claims.exp < now {
            return Err("Refresh token expired".into());
        }

        Ok((
            decoded.claims.uid,
            decoded.claims.local_user_id,
            decoded.claims.jti,
        ))
    }

    /// Generate token pair (access + refresh)
    pub fn generate_token_pair(
        &self,
        uid: &str,
        email: &str,
        email_verified: bool,
        role: UserRole,
        local_user_id: &str,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let access_token = self.generate_access_token(uid, email, email_verified, role.clone(), local_user_id)?;
        let refresh_token = self.generate_refresh_token(uid, local_user_id)?;

        Ok((access_token, refresh_token))
    }

    /// Extract token from Authorization header
    pub fn extract_token_from_header(auth_header: &str) -> Option<&str> {
        if auth_header.starts_with("Bearer ") {
            Some(&auth_header[7..])
        } else {
            None
        }
    }

    /// Get token expiration time
    pub fn get_token_expiry(&self) -> i64 {
        self.access_token_expiry_hours * 3600 // Convert to seconds
    }
}

/// Generate random string for JWT ID
fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_lifecycle() {
        std::env::set_var("JWT_SECRET", "test-secret-key-for-unit-tests-only");
        std::env::set_var("JWT_EXPIRATION_HOURS", "24");
        std::env::set_var("REFRESH_TOKEN_EXPIRATION_DAYS", "7");

        let jwt_service = JwtService::new().unwrap();

        let (access_token, refresh_token) = jwt_service
            .generate_token_pair(
                "test-uid",
                "test@example.com",
                true,
                UserRole::Borrower,
                "local-123",
            )
            .unwrap();

        // Verify access token
        let claims = jwt_service.verify_access_token(&access_token).unwrap();
        assert_eq!(claims.sub, "test-uid");
        assert_eq!(claims.email, Some("test@example.com".to_string()));

        // Verify refresh token
        let (uid, local_id, _jti) = jwt_service.verify_refresh_token(&refresh_token).unwrap();
        assert_eq!(uid, "test-uid");
        assert_eq!(local_id, "local-123");
    }

    #[test]
    fn test_extract_token_from_header() {
        assert_eq!(
            JwtService::extract_token_from_header("Bearer abc123"),
            Some("abc123")
        );
        assert_eq!(
            JwtService::extract_token_from_header("Basic abc123"),
            None
        );
    }
}
