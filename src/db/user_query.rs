use diesel::prelude::*;
use diesel::SelectableHelper;
use crate::schema::users;
use crate::models::user::{NewUser, User};
use crate::errors::app_error::AppError;

pub fn insert_user(conn: &mut SqliteConnection, new_user: NewUser) -> Result<(), AppError> {
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(conn)?;
    Ok(())
}

pub fn find_user_by_email(conn: &mut SqliteConnection, email: &str) -> Result<Option<User>, AppError> {
    let user = users::table
        .filter(users::email.eq(email))
        .select(User::as_select())
        .first::<User>(conn)
        .optional()?;
    Ok(user)
}

pub fn find_user_by_id(conn: &mut SqliteConnection, user_id: i32) -> Result<Option<User>, AppError> {
    let user = users::table
        .filter(users::id.eq(user_id))
        .select(User::as_select())
        .first::<User>(conn)
        .optional()?;
    Ok(user)
}

pub fn update_user_profile(
    conn: &mut SqliteConnection,
    user_id: i32,
    new_username: &str,
    new_email: &str,
) -> Result<(), AppError> {
    use crate::schema::users::dsl::*;
    diesel::update(users.filter(id.eq(user_id)))
        .set((username.eq(new_username), email.eq(new_email)))
        .execute(conn)?;
    Ok(())
}

pub fn update_user_password(
    conn: &mut SqliteConnection,
    user_id: i32,
    new_password: &str,
) -> Result<(), AppError> {
    use crate::schema::users::dsl::*;
    diesel::update(users.filter(id.eq(user_id)))
        .set(password.eq(new_password))
        .execute(conn)?;
    Ok(())
}