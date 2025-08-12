use crate::models::user::{User, UserResponse};
use crate::db::user_query;
use crate::errors::app_error::AppError;
use diesel::r2d2;
use diesel::pg::PgConnection;
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::Serialize;

// Response struct for email check
#[derive(Serialize)]
pub struct EmailCheckResponse {
    pub exists: bool,
    pub message: String,
}

pub fn get_user_by_id(
    pool: &r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
    user_id: i32,
) -> Result<UserResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let user = user_query::find_user_by_id(&mut conn, user_id)
        .map_err(|_| AppError::NotFound("User not found".to_string()))?;

    Ok(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        password: user.password,
        age: user.age,
        gender: user.gender,
        avatar: user.avatar, // Tambahan field avatar
        settings: user.settings.clone(),
        created_at: user.created_at,
        updated_at: user.updated_at,
    })
}

pub fn edit_profile(
    pool: &r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
    user_id: i32,
    new_username: &str,
    new_email: &str,
    new_age: Option<i32>,
    new_gender: Option<String>,
    new_avatar: Option<String>, // Tambahan parameter avatar
) -> Result<UserResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // Check if user exists
    let existing_user = user_query::find_user_by_id(&mut conn, user_id)
        .map_err(|_| AppError::NotFound("User not found".to_string()))?;

    // Check if new email is already taken by another user
    if new_email != existing_user.email {
        if let Ok(other_user) = user_query::find_user_by_email(&mut conn, new_email) {
            if other_user.id != user_id {
                return Err(AppError::BadRequest("Email already exists".to_string()));
            }
        }
    }

    // Check if new username is already taken by another user
    if new_username != existing_user.username {
        if let Ok(other_user) = user_query::find_user_by_username(&mut conn, new_username) {
            if other_user.id != user_id {
                return Err(AppError::BadRequest("Username already exists".to_string()));
            }
        }
    }

    // Update user dengan tambahan avatar parameter
    let updated_user = user_query::update_user_profile(&mut conn, user_id, new_username, new_email, new_age, new_gender, new_avatar)?;

    Ok(UserResponse {
        id: updated_user.id,
        username: updated_user.username,
        email: updated_user.email,
        password: updated_user.password,
        age: updated_user.age,
        gender: updated_user.gender,
        avatar: updated_user.avatar, // Tambahan field avatar
        settings: updated_user.settings.clone(),
        created_at: updated_user.created_at,
        updated_at: updated_user.updated_at,
    })
}

// Function for internal use to get full user data including password hash
pub fn get_user_full_data(
    pool: &r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
    user_id: i32,
) -> Result<User, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let user = user_query::find_user_by_id(&mut conn, user_id)
        .map_err(|_| AppError::NotFound("User not found".to_string()))?;

    Ok(user)
}

pub fn change_password(
    pool: &r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
    user_id: i32,
    old_password: &str,
    new_password: &str,
) -> Result<(), AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // Find user - using get_user_full_data for consistency
    let user = get_user_full_data(pool, user_id)?;

    // Verify old password
    let is_valid = verify(old_password, &user.password)
        .map_err(|_| AppError::InternalServerError("Failed to verify password".to_string()))?;

    if !is_valid {
        return Err(AppError::BadRequest("Invalid old password".to_string()));
    }

    // Hash new password
    let hashed_new_password = hash(new_password, DEFAULT_COST)
        .map_err(|_| AppError::InternalServerError("Failed to hash password".to_string()))?;

    // Update password
    user_query::update_user_password(&mut conn, user_id, &hashed_new_password)?;

    Ok(())
}

// New function to get all users
pub fn get_all_users(
    pool: &r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
) -> Result<Vec<UserResponse>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let users = user_query::get_all_users(&mut conn)?;

    // Map User to UserResponse dengan tambahan avatar
    let user_responses = users.into_iter().map(|user| UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        password: user.password,
        age: user.age,
        gender: user.gender,
        avatar: user.avatar, // Tambahan field avatar
        settings: user.settings.clone(),
        created_at: user.created_at,
        updated_at: user.updated_at,
    }).collect();

    Ok(user_responses)
}

// Function to check if email exists - untuk forgot password flow
pub fn check_email_exists(
    pool: &r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
    email: &str,
) -> Result<EmailCheckResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    match user_query::find_user_by_email(&mut conn, email) {
        Ok(_) => Ok(EmailCheckResponse {
            exists: true,
            message: "Email exists in database - you can proceed to reset password".to_string(),
        }),
        Err(AppError::NotFound(_)) => Ok(EmailCheckResponse {
            exists: false,
            message: "Email not found in database".to_string(),
        }),
        Err(e) => Err(e),
    }
}

// New function to reset password by email (for forgot password)
pub fn reset_password(
    pool: &r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
    email: &str,
    new_password: &str,
) -> Result<(), AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // First, check if user exists with this email
    let user = user_query::find_user_by_email(&mut conn, email)
        .map_err(|_| AppError::NotFound("Email not found in database".to_string()))?;

    // Hash the new password
    let hashed_new_password = hash(new_password, DEFAULT_COST)
        .map_err(|_| AppError::InternalServerError("Failed to hash password".to_string()))?;

    // Update password using user ID
    user_query::update_user_password(&mut conn, user.id, &hashed_new_password)?;

    Ok(())
}