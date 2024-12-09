use crate::error::{ApiError, ApiErrorKind};
use crate::service::auth::{
    handle_login, handle_registration, AccountLoginForm, AccountRegistrationForm,
};
use crate::AppState;
use actix_web::{
    get, post,
    web::{Data, Json, ServiceConfig},
    HttpResponse, Result,
};
use serde_json::json;
use sys_info::{hostname, os_release, os_type};

#[get("/")]
async fn status(state: Data<AppState>) -> HttpResponse {
    // Пример данных, которые вы можете получать из базы данных.
    let registered_users = 120_000; // Подставьте реальную статистику
    let active_users = 15_000; // За последние 24 часа
    let unread_messages = 500_000;
    let total_chats = 45_000;
    let group_chats = 10_000;

    let uptime = state
        .start_time
        .elapsed()
        .map(|dur| format!("{}s", dur.as_secs()))
        .unwrap_or_else(|_| "Unknown".to_string());

    let hostname = hostname().unwrap_or_else(|_| "Unknown".to_string());
    let os_type = os_type().unwrap_or_else(|_| "Unknown".to_string());
    let os_release = os_release().unwrap_or_else(|_| "Unknown".to_string());

    HttpResponse::Ok().json(json!({
        "status": "Ok!",
        "version": env!("CARGO_PKG_VERSION"),
        "hostname": hostname,
        "os": format!("{} {}", os_type, os_release),
        "uptime": uptime,
        "registered_users": registered_users,
        "active_users": active_users,
        "unread_messages": unread_messages,
        "total_chats": total_chats,
        "group_chats": group_chats,
        "database_status": "Connected", // Проверьте соединение с базой данных
        "push_notifications_status": "Operational", // Проверьте статус push-сервиса
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

#[post("/auth/login")]
async fn login(
    state: Data<AppState>,
    form: Json<AccountLoginForm>,
) -> Result<HttpResponse, ApiError> {
    handle_login(&state.conn, &form)
        .await
        .map(|token| HttpResponse::Ok().json(json!({ "token": token })))
        .map_err(|_| ApiError::new(ApiErrorKind::Unauthorized))
}

#[post("/auth/register")]
async fn register(
    state: Data<AppState>,
    form: Json<AccountRegistrationForm>,
) -> Result<HttpResponse, ApiError> {
    handle_registration(&state.conn, &form)
        .await
        .map(|_| HttpResponse::Created().json("User registered successfully"))
        .map_err(|err| {
            println!("{err}");
            return ApiError::new(ApiErrorKind::InternalServerError);
        })
}

pub fn init(cfg: &mut ServiceConfig) {
    cfg.service(status).service(login).service(register);
}
