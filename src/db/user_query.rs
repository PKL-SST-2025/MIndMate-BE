use diesel::prelude::*;
use diesel::pg::PgConnection;
use crate::models::user::{User, NewUser};
use crate::errors::app_error::AppError;
use crate::schema::users;
use chrono::Utc;

// Function utama yang support semua parameter
pub fn create_user(
    conn: &mut PgConnection,
    username: &str,
    email: &str,
    password: &str,
    age: Option<i32>,
    gender: Option<String>,
    settings: Option<String>,
) -> Result<User, AppError> {
    let new_user = NewUser {
        username: username.to_string(),
        email: email.to_string(),
        password: password.to_string(),
        age,
        gender,
        settings,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Get the created user
    users::table
        .filter(users::email.eq(email))
        .select(User::as_select())
        .first(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn find_user_by_id(
    conn: &mut PgConnection,
    user_id: i32,
) -> Result<User, AppError> {
    users::table
        .filter(users::id.eq(user_id))
        .select(User::as_select())
        .first(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::NotFound("User not found".to_string()),
            _ => AppError::DatabaseError(e.to_string()),
        })
}

pub fn find_user_by_email(
    conn: &mut PgConnection,
    email: &str,
) -> Result<User, AppError> {
    users::table
        .filter(users::email.eq(email))
        .select(User::as_select())
        .first(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::NotFound("User not found".to_string()),
            _ => AppError::DatabaseError(e.to_string()),
        })
}

pub fn find_user_by_username(
    conn: &mut PgConnection,
    username: &str,
) -> Result<User, AppError> {
    users::table
        .filter(users::username.eq(username))
        .select(User::as_select())
        .first(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::NotFound("User not found".to_string()),
            _ => AppError::DatabaseError(e.to_string()),
        })
}

// Modifikasi function untuk include avatar parameter
pub fn update_user_profile(
    conn: &mut PgConnection,
    user_id: i32,
    new_username: &str,
    new_email: &str,
    new_age: Option<i32>,
    new_gender: Option<String>,
    new_avatar: Option<String>, // Tambahan parameter avatar
) -> Result<User, AppError> {
    diesel::update(users::table.filter(users::id.eq(user_id)))
        .set((
            users::username.eq(new_username),
            users::email.eq(new_email),
            users::age.eq(new_age),
            users::gender.eq(new_gender),
            users::avatar.eq(new_avatar), // Update avatar field
            users::updated_at.eq(Utc::now().naive_utc()),
        ))
        .execute(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    find_user_by_id(conn, user_id)
}

pub fn update_user_password(
    conn: &mut PgConnection,
    user_id: i32,
    new_password: &str,
) -> Result<(), AppError> {
    diesel::update(users::table.filter(users::id.eq(user_id)))
        .set((
            users::password.eq(new_password),
            users::updated_at.eq(Utc::now().naive_utc()),
        ))
        .execute(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}

// New function to get all users
pub fn get_all_users(conn: &mut PgConnection) -> Result<Vec<User>, AppError> {
    users::table
        .select(User::as_select())
        .load::<User>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}