use axum::{extract::{State, Json}, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use crate::service::auth_service::{register_user, login_user};
use crate::errors::app_error::AppError;
use diesel::SqliteConnection;
use diesel::r2d2;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn register(
    State(pool): State<r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>>,
    Json(data): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    register_user(&pool, &data.username, &data.email, &data.password).await?;
    Ok((StatusCode::OK, Json("User registered successfully")))
}

pub async fn login(
    State(pool): State<r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>>,
    Json(data): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let token = login_user(&pool, &data.email, &data.password).await?;
    Ok((StatusCode::OK, Json(serde_json::json!({ "token": token }))))
}