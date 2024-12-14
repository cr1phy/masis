use crate::{
    error::{ApiError, ApiErrorKind},
    service::session::create_session,
    state::AppState,
};
use ::entity::prelude::Account;
use actix_web::{
    get, post,
    web::{Data, Json, ServiceConfig},
    HttpRequest, HttpResponse,
};
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use entity::account;
use log::{error, info};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use woothee::parser::Parser;

#[get("/v1")]
async fn status() -> HttpResponse {
    HttpResponse::Ok().json(json!({"status": "Ok!", "version": env!("CARGO_PKG_VERSION")}))
}

#[derive(Debug, Deserialize)]
pub struct RegistrationForm {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[post("/v1/auth/registration")]
async fn registration(
    req: HttpRequest,
    state: Data<AppState>,
    form: Json<RegistrationForm>,
) -> Result<HttpResponse, ApiError> {
    let db = &state.conn;

    // Проверяем email
    if Account::find()
        .filter(account::Column::Email.eq(&form.email))
        .one(db)
        .await
        .map_err(|_| ApiError::new(ApiErrorKind::InternalServerError))?
        .is_some()
    {
        return Err(ApiError::new(ApiErrorKind::EmailAlreadyInUse));
    }

    // Проверяем username
    if Account::find()
        .filter(account::Column::Username.eq(&form.username))
        .one(db)
        .await
        .map_err(|e| ApiError::new(ApiErrorKind::InternalServerError))?
        .is_some()
    {
        return Err(ApiError::new(ApiErrorKind::UsernameAlreadyInUse));
    }

    let hashed_password = hash(&form.password, DEFAULT_COST)
        .map_err(|_| ApiError::new(ApiErrorKind::InternalServerError))?;

    let new_account = account::ActiveModel {
        id: Set(Uuid::now_v7()),
        username: Set(form.username.clone()),
        email: Set(form.email.clone()),
        password: Set(hashed_password.into_bytes()),
        date_of_registration: Set(Utc::now().naive_utc()),
        time_of_last_online: Set(Utc::now().naive_utc()),
    };

    let account = Account::insert(new_account)
        .exec(db)
        .await
        .map_err(|_| ApiError::new(ApiErrorKind::InternalServerError))?;

    let device_name = req
        .headers()
        .get("User-Agent")
        .and_then(|header| header.to_str().ok())
        .unwrap_or("Unknown Device")
        .to_string();

    let client_ip = req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or("127.0.0.1".to_string());

    let token = create_session(
        account.last_insert_id,
        device_name,
        client_ip,
        &state.jwt_secret,
        db,
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "Ok!",
        "token": token,
    })))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
struct DeviceInfo {
    device_name: String,
    os_name: String,
    os_version: Option<String>,
    os_family: String,
}

#[post("/v1/auth/login")]
async fn login(
    req: HttpRequest,
    state: Data<AppState>,
    form: Json<LoginForm>,
) -> Result<HttpResponse, ApiError> {
    let db = &state.conn;

    // Проверка email в базе данных
    let user = Account::find()
        .filter(account::Column::Email.eq(&form.email))
        .one(db)
        .await
        .map_err(|e| {
            error!("Ошибка при проверке email: {:?}", e);
            ApiError::new(ApiErrorKind::InternalServerError)
        })?
        .ok_or_else(|| {
            info!("Неверный email при попытке входа: {}", form.email);
            ApiError::new(ApiErrorKind::InvalidCredentials)
        })?;

    // Сравнение пароля
    let password_valid = bcrypt::verify(&form.password, &String::from_utf8_lossy(&user.password))
        .map_err(|e| {
        error!("Ошибка при верификации пароля: {:?}", e);
        ApiError::new(ApiErrorKind::InternalServerError)
    })?;

    if !password_valid {
        info!("Неверный пароль для email: {}", form.email);
        return Err(ApiError::new(ApiErrorKind::InvalidCredentials));
    }

    // Извлекаем информацию о пользователе
    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Парсим User-Agent с помощью woothee
    let device_info = Parser::new().parse(user_agent).unwrap_or_default();

    // Логируем результат
    info!(
        "Успешный вход для email: {}, устройство: {:?}, IP: {}",
        form.email,
        &device_info,
        req.peer_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_default()
    );

    // Возвращаем ответ с информацией
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "Ok!",
        "device": device_info.vendor,
        "os_name": device_info.os,
        "os_version": device_info.os_version,
        "os_family": device_info.category,
    })))
}

#[post("/v1/auth/logout")]
async fn logout() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn init(cfg: &mut ServiceConfig) {
    cfg.service(status)
        .service(registration)
        .service(login)
        .service(logout);
}
