//! Authentication Handlers
//!
//! HTTP handlers for authentication endpoints:
//! - POST /auth/login - Email/password login
//! - POST /auth/register - User registration
//! - POST /auth/logout - Logout and revoke tokens
//! - POST /auth/refresh - Refresh access token
//! - POST /auth/verify - Verify JWT token validity
//! - GET /auth/me - Get current user profile
//! - PUT /auth/me - Update user profile
//! - POST /auth/google - Google Sign-In

pub mod auth;
pub mod google;
pub mod cookie;
