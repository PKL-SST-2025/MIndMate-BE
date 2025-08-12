use axum::Router;
use diesel::pg::PgConnection;
use diesel::r2d2;

pub mod auth_path;
pub mod user_path;
pub mod mood_path;
pub mod journal_path;

pub fn init_routes() -> Router<r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>> {
    Router::new()
        .merge(auth_path::auth_routes())
        .merge(user_path::user_routes())
        .merge(mood_path::mood_routes())
        .merge(journal_path::journal_routes())
}