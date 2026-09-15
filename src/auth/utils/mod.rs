//! Authentication Utilities
//!
//! Helper functions for authentication operations

use actix_web::HttpRequest;

/// Extract client IP address from request
pub fn extract_client_ip(req: &HttpRequest) -> Option<String> {
    // Check X-Forwarded-For header (for proxied requests)
    if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP if multiple are present
            return forwarded_str.split(',').next().map(|s| s.trim().to_string());
        }
    }

    // Check X-Real-IP header
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return Some(ip_str.to_string());
        }
    }

    // Use peer address (direct connection)
    req.peer_addr().map(|addr| addr.ip().to_string())
}

/// Extract user agent from request
pub fn extract_user_agent(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("User-Agent")
        .and_then(|ua| ua.to_str().ok())
        .map(|s| s.to_string())
}

/// Sanitize email address
pub fn sanitize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Validate password strength
pub fn validate_password_strength(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long");
    }
    Ok(())
}

pub fn firebase_auth_error_message(raw: &str) -> String {
    let upper = raw.to_ascii_uppercase();
    if upper.contains("CONFIGURATION_NOT_FOUND") || upper.contains("OPERATION_NOT_ALLOWED") {
        return "Firebase Authentication is not set up. In Firebase Console open Authentication, click Get started, then enable Email/Password and Google. Also enable the Identity Toolkit API for this Google Cloud project.".to_string();
    }
    if upper.contains("EMAIL_EXISTS") {
        return "That email is already registered. Sign in instead.".to_string();
    }
    if upper.contains("INVALID_PASSWORD") || upper.contains("INVALID_LOGIN_CREDENTIALS") {
        return "Invalid email or password.".to_string();
    }
    if upper.contains("EMAIL_NOT_FOUND") {
        return "No account with that email.".to_string();
    }
    if upper.contains("WEAK_PASSWORD") {
        return "Password is too weak. Use at least 8 characters.".to_string();
    }
    "Authentication failed. Check Firebase Auth and try again.".to_string()
}
