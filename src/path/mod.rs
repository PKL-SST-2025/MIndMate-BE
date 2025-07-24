use axum::Router;
use diesel::SqliteConnection;
use diesel::r2d2;

pub mod auth_path;

pub fn init_routes() -> Router<r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>> {
    Router::new()
        .merge(auth_path::auth_routes())
}