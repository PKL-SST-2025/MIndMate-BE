use axum::{
    extract::{State, Json, Query},
    response::IntoResponse,
};
use diesel::{r2d2, SqliteConnection};
use serde::Deserialize;
use std::collections::HashMap;

use crate::{
    errors::app_error::AppError,
    middleware::auth_middleware::AuthenticatedUser,
    service::user_service::{get_user_by_id, edit_profile, change_password, get_all_users, check_email_exists, reset_password},
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

/// Request body untuk check email
#[derive(Deserialize)]
pub struct CheckEmailRequest {
    pub email: String,
}

/// Handler untuk mengecek ketersediaan email via GET query parameter
/// GET /user/check-email?email=example@email.com
pub async fn check_email_handler_get(
    State(pool): State<DbPool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let email = params
        .get("email")
        .ok_or_else(|| AppError::BadRequest("Email parameter is required".to_string()))?;

    // Basic email format validation (optional)
    if !email.contains('@') || !email.contains('.') {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }

    let result = check_email_exists(&pool, email)?;
    Ok(Json(result))
}

/// Handler untuk mengecek ketersediaan email via POST body
/// POST /user/check-email dengan body: {"email": "example@email.com"}
pub async fn check_email_handler_post(
    State(pool): State<DbPool>,
    Json(data): Json<CheckEmailRequest>,
) -> Result<impl IntoResponse, AppError> {
    let email = data.email.trim();

    // Basic email format validation
    if email.is_empty() {
        return Err(AppError::BadRequest("Email cannot be empty".to_string()));
    }
    
    if !email.contains('@') || !email.contains('.') {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }

    let result = check_email_exists(&pool, email)?;
    Ok(Json(result))
}

/// Request body untuk reset password (lupa password)
#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub new_password: String,
    pub confirm_password: String,
}

/// Handler untuk reset password setelah verifikasi email
/// POST /user/reset-password dengan body: {"email": "example@email.com", "new_password": "newpass123", "confirm_password": "newpass123"}
pub async fn reset_password_handler(
    State(pool): State<DbPool>,
    Json(data): Json<ResetPasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    let email = data.email.trim();
    let new_password = data.new_password.trim();
    let confirm_password = data.confirm_password.trim();

    // Basic validation
    if email.is_empty() {
        return Err(AppError::BadRequest("Email cannot be empty".to_string()));
    }
    
    if !email.contains('@') || !email.contains('.') {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }

    if new_password.is_empty() {
        return Err(AppError::BadRequest("New password cannot be empty".to_string()));
    }

    if new_password.len() < 6 {
        return Err(AppError::BadRequest("Password must be at least 6 characters long".to_string()));
    }

    if new_password != confirm_password {
        return Err(AppError::BadRequest("Passwords do not match".to_string()));
    }

    // Reset password
    reset_password(&pool, email, new_password)?;
    Ok(Json("Password reset successfully"))
}