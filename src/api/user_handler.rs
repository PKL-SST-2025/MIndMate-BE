use axum::{
    extract::{State, Json},
    response::IntoResponse,
};
use crate::service::user_service::{get_user_by_id, edit_profile, change_password};
use crate::errors::app_error::AppError;
use crate::middleware::auth_middleware::AuthenticatedUser;
use diesel::r2d2;
use diesel::SqliteConnection;
use serde::Deserialize;

pub async fn get_profile(
    State(pool): State<r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let user_data = get_user_by_id(&pool, user_id)?;
    Ok(Json(user_data))
}

#[derive(Deserialize)]
pub struct EditProfileRequest {
    pub username: String,
    pub email: String,
}

pub async fn edit_profile(
    State(pool): State<r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>>,
    user: AuthenticatedUser,
    Json(data): Json<EditProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    edit_profile(&pool, user_id, &data.username, &data.email)?;
    Ok(Json("Profile updated successfully"))
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

pub async fn change_password(
    State(pool): State<r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>>,
    user: AuthenticatedUser,
    Json(data): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    change_password(&pool, user_id, &data.old_password, &data.new_password)?;
    Ok(Json("Password changed successfully"))
}