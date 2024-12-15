use lettre::SmtpTransport;
use sea_orm::DbConn;

#[derive(Clone)]
pub struct AppState {
    pub conn: DbConn,
    pub jwt_secret: String,
    pub 2fa_codes: HashMap<String, String>,
    pub smtp_transport: SmtpTransport,
    pub smtp_from_email: String,
}