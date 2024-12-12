use actix_web::{get, post, web::ServiceConfig, HttpResponse};

#[get("/v1")]
async fn status() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[post("/v1/auth/registration")]
async fn registration() -> HttpResponse {
    HttpResponse::Ok().finish()
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
