use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::prelude::*;
use jsonwebtoken::{encode, Header, EncodingKey};
use crate::models::auth::Claims;
use crate::models::user::{NewUser, User};
use crate::db::user_query;
use crate::errors::app_error::AppError;

pub async fn register_user(
    pool: &r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>,
    username: &str,
    email: &str,
    password: &str,
) -> Result<(), AppError> {
    let conn = &mut pool.get()?;
    let hashed_password = hash(password, DEFAULT_COST)?;
    let new_user = NewUser {
        username: username.to_string(),
        email: email.to_string(),
        password: hashed_password,
        settings: None,
    };
    user_query::insert_user(conn, new_user)?;
    Ok(())
}

pub async fn login_user(
    pool: &r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>,
    email: &str,
    password: &str,
) -> Result<String, AppError> {
    let conn = &mut pool.get()?;
    let user = user_query::find_user_by_email(conn, email)?
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    if !verify(password, &user.password)? {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let claims = Claims {
        sub: user.id.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp() as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(std::env::var("JWT_SECRET").expect("JWT_SECRET must be set").as_ref()),
    )?;
    Ok(token)
}