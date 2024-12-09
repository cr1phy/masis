use actix_web::{
    get, post,
    web::{Data, Json, ServiceConfig},
    HttpResponse, Result,
};
use serde_json::json;
use crate::error::{ApiError, ApiErrorKind};
use crate::service::auth::{handle_login, handle_registration, AccountLoginForm, AccountRegistrationForm};
use crate::AppState;

#[get("/")]
async fn status() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "Ok!",
        "version": env!("CARGO_PKG_VERSION"),
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
        .map_err(|err| ApiError::new(ApiErrorKind::InternalServerError))
}

pub fn init(cfg: &mut ServiceConfig) {
    cfg.service(status)
        .service(login)
        .service(register);
}

