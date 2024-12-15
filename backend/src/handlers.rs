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
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use entity::account;
use lettre::{Message, Transport};
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
    pub two_factor_code: Option<String>, // Опциональный код 2FA
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
            log::error!("Ошибка базы данных при поиске пользователя: {:?}", e);
            ApiError::new(ApiErrorKind::InternalServerError)
        })?
        .ok_or_else(|| {
            log::info!("Неверный email при попытке входа: {}", form.email);
            ApiError::new(ApiErrorKind::InvalidCredentials)
        })?;

    // Проверка пароля
    let password_valid = verify(&form.password, &String::from_utf8_lossy(&user.password))
        .map_err(|e| {
            log::error!("Ошибка при верификации пароля: {:?}", e);
            ApiError::new(ApiErrorKind::InternalServerError)
        })?;

    if !password_valid {
        log::info!("Неверный пароль для email: {}", form.email);
        return Err(ApiError::new(ApiErrorKind::InvalidCredentials));
    }

    // Если 2FA код указан
    if let Some(provided_code) = &form.two_factor_code {
        let stored_code = state.conn.get_temp_2fa_code(user.id).await?;
        if provided_code != stored_code {
            log::info!("Неверный 2FA код для email: {}", form.email);
            return Err(ApiError::new(ApiErrorKind::InvalidSession)); // Код неверный
        }
        state.conn.clear_temp_2fa_code(user.id).await?;
        log::info!("Успешная аутентификация с 2FA для email: {}", form.email);

        // Продолжим, создадим токен и вернем ответ
        let token = generate_token_and_session(user.id, &state, req).await?;
        return Ok(HttpResponse::Ok().json(serde_json::json!({ "token": token })));
    }

    // Генерация и отправка 2FA кода
    let two_factor_code: String = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    state.conn.set_temp_2fa_code(user.id, &two_factor_code).await?;

    send_2fa_email(&state, &user.email, &two_factor_code).await.map_err(|e| {
        log::error!("Ошибка при отправке email с 2FA: {:?}", e);
        ApiError::new(ApiErrorKind::InternalServerError)
    })?;

    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "status": "2FA code sent to email"
    })))
}

// Отправка email с кодом 2FA
async fn send_2fa_email(
    state: &Data<AppState>,
    email: &str,
    code: &str,
) -> Result<(), lettre::address::AddressError> {
    let email = Message::builder()
        .from(state.smtp_from_email.parse()?)
        .to(email.parse()?)
        .subject("Ваш код для входа")
        .body(format!("Ваш код для двухфакторной аутентификации: {}", code))?;

    state.smtp_transport.send(&email).map_err(|e| {
        log::error!("Ошибка отправки письма: {:?}", e);
        e
    })?;

    Ok(())
}

// Функция для генерации токена и сессии
async fn generate_token_and_session(
    user_id: Uuid,
    state: &Data<AppState>,
    req: HttpRequest,
) -> Result<String, ApiError> {
    let device_info = extract_device_info(&req);
    let ip = req.peer_addr().map(|addr| addr.ip().to_string()).unwrap_or_default();
    create_session(user_id, device_info, ip, &state.jwt_secret, &state.conn).await
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
