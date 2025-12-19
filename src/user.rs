use crate::models::{User, UserRole};
use crate::db::Db;
use uuid::Uuid;
use rusqlite::Result;

pub struct UserManager<'a> {
    db: &'a Db,
}

impl<'a> UserManager<'a> {
    pub fn new(db: &'a Db) -> Self {
        UserManager { db }
    }

    pub fn register_user(&self, name: String, role: UserRole) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let user = User { id, name, role };
        self.db.save_user(&user)?;
        Ok(id)
    }

    pub fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        self.db.load_user(id)
    }

    pub fn get_all_users(&self) -> Result<Vec<User>> {
        self.db.load_all_users()
    }
}