use diesel::prelude::*;
use diesel::pg::PgConnection;
use chrono::{NaiveDate, Utc};
use crate::models::journal::{Journal, NewJournal};
use crate::errors::app_error::AppError;
use crate::schema::journals;

pub fn create_journal(
    conn: &mut PgConnection,
    user_id: i32,
    title: &str,
    content: &str,
    created_at: Option<NaiveDate>,
) -> Result<Journal, AppError> {
    let now = Utc::now().naive_utc();
    
    // Convert NaiveDate to NaiveDateTime
    let created_datetime = if let Some(date) = created_at {
        // Use the provided date at midnight
        date.and_hms_opt(0, 0, 0)
            .ok_or_else(|| AppError::BadRequest("Invalid date provided".to_string()))?
    } else {
        // Use current timestamp if no date provided
        now
    };
    
    let new_journal = NewJournal {
        user_id,
        title: title.to_string(),
        content: content.to_string(),
        created_at: created_datetime,
        updated_at: None,
    };

    diesel::insert_into(journals::table)
        .values(&new_journal)
        .execute(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Get the created journal by ordering by id desc to get the latest
    journals::table
        .filter(journals::user_id.eq(user_id))
        .order(journals::id.desc())
        .select(Journal::as_select())
        .first(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn find_journal_by_id(
    conn: &mut PgConnection,
    journal_id: i32,
) -> Result<Journal, AppError> {
    journals::table
        .filter(journals::id.eq(journal_id))
        .select(Journal::as_select())
        .first(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::NotFound("Journal not found".to_string()),
            _ => AppError::DatabaseError(e.to_string()),
        })
}

pub fn find_journals_by_user(
    conn: &mut PgConnection,
    user_id: i32,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<Journal>, AppError> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    journals::table
        .filter(journals::user_id.eq(user_id))
        .order(journals::created_at.desc())
        .limit(limit as i64)
        .offset(offset as i64)
        .select(Journal::as_select())
        .load::<Journal>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn find_journal_by_user_and_date(
    conn: &mut PgConnection,
    user_id: i32,
    date: NaiveDate,
) -> Result<Journal, AppError> {
    let start_of_day = date.and_hms_opt(0, 0, 0).unwrap_or_default();
    let end_of_day = date.and_hms_opt(23, 59, 59).unwrap_or_default();

    journals::table
        .filter(journals::user_id.eq(user_id))
        .filter(journals::created_at.ge(start_of_day))
        .filter(journals::created_at.le(end_of_day))
        .select(Journal::as_select())
        .first(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::NotFound("Journal not found for this date".to_string()),
            _ => AppError::DatabaseError(e.to_string()),
        })
}

pub fn find_journals_by_date_range(
    conn: &mut PgConnection,
    user_id: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<Journal>, AppError> {
    let start_datetime = start_date.and_hms_opt(0, 0, 0).unwrap_or_default();
    let end_datetime = end_date.and_hms_opt(23, 59, 59).unwrap_or_default();

    journals::table
        .filter(journals::user_id.eq(user_id))
        .filter(journals::created_at.between(start_datetime, end_datetime))
        .order(journals::created_at.asc())
        .select(Journal::as_select())
        .load::<Journal>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn update_journal(
    conn: &mut PgConnection,
    journal_id: i32,
    user_id: i32,
    new_title: Option<String>,
    new_content: Option<String>,
    new_created_at: Option<NaiveDate>, // Added this parameter
) -> Result<Journal, AppError> {
    // Check if journal exists and belongs to user
    let existing_journal = journals::table
        .filter(journals::id.eq(journal_id))
        .filter(journals::user_id.eq(user_id))
        .select(Journal::as_select())
        .first::<Journal>(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::NotFound("Journal not found".to_string()),
            _ => AppError::DatabaseError(e.to_string()),
        })?;

    // Build update values
    let title_to_update = new_title.unwrap_or(existing_journal.title);
    let content_to_update = new_content.unwrap_or(existing_journal.content);
    let created_at_to_update = if let Some(date) = new_created_at {
        date.and_hms_opt(0, 0, 0)
            .ok_or_else(|| AppError::BadRequest("Invalid date provided".to_string()))?
    } else {
        existing_journal.created_at
    };

    diesel::update(journals::table.filter(journals::id.eq(journal_id)))
        .set((
            journals::title.eq(title_to_update),
            journals::content.eq(content_to_update),
            journals::created_at.eq(created_at_to_update),
            journals::updated_at.eq(Some(Utc::now().naive_utc())),
        ))
        .execute(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    find_journal_by_id(conn, journal_id)
}

pub fn delete_journal(
    conn: &mut PgConnection,
    journal_id: i32,
    user_id: i32,
) -> Result<bool, AppError> {
    let result = diesel::delete(
        journals::table
            .filter(journals::id.eq(journal_id))
            .filter(journals::user_id.eq(user_id))
    )
    .execute(conn)
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(result > 0)
}

pub fn get_recent_journals(
    conn: &mut PgConnection,
    user_id: i32,
    days: i32,
) -> Result<Vec<Journal>, AppError> {
    let cutoff_date = Utc::now().date_naive() - chrono::Duration::days(days as i64);
    let cutoff_datetime = cutoff_date.and_hms_opt(0, 0, 0).unwrap_or_default();
    
    journals::table
        .filter(journals::user_id.eq(user_id))
        .filter(journals::created_at.ge(cutoff_datetime))
        .order(journals::created_at.desc())
        .select(Journal::as_select())
        .load::<Journal>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn get_journal_stats_simple(
    conn: &mut PgConnection,
    user_id: i32,
) -> Result<i64, AppError> {
    use diesel::dsl::count;
    
    journals::table
        .filter(journals::user_id.eq(user_id))
        .select(count(journals::id))
        .first(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn get_all_journals_by_user(
    conn: &mut PgConnection,
    user_id: i32,
) -> Result<Vec<Journal>, AppError> {
    journals::table
        .filter(journals::user_id.eq(user_id))
        .order(journals::created_at.desc())
        .select(Journal::as_select())
        .load::<Journal>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn search_journals(
    conn: &mut PgConnection,
    user_id: i32,
    search_query: &str,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<Journal>, AppError> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    let search_pattern = format!("%{}%", search_query);

    journals::table
        .filter(journals::user_id.eq(user_id))
        .filter(
            journals::title.like(&search_pattern)
                .or(journals::content.like(&search_pattern))
        )
        .order(journals::created_at.desc())
        .limit(limit as i64)
        .offset(offset as i64)
        .select(Journal::as_select())
        .load::<Journal>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}