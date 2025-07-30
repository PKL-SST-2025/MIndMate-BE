use diesel::prelude::*;
use diesel::SqliteConnection;
use crate::errors::app_error::AppError;
use crate::schema::token_blacklist;
use chrono::{NaiveDateTime, Utc};

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::token_blacklist)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct BlacklistedToken {
    pub id: Option<i32>,
    pub token: String,
    pub created_at: Option<NaiveDateTime>,
}

pub fn insert_blacklisted_token(
    conn: &mut SqliteConnection, 
    token_str: &str
) -> Result<(), AppError> {
    let blacklisted_token = BlacklistedToken {
        id: None,
        token: token_str.to_string(),
        created_at: Some(Utc::now().naive_utc()),
    };

    diesel::insert_into(token_blacklist::table)
        .values(&blacklisted_token)
        .execute(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub fn is_token_blacklisted(
    conn: &mut SqliteConnection, 
    token_str: &str
) -> Result<bool, AppError> {
    let result = token_blacklist::table
        .filter(token_blacklist::token.eq(token_str))
        .first::<BlacklistedToken>(conn);

    match result {
        Ok(_) => Ok(true),  // Token found in blacklist
        Err(diesel::result::Error::NotFound) => Ok(false), // Token not in blacklist
        Err(e) => Err(AppError::DatabaseError(e.to_string())),
    }
}

pub fn cleanup_expired_tokens(conn: &mut SqliteConnection, cutoff_date: NaiveDateTime) -> QueryResult<usize> {
    diesel::delete(
        crate::schema::token_blacklist::table
            .filter(crate::schema::token_blacklist::created_at.lt(cutoff_date))
    )
    .execute(conn)
}
