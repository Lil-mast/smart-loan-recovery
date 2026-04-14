//! Authentication Middleware
//!
//! Actix-web middleware for:
//! - JWT verification
//! - Role-based access control (RBAC)
//! - Token validation

pub mod auth;
pub mod rbac;

use crate::auth::models::FirebaseClaims;
use crate::models::UserRole;

/// Request extension to store authenticated user claims
#[derive(Clone)]
pub struct AuthContext {
    pub claims: FirebaseClaims,
    pub role: UserRole,
}

impl AuthContext {
    pub fn new(claims: FirebaseClaims) -> Self {
        let role = claims.role.clone().unwrap_or(UserRole::Borrower);
        Self { claims, role }
    }

    pub fn user_id(&self) -> &str {
        &self.claims.sub
    }

    pub fn local_user_id(&self) -> Option<&str> {
        self.claims.local_user_id.as_deref()
    }

    pub fn email(&self) -> Option<&str> {
        self.claims.email.as_deref()
    }

    pub fn role(&self) -> &UserRole {
        &self.role
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin)
    }

    pub fn is_lender(&self) -> bool {
        matches!(self.role, UserRole::Lender | UserRole::Admin)
    }

    pub fn is_borrower(&self) -> bool {
        matches!(self.role, UserRole::Borrower | UserRole::Lender | UserRole::Admin)
    }
}
