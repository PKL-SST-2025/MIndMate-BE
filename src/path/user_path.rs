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
            put(user_handler::edit_profile_handler)
        )
        .route(
            "/user/password",
            put(user_handler::change_password_handler)
        )
        .route(
            "/users",
            get(user_handler::get_all_users_handler)
        )
        .route(
            "/user/check-email",
            get(user_handler::check_email_handler)
        )
}