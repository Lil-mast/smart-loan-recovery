//! Role-Based Access Control (RBAC) Middleware
//!
//! Enforces role-based permissions on API endpoints
//! - Admin: Full access
//! - Lender: Can view all loans, create loans
//! - Borrower: Can only view own loans

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::auth::middleware::AuthContext;
use crate::models::UserRole;

/// Role-based access control middleware
pub struct RequireRole {
    allowed_roles: Vec<UserRole>,
    require_all: bool, // If true, user must have ALL roles (for composite permissions)
}

impl RequireRole {
    /// Create middleware requiring any of the specified roles
    pub fn any_of(roles: Vec<UserRole>) -> Self {
        Self {
            allowed_roles: roles,
            require_all: false,
        }
    }

    /// Create middleware requiring a specific role
    pub fn role(role: UserRole) -> Self {
        Self {
            allowed_roles: vec![role],
            require_all: false,
        }
    }

    /// Require Admin role
    pub fn admin() -> Self {
        Self::role(UserRole::Admin)
    }

    /// Require Lender role (includes Admin)
    pub fn lender() -> Self {
        Self::any_of(vec![UserRole::Lender, UserRole::Admin])
    }

    /// Require Borrower role (includes Lender and Admin)
    pub fn borrower() -> Self {
        Self::any_of(vec![UserRole::Borrower, UserRole::Lender, UserRole::Admin])
    }

    /// Check if user has required role
    fn check_role(&self, user_role: &UserRole) -> bool {
        if self.require_all {
            // All roles must match (not typically used for simple RBAC)
            self.allowed_roles.iter().all(|r| r == user_role)
        } else {
            // Any role can match
            self.allowed_roles.iter().any(|allowed| {
                matches!(
                    (allowed, user_role),
                    // Admin can do everything
                    (UserRole::Admin, UserRole::Admin)
                        // Lender can do lender + admin things
                        | (UserRole::Lender, UserRole::Lender)
                        | (UserRole::Lender, UserRole::Admin)
                        // Borrower can do borrower things
                        | (UserRole::Borrower, UserRole::Borrower)
                        | (UserRole::Borrower, UserRole::Lender)
                        | (UserRole::Borrower, UserRole::Admin)
                )
            })
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequireRole
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RbacMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RbacMiddleware {
            service,
            allowed_roles: self.allowed_roles.clone(),
            require_all: self.require_all,
        }))
    }
}

pub struct RbacMiddleware<S> {
    service: S,
    allowed_roles: Vec<UserRole>,
    require_all: bool,
}

impl<S, B> Service<ServiceRequest> for RbacMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let allowed_roles = self.allowed_roles.clone();
        let require_all = self.require_all;

        // Check if user is authenticated and has required role
        let auth_context = req.extensions().get::<AuthContext>().cloned();

        let fut = self.service.call(req);

        Box::pin(async move {
            let auth_context = match auth_context {
                Some(ctx) => ctx,
                None => {
                    return Err(actix_web::error::ErrorUnauthorized(
                        "Authentication required for this endpoint",
                    ));
                }
            };

            let user_role = auth_context.role();

            // Check role permissions
            let has_permission = if require_all {
                allowed_roles.iter().all(|r| r == user_role)
            } else {
                allowed_roles.iter().any(|allowed| {
                    matches!(
                        (allowed, user_role),
                        (UserRole::Admin, UserRole::Admin)
                            | (UserRole::Lender, UserRole::Lender)
                            | (UserRole::Lender, UserRole::Admin)
                            | (UserRole::Borrower, UserRole::Borrower)
                            | (UserRole::Borrower, UserRole::Lender)
                            | (UserRole::Borrower, UserRole::Admin)
                    )
                })
            };

            if !has_permission {
                log::warn!(
                    "Access denied: user with role {:?} tried to access resource requiring {:?}",
                    user_role,
                    allowed_roles
                );
                return Err(actix_web::error::ErrorForbidden(
                    format!(
                        "Access denied. Required role: {:?}, your role: {:?}",
                        allowed_roles, user_role
                    ),
                ));
            }

            // User has permission, proceed
            fut.await
        })
    }
}

/// Resource ownership check middleware
/// Ensures users can only access their own resources (with admin override)
pub struct ResourceOwnership;

impl ResourceOwnership {
    /// Check if user can access a resource
    pub fn can_access(
        auth_context: &AuthContext,
        resource_owner_id: &str,
    ) -> Result<(), actix_web::Error> {
        // Admin can access everything
        if auth_context.is_admin() {
            return Ok(());
        }

        // User can access their own resources
        if let Some(user_id) = auth_context.local_user_id() {
            if user_id == resource_owner_id {
                return Ok(());
            }
        }

        // Check if Firebase UID matches (for newly created users)
        if auth_context.user_id() == resource_owner_id {
            return Ok(());
        }

        Err(actix_web::error::ErrorForbidden(
            "You can only access your own resources",
        ))
    }
}

/// Macro-like helper for role checks
#[macro_export]
macro_rules! require_role {
    ($req:expr, $($role:path),+) => {
        {
            use $crate::auth::middleware::auth::get_auth_context;
            use $crate::models::UserRole;
            
            let ctx = get_auth_context($req)
                .ok_or(actix_web::error::ErrorUnauthorized("Authentication required"))?;
            
            let allowed = vec![$($role),+];
            let user_role = ctx.role();
            
            let has_permission = allowed.iter().any(|allowed_role| {
                matches!(
                    (allowed_role, user_role),
                    (UserRole::Admin, UserRole::Admin)
                        | (UserRole::Lender, UserRole::Lender)
                        | (UserRole::Lender, UserRole::Admin)
                        | (UserRole::Borrower, UserRole::Borrower)
                        | (UserRole::Borrower, UserRole::Lender)
                        | (UserRole::Borrower, UserRole::Admin)
                )
            });
            
            if !has_permission {
                return Err(actix_web::error::ErrorForbidden("Insufficient permissions"));
            }
            
            Ok(ctx)
        }
    };
}
