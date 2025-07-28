use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::TypedHeader;
use headers::{Authorization, authorization::Bearer};
use serde::Deserialize;
use diesel::r2d2::{ Pool, ConnectionManager};
use diesel::SqliteConnection;

use crate::service::auth_service::{register_user, login_user};
use crate::errors::app_error::AppError;
use crate::db::token_blacklist_query::{insert_blacklisted_token, is_token_blacklisted};
use crate::utils::jwt::validate_token;

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
    State(pool): State<Pool<ConnectionManager<SqliteConnection>>>,
    Json(data): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    register_user(&pool, &data.username, &data.email, &data.password).await?;
    Ok((StatusCode::OK, Json("User registered successfully")))
}

pub async fn login(
    State(pool): State<Pool<ConnectionManager<SqliteConnection>>>,
    Json(data): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let token = login_user(&pool, &data.email, &data.password).await?;
    Ok((StatusCode::OK, Json(serde_json::json!({ "token": token }))))
}

pub async fn logout(
    State(pool): State<Pool<ConnectionManager<SqliteConnection>>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, AppError> {
    let token = auth.token();
    let claims = validate_token(token)?;
    let _user_id = claims.sub;

    let conn = &mut pool.get()?;

    if is_token_blacklisted(conn, token)? {
        return Err(AppError::Unauthorized("Token sudah tidak berlaku".to_string()));
    }

    insert_blacklisted_token(conn, token)?;

    Ok((StatusCode::OK, Json("Logout berhasil")))
}
