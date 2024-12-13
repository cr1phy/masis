mod error;
mod handlers;
mod state;
mod types;

use actix_cors::Cors;
use actix_web::{http, middleware, web::Data, App, HttpServer};
use listenfd::ListenFd;
use migration::{Migrator, MigratorTrait};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use sea_orm::Database;
use state::AppState;
use std::{env, io, time::SystemTime};

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt().init();

    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    let conn = Database::connect(&db_url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();

    let state = AppState {
        conn,
        start_time: SystemTime::now(),
    };

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://www.masis.ru")
            .allowed_origin_fn(|origin, _req_head| origin.as_bytes().ends_with(b".masis.ru"))
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(cors)
            .app_data(Data::new(state.clone()))
            .configure(handlers::init)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind_openssl(&server_url, builder)?,
    };

    log::info!("Starting server at {server_url}");
    server.run().await?;

    Ok(())
}
