use axum::{Router, Server};
use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

mod api;
mod service;
mod models;
mod db;
mod path;
mod config;
mod errors;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::pool::create_pool(database_url);

    let app = Router::new()
        .merge(path::init_routes())
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server failed to start");
}