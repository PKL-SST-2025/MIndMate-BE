use axum::{
    extract::{State, Json},
    response::IntoResponse,
};
use diesel::{r2d2, SqliteConnection};
use serde::Deserialize;

use crate::{
    errors::app_error::AppError,
    middleware::auth_middleware::AuthenticatedUser,
    service::user_service::{get_user_by_id, edit_profile, change_password, get_all_users},
};

// Type alias agar lebih singkat
type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

/// Handler untuk mengambil profil pengguna
pub async fn get_profile(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let user_data = get_user_by_id(&pool, user_id)?;
    Ok(Json(user_data))
}

/// Request body untuk edit profil
#[derive(Deserialize)]
pub struct EditProfileRequest {
    pub username: String,
    pub email: String,
    pub age: Option<i32>,
    pub gender: Option<String>,
}

/// Handler untuk mengedit profil pengguna
pub async fn edit_profile_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Json(data): Json<EditProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    edit_profile(&pool, user_id, &data.username, &data.email, data.age, data.gender)?;
    Ok(Json("Profile updated successfully"))
}

/// Request body untuk ganti password
#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// Handler untuk mengganti password pengguna
pub async fn change_password_handler(
    State(pool): State<DbPool>,
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

/// Handler untuk mendapatkan semua pengguna
pub async fn get_all_users_handler(
    State(pool): State<DbPool>,
) -> Result<impl IntoResponse, AppError> {
    let users = get_all_users(&pool)?;
    Ok(Json(users))
}
