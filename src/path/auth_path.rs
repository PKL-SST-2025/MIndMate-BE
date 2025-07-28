use axum::Router;
use diesel::SqliteConnection;
use diesel::r2d2;
use crate::api::auth_handler;

pub fn auth_routes() -> Router<r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>> {
    Router::new()
        .route("/auth/register", axum::routing::post(auth_handler::register))
        .route("/auth/login", axum::routing::post(auth_handler::login))
        .route("/auth/logout", axum::routing::post(auth_handler::logout))
}
