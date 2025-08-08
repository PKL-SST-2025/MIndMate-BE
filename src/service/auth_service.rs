use crate::models::{user::User, auth::LoginResponse};
use crate::db::{user_query, token_blacklist_query};
use crate::errors::app_error::AppError;
use crate::utils::jwt::{generate_token, validate_token};
use diesel::r2d2;
use diesel::SqliteConnection;
use bcrypt::{hash, verify, DEFAULT_COST};

pub fn register_user(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    username: &str,
    email: &str,
    password: &str,
    age: Option<i32>,           // Parameter baru
    gender: Option<String>,     // Parameter baru
    settings: Option<String>,
) -> Result<User, AppError> {
    // Validate age and gender are not None or empty
    if age.is_none() {
        return Err(AppError::BadRequest("Age must be provided".to_string()));
    }
    if gender.is_none() || gender.as_ref().unwrap().trim().is_empty() {
        return Err(AppError::BadRequest("Gender must be provided".to_string()));
    }

    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // Check if email already exists
    if user_query::find_user_by_email(&mut conn, email).is_ok() {
        return Err(AppError::BadRequest("Email already exists".to_string()));
    }

    // Check if username already exists
    if user_query::find_user_by_username(&mut conn, username).is_ok() {
        return Err(AppError::BadRequest("Username already exists".to_string()));
    }

    // Hash password
    let hashed_password = hash(password, DEFAULT_COST)
        .map_err(|_| AppError::InternalServerError("Failed to hash password".to_string()))?;

    // Gunakan create_user yang sudah diupdate dengan semua parameter
    let user = user_query::create_user(&mut conn, username, email, &hashed_password, age, gender, settings)?;
    
    Ok(user)
}

pub fn login_user(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    email: &str,
    password: &str,
) -> Result<LoginResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // Find user by email
    let user = user_query::find_user_by_email(&mut conn, email)
        .map_err(|_| AppError::Unauthorized("Invalid email or password".to_string()))?;

    // Verify password
    let is_valid = verify(password, &user.password)
        .map_err(|_| AppError::InternalServerError("Failed to verify password".to_string()))?;

    if !is_valid {
        return Err(AppError::Unauthorized("Invalid email or password".to_string()));
    }

    // Generate JWT token with user ID
    let token = generate_token(&user.id.to_string())
        .map_err(|_| AppError::InternalServerError("Failed to generate token".to_string()))?;

    Ok(LoginResponse {
        token,
        user: crate::models::user::UserResponse {
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
        },
    })
}

pub fn logout_user(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    token: &str,
) -> Result<(), AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // Validate token first
    validate_token(token)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    // Check if token is already blacklisted
    let is_blacklisted = token_blacklist_query::is_token_blacklisted(&mut conn, token)
        .map_err(|_| AppError::InternalServerError("Failed to check token blacklist".to_string()))?;

    if is_blacklisted {
        return Err(AppError::Unauthorized("Token is already blacklisted".to_string()));
    }

    // Add token to blacklist
    token_blacklist_query::insert_blacklisted_token(&mut conn, token)?;

    Ok(())
}