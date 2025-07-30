use axum::Router;
use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tokio::time::{sleep, Duration};
use diesel::r2d2;
use diesel::SqliteConnection;

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

// Background task untuk cleanup expired tokens
async fn token_cleanup_task(pool: r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>) {
    loop {
        // Jalankan setiap 24 jam
        sleep(Duration::from_secs(24 * 60 * 60)).await;
        
        let cutoff_date = chrono::Utc::now().naive_utc() - chrono::Duration::days(7);
        
        match pool.get() {
            Ok(mut conn) => {
                match db::token_blacklist_query::cleanup_expired_tokens(&mut conn, cutoff_date) {
                    Ok(deleted_count) => {
                        println!("✅ Cleaned up {} expired tokens", deleted_count);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to cleanup expired tokens: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to get DB connection for cleanup: {}", e);
            }
        }
    }
}

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

    // Clone pool untuk background task
    let cleanup_pool = pool.clone();
    
    // Jalankan background task untuk cleanup
    tokio::spawn(async move {
        token_cleanup_task(cleanup_pool).await;
    });

    // Create API routes dengan prefix /api
    let api_routes = Router::new()
        .merge(path::init_routes())
        .with_state(pool);

    // Create the main app dengan prefix /api
    let app = Router::new()
        .nest("/api", api_routes)  
        .layer(CorsLayer::permissive());

    // Define address and port to run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("🚀 Server listening on http://{}", addr);
    println!("📡 All routes available at http://{}/api/...", addr);
    println!("🧹 Token cleanup task started (runs every 24 hours)");

    // Run the Axum server
    axum::serve(tokio::net::TcpListener::bind(&addr).await.unwrap(), app)
        .await
        .expect("Server failed to start");
}