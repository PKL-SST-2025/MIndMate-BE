use axum::{Router, routing::{get, put}};
use diesel::SqliteConnection;
use diesel::r2d2;
use crate::api::user_handler;

pub fn user_routes() -> Router<r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>> {
    Router::new()
        .route(
            "/user/profile",
            get(user_handler::get_profile)
        )
        .route(
            "/user/profile",
            put(user_handler::edit_profile)
        )
        .route(
            "/user/password",
            put(user_handler::change_password)
        )
}