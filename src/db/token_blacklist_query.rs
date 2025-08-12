use diesel::prelude::*;
use diesel::pg::PgConnection;
use crate::errors::app_error::AppError;
use crate::schema::token_blacklist;
use chrono::{NaiveDateTime, Utc};

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::token_blacklist)]
pub struct NewBlacklistedToken {
    pub token: String,
    pub created_at: Option<NaiveDateTime>,
}

pub fn insert_blacklisted_token(
    conn: &mut PgConnection, 
    token_str: &str
) -> Result<(), AppError> {
    let blacklisted_token = NewBlacklistedToken {
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
    conn: &mut PgConnection, 
    token_str: &str
) -> Result<bool, AppError> {
    use diesel::dsl::exists;
    use diesel::select;
    
    // Menggunakan exists() untuk efisiensi - tidak perlu load seluruh row
    select(exists(
        token_blacklist::table
            .filter(token_blacklist::token.eq(token_str))
    ))
    .get_result(conn)
    .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn cleanup_expired_tokens(conn: &mut PgConnection, cutoff_date: NaiveDateTime) -> QueryResult<usize> {
    diesel::delete(
        crate::schema::token_blacklist::table
            .filter(crate::schema::token_blacklist::created_at.lt(cutoff_date))
    )
    .execute(conn)
}