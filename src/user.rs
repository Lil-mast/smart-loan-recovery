use crate::models::{User, UserRole};
use uuid::Uuid;

pub struct UserManager {
    // In-memory for now; persist later
    users: Vec<User>,
}

impl UserManager {
    pub fn new() -> Self {
        UserManager { users: Vec::new() }
    }

    pub fn register_user(&mut self, name: String, role: UserRole) -> Result<Uuid, String> {
        let id = Uuid::new_v4();
        let user = User { id, name, role };
        self.users.push(user);
        Ok(id)
    }

    pub fn get_user(&self, id: Uuid) -> Option<&User> {
        self.users.iter().find(|u| u.id == id)
    }

    // Add loan association logic later when integrating with loans
}