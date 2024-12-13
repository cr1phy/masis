use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Account {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password: Vec<u8>,
    pub date_of_registration: DateTime<Utc>,
    pub time_of_last_online: Option<DateTime<Utc>>,
}
