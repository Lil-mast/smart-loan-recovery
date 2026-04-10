//! Authentication Services
//!
//! Core services for Firebase authentication and JWT handling

pub mod firebase;
pub mod jwt;

use std::collections::HashSet;
use std::sync::Mutex;

/// Token blacklist for revoked tokens (logout)
pub struct TokenBlacklist {
    revoked_tokens: Mutex<HashSet<String>>,
}

impl TokenBlacklist {
    pub fn new() -> Self {
        Self {
            revoked_tokens: Mutex::new(HashSet::new()),
        }
    }

    /// Add a token to the blacklist
    pub fn revoke_token(&self, token: &str) {
        let mut tokens = self.revoked_tokens.lock().unwrap();
        tokens.insert(token.to_string());
    }

    /// Check if a token is revoked
    pub fn is_revoked(&self, token: &str) -> bool {
        let tokens = self.revoked_tokens.lock().unwrap();
        tokens.contains(token)
    }
}
