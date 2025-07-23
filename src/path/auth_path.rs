use axum::Router;
use diesel::SqliteConnection;

pub fn auth_routes() -> Router<r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>> {
    Router::new()
        .route("/auth/register", axum::routing::post(api::auth_handler::register))
        .route("/auth/login", axum::routing::post(api::auth_handler::login))
}