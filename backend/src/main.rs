mod error;
mod handlers;
mod service;
mod state;
mod types;

use actix_web::{middleware, web::Data, App, HttpServer};
use listenfd::ListenFd;
use migration::{Migrator, MigratorTrait};
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

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(Data::new(state.clone()))
            .configure(handlers::init)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind_auto_h2c(&server_url)?,
    };

    log::info!("Starting server at {server_url}");
    server.run().await?;

    Ok(())
}
