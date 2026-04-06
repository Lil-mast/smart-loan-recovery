use crate::models::{User, UserRole};
use crate::db::Db;
use rusqlite::Result;
use rand::prelude::*;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const ID_LEN: usize = 4;

fn generate_id(db: &Db) -> Result<String> {
    let mut rng = rand::thread_rng();
    for _ in 0..10 { // Max 10 retries
        let id: String = (0..ID_LEN)
            .map(|_| {
                let idx = (rng.next_u32() as usize) % CHARSET.len();
                CHARSET[idx] as char
            })
            .collect();
        
        if db.load_user(&id)?.is_none() {
            return Ok(id);
        }
    }
    Err(rusqlite::Error::InvalidQuery)
}

pub struct UserManager<'a> {
    db: &'a Db,
}

impl<'a> UserManager<'a> {
    pub fn new(db: &'a Db) -> Self {
        UserManager { db }
    }

    pub fn register_user(&self, name: String, email: Option<String>, role: UserRole, lender_id: Option<String>, organization: Option<String>) -> Result<String> {
        let id = generate_id(self.db)?;
        let user = User {
            id: id.clone(),
            name,
            role,
            email,
            lender_id,
            organization,
        };
        self.db.save_user(&user)?;
        Ok(id)
    }

    pub fn get_user(&self, id: &str) -> Result<Option<User>> {
        self.db.load_user(id)
    }

    pub fn get_all_users(&self) -> Result<Vec<User>> {
        self.db.load_all_users()
    }
}

