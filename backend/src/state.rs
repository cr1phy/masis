use sea_orm::DbConn;

#[derive(Clone)]
pub struct AppState {
    pub conn: DbConn,
    pub jwt_secret: String,
}
