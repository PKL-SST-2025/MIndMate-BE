use axum::Router;
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
mod schema;
mod middleware;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize logger (make sure RUST_LOG is set, e.g. to "debug")
    env_logger::init();

    // Get the database URL from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create the database connection pool
    let pool = db::pool::create_pool(database_url);

    // Create the app with routing and middleware
    let app = Router::new()
        .merge(path::init_routes())
        .layer(CorsLayer::permissive())
        .with_state(pool);

    // Define address and port to run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("🚀 Server listening on http://{}", addr);

    // Run the Axum server
    axum::serve(tokio::net::TcpListener::bind(&addr).await.unwrap(), app)
        .await
        .expect("Server failed to start");
}