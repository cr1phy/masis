use actix_web::{get, post, web::ServiceConfig, HttpResponse};
use serde_json::json;

#[get("/")]
async fn status() -> HttpResponse {
    HttpResponse::Ok().json(json! {{"status": "Ok!", "version": env!("CARGO_PKG_VERSION")}})
}

#[post("/auth/login")]
async fn login() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[post("/auth/logout")]
async fn logout() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn init(cfg: &mut ServiceConfig) {
    cfg.service(status);
}
