use diesel::prelude::*;
use crate::schema::token_blacklist;
use crate::errors::app_error::AppError;
use diesel::SqliteConnection;

#[derive(Insertable)]
#[diesel(table_name = token_blacklist)]
pub struct NewBlacklistedToken<'a> {
    pub token: &'a str,
}

pub fn insert_blacklisted_token(conn: &mut SqliteConnection, token_str: &str) -> Result<(), AppError> {
    let new_token = NewBlacklistedToken { token: token_str };
    diesel::insert_into(token_blacklist::table)
        .values(&new_token)
        .execute(conn)?;
    Ok(())
}

pub fn is_token_blacklisted(conn: &mut SqliteConnection, token_str: &str) -> Result<bool, AppError> {
    use crate::schema::token_blacklist::dsl::*;
    let count = token_blacklist
        .filter(token.eq(token_str))
        .count()
        .get_result::<i64>(conn)?;
    Ok(count > 0)
}
