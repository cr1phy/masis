use crate::{
    error::{ApiError, ApiErrorKind},
    state::AppState,
};
use ::entity::prelude::Account;
use actix_web::{
    get, post,
    web::{Data, Json, ServiceConfig},
    HttpResponse,
};
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use entity::account;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

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
    state: Data<AppState>,
    form: Json<RegistrationForm>,
) -> Result<HttpResponse, ApiError> {
    let db = &state.conn;

    if Account::find()
        .filter(account::Column::Email.eq(&form.email))
        .one(db)
        .await
        .unwrap()
        .is_some()
    {
        return Err(ApiError::new(ApiErrorKind::EmailAlreadyInUse));
    }

    if Account::find()
        .filter(account::Column::Username.eq(&form.username))
        .one(db)
        .await
        .unwrap()
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

    Account::insert(new_account)
        .exec(db)
        .await
        .map(|user| HttpResponse::Created().json(user.last_insert_id))
        .map_err(|_| ApiError::new(ApiErrorKind::InternalServerError))
}

#[post("/v1/auth/login")]
async fn login() -> HttpResponse {
    HttpResponse::Ok().finish()
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
