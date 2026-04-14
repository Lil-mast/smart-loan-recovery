use actix_web::HttpRequest;
use actix_web::cookie::{Cookie, SameSite, time::Duration};

const ACCESS_TOKEN_COOKIE: &str = "access_token";
const REFRESH_TOKEN_COOKIE: &str = "refresh_token";

/// Build authentication cookies (httpOnly, Secure, SameSite=Strict)
pub fn build_auth_cookies(
    access_token: String,
    refresh_token: String,
    access_token_expiry_secs: i64,
    refresh_token_expiry_secs: i64,
) -> (Cookie<'static>, Cookie<'static>) {
    let is_secure = is_secure_context();
    
    // Access token cookie
    let access_cookie = Cookie::build(ACCESS_TOKEN_COOKIE, access_token)
        .http_only(true)
        .secure(is_secure)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(Duration::seconds(access_token_expiry_secs))
        .finish();
    
    // Refresh token cookie
    let refresh_cookie = Cookie::build(REFRESH_TOKEN_COOKIE, refresh_token)
        .http_only(true)
        .secure(is_secure)
        .same_site(SameSite::Strict)
        .path("/auth/refresh") // Only sent to refresh endpoint
        .max_age(Duration::seconds(refresh_token_expiry_secs))
        .finish();
    
    (access_cookie, refresh_cookie)
}

/// Build logout cookies (expire immediately)
pub fn build_logout_cookies() -> (Cookie<'static>, Cookie<'static>) {
    let access_cookie = Cookie::build(ACCESS_TOKEN_COOKIE, "")
        .http_only(true)
        .path("/")
        .max_age(Duration::seconds(0))
        .finish();
    
    let refresh_cookie = Cookie::build(REFRESH_TOKEN_COOKIE, "")
        .http_only(true)
        .path("/")
        .max_age(Duration::seconds(0))
        .finish();
    
    (access_cookie, refresh_cookie)
}

/// Extract access token from cookie
#[allow(dead_code)]
pub fn extract_access_token_from_cookie(req: &HttpRequest) -> Option<String> {
    req.cookie(ACCESS_TOKEN_COOKIE)
        .map(|c| c.value().to_string())
}

/// Extract refresh token from cookie
#[allow(dead_code)]
pub fn extract_refresh_token_from_cookie(req: &HttpRequest) -> Option<String> {
    req.cookie(REFRESH_TOKEN_COOKIE)
        .map(|c| c.value().to_string())
}

/// Check if we should use secure cookies (HTTPS)
fn is_secure_context() -> bool {
    // In production, always use secure cookies
    // For local development, allow non-secure
    std::env::var("RUST_ENV").map(|v| v == "production").unwrap_or(false)
}
