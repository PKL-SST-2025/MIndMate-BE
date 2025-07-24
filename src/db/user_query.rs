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