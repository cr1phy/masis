use actix_web::{get, post, rt, web::{self, ServiceConfig}, Error, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
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

#[get("/acc/{token}")]
async fn user_events(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        while let Some(msg) = stream.recv().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    session.text(text).await.unwrap();
                }

                Ok(AggregatedMessage::Binary(bin)) => {
                    session.binary(bin).await.unwrap();
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }

                _ => {}
            }
        }
    });

    Ok(res)
}

pub fn init(cfg: &mut ServiceConfig) {
    cfg.service(status);
}