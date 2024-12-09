use std::time::SystemTime;

use sea_orm::DbConn;

#[derive(Clone)]
pub struct AppState {
    pub conn: DbConn,
    pub start_time: SystemTime,
}
