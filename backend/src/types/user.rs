use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Account {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password: String,
    pub date_of_registration: DateTime<Utc>,
    pub time_of_last_online: Option<DateTime<Utc>>,
}

impl Account {
    pub fn new(username: &str, email: &str, password: &str) -> Self {
        let password_hash = hash(password, DEFAULT_COST).expect("Failed to hash password");
        Account {
            id: Uuid::now_v7(),
            username: username.to_string(),
            email: email.to_string(),
            password: password_hash,
            date_of_registration: Utc::now(),
            time_of_last_online: None,
        }
    }

    pub fn verify_password(&self, password: &str) -> bool {
        verify(password, &self.password).unwrap_or(false)
    }

    pub fn update_last_online(&mut self) {
        self.time_of_last_online = Some(Utc::now());
    }
}
