//! Authentication Utilities
//!
//! Helper functions for authentication operations

#![allow(dead_code)]

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

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase {
        return Err("Password must contain at least one uppercase letter");
    }
    if !has_lowercase {
        return Err("Password must contain at least one lowercase letter");
    }
    if !has_digit {
        return Err("Password must contain at least one digit");
    }
    if !has_special {
        return Err("Password must contain at least one special character");
    }

    Ok(())
}
