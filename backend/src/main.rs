use std::io;

use actix_web::{get, middleware, web::ServiceConfig, App, HttpResponse, HttpServer};

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt().init();
    
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(init)
    }).bind_auto_h2c("0.0.0.0:8888")?.run().await?;
    Ok(())
}

#[get("/")]
async fn status() -> HttpResponse {
    HttpResponse::Ok().body("ok.")
}

fn init(cfg: &mut ServiceConfig) {
    cfg.service(status);
}
