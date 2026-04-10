//! JWT Authentication Middleware
//!
//! Verifies JWT tokens from Authorization header
//! Attaches user claims to request extensions

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::auth::models::FirebaseClaims;
use crate::auth::services::jwt::JwtService;
use crate::auth::services::TokenBlacklist;
use crate::auth::middleware::AuthContext;

/// JWT Authentication Middleware
#[derive(Clone)]
pub struct JwtAuth {
    jwt_service: Arc<JwtService>,
    token_blacklist: Arc<TokenBlacklist>,
    optional: bool, // If true, authentication is optional
}

impl JwtAuth {
    pub fn new(jwt_service: Arc<JwtService>, token_blacklist: Arc<TokenBlacklist>) -> Self {
        Self {
            jwt_service,
            token_blacklist,
            optional: false,
        }
    }

    /// Make authentication optional (for routes that work with/without auth)
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service,
            jwt_service: self.jwt_service.clone(),
            token_blacklist: self.token_blacklist.clone(),
            optional: self.optional,
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: S,
    jwt_service: Arc<JwtService>,
    token_blacklist: Arc<TokenBlacklist>,
    optional: bool,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
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
        let jwt_service = self.jwt_service.clone();
        let token_blacklist = self.token_blacklist.clone();
        let optional = self.optional;

        // Extract Authorization header
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .map(|h| h.to_string());

        let fut = self.service.call(req);

        Box::pin(async move {
            // Check if auth header exists
            let token = match auth_header {
                Some(header) => {
                    match JwtService::extract_token_from_header(&header) {
                        Some(t) => t.to_string(),
                        None => {
                            if optional {
                                return fut.await;
                            }
                            return Err(actix_web::error::ErrorUnauthorized(
                                "Invalid authorization header format. Use 'Bearer <token>'",
                            ));
                        }
                    }
                }
                None => {
                    if optional {
                        return fut.await;
                    }
                    return Err(actix_web::error::ErrorUnauthorized(
                        "Missing authorization header",
                    ));
                }
            };

            // Check if token is blacklisted (revoked)
            if token_blacklist.is_revoked(&token) {
                return Err(actix_web::error::ErrorUnauthorized(
                    "Token has been revoked. Please log in again.",
                ));
            }

            // Verify the token
            let claims = match jwt_service.verify_access_token(&token) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Token verification failed: {}", e);
                    return Err(actix_web::error::ErrorUnauthorized(
                        format!("Invalid token: {}", e),
                    ));
                }
            };

            // Attach claims to request extensions
            let auth_context = AuthContext::new(claims);
            let req = fut.await?;
            req.request().extensions_mut().insert(auth_context);

            Ok(req)
        })
    }
}

/// Helper function to extract auth context from request
pub fn get_auth_context(req: &actix_web::HttpRequest) -> Option<&AuthContext> {
    req.extensions().get::<AuthContext>()
}

/// Require authentication and return auth context or error
pub fn require_auth(req: &actix_web::HttpRequest) -> Result<&AuthContext, actix_web::Error> {
    get_auth_context(req).ok_or_else(|| {
        actix_web::error::ErrorUnauthorized("Authentication required")
    })
}
