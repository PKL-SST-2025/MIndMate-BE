use crate::models::journal::JournalResponse; 
use crate::db::journal_query;
use crate::errors::app_error::AppError;
use diesel::r2d2;
use diesel::SqliteConnection;
use chrono::NaiveDate;

pub fn create_journal(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    title: &str,
    content: &str,
    created_at: Option<String>, // Changed from NaiveDate to String
) -> Result<JournalResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // Validate input
    if title.trim().is_empty() {
        return Err(AppError::BadRequest("Title cannot be empty".to_string()));
    }

    if content.trim().is_empty() {
        return Err(AppError::BadRequest("Content cannot be empty".to_string()));
    }

    // Parse the date from MM-DD-YYYY format if provided
    let parsed_date = if let Some(date_str) = created_at {
        Some(NaiveDate::parse_from_str(&date_str, "%m-%d-%Y")
            .map_err(|_| AppError::BadRequest("Invalid date format. Use MM-DD-YYYY".to_string()))?)
    } else {
        None
    };

    let journal_data = journal_query::create_journal(&mut conn, user_id, title, content, parsed_date)?;

    Ok(JournalResponse {
        id: journal_data.id,
        user_id: journal_data.user_id,
        title: journal_data.title,
        content: journal_data.content,
        created_at: journal_data.created_at,
        updated_at: journal_data.updated_at,
    })
}

pub fn get_journal_by_id(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    journal_id: i32,
    user_id: i32,
) -> Result<JournalResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let journal = journal_query::find_journal_by_id(&mut conn, journal_id)
        .map_err(|_| AppError::NotFound("Journal not found".to_string()))?;

    // Check if user owns this journal
    if journal.user_id != user_id {
        return Err(AppError::BadRequest("Unauthorized access to journal".to_string()));
    }

    Ok(JournalResponse {
        id: journal.id,
        user_id: journal.user_id,
        title: journal.title,
        content: journal.content,
        created_at: journal.created_at,
        updated_at: journal.updated_at,
    })
}

pub fn get_user_journals(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<JournalResponse>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let journals = journal_query::find_journals_by_user(&mut conn, user_id, limit, offset)?;

    let journal_responses = journals.into_iter().map(|journal| JournalResponse {
        id: journal.id,
        user_id: journal.user_id,
        title: journal.title,
        content: journal.content,
        created_at: journal.created_at,
        updated_at: journal.updated_at,
    }).collect();

    Ok(journal_responses)
}

pub fn get_journal_by_date(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    date: NaiveDate,
) -> Result<JournalResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let journal = journal_query::find_journal_by_user_and_date(&mut conn, user_id, date)?;

    Ok(JournalResponse {
        id: journal.id,
        user_id: journal.user_id,
        title: journal.title,  
        content: journal.content,
        created_at: journal.created_at,
        updated_at: journal.updated_at,
    })
}

pub fn get_journals_by_date_range(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<JournalResponse>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    if start_date > end_date {
        return Err(AppError::BadRequest("Start date cannot be after end date".to_string()));
    }

    let journals = journal_query::find_journals_by_date_range(&mut conn, user_id, start_date, end_date)?;

    let journal_responses = journals.into_iter().map(|journal| JournalResponse {
        id: journal.id,
        user_id: journal.user_id,
        title: journal.title,
        content: journal.content,
        created_at: journal.created_at,
        updated_at: journal.updated_at,
    }).collect();

    Ok(journal_responses)
}

pub fn update_journal(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    journal_id: i32,
    user_id: i32,
    new_title: Option<String>,
    new_content: Option<String>,
) -> Result<JournalResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // Validate input if provided
    if let Some(ref title) = new_title {
        if title.trim().is_empty() {
            return Err(AppError::BadRequest("Title cannot be empty".to_string()));
        }
    }

    if let Some(ref content) = new_content {
        if content.trim().is_empty() {
            return Err(AppError::BadRequest("Content cannot be empty".to_string()));
        }
    }

    let updated_journal = journal_query::update_journal(&mut conn, journal_id, user_id, new_title, new_content)?;

    Ok(JournalResponse {
        id: updated_journal.id,
        user_id: updated_journal.user_id,
        title: updated_journal.title,
        content: updated_journal.content,
        created_at: updated_journal.created_at,
        updated_at: updated_journal.updated_at,
    })
}

pub fn delete_journal(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    journal_id: i32,
    user_id: i32,
) -> Result<(), AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let deleted = journal_query::delete_journal(&mut conn, journal_id, user_id)?;
    if !deleted {
        return Err(AppError::NotFound("Journal not found".to_string()));
    }

    Ok(())
}

pub fn get_recent_journals(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    days: Option<i32>,
) -> Result<Vec<JournalResponse>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let days = days.unwrap_or(7);
    
    if days <= 0 || days > 365 {
        return Err(AppError::BadRequest("Days must be between 1 and 365".to_string()));
    }

    let journals = journal_query::get_recent_journals(&mut conn, user_id, days)?;

    let journal_responses = journals.into_iter().map(|journal| JournalResponse {
        id: journal.id,
        user_id: journal.user_id,
        title: journal.title,
        content: journal.content,
        created_at: journal.created_at,
        updated_at: journal.updated_at,
    }).collect();

    Ok(journal_responses)
}

pub fn get_journal_stats_count(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
) -> Result<i64, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    journal_query::get_journal_stats_simple(&mut conn, user_id)
}

pub fn get_all_user_journals(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
) -> Result<Vec<JournalResponse>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let journals = journal_query::get_all_journals_by_user(&mut conn, user_id)?;

    let journal_responses = journals.into_iter().map(|journal| JournalResponse {
        id: journal.id,
        user_id: journal.user_id,
        title: journal.title,
        content: journal.content,
        created_at: journal.created_at,
        updated_at: journal.updated_at,
    }).collect();

    Ok(journal_responses)
}

pub fn search_journals(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    search_query: &str,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<JournalResponse>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    if search_query.trim().is_empty() {
        return Err(AppError::BadRequest("Search query cannot be empty".to_string()));
    }

    let journals = journal_query::search_journals(&mut conn, user_id, search_query, limit, offset)?;

    let journal_responses = journals.into_iter().map(|journal| JournalResponse {
        id: journal.id,
        user_id: journal.user_id,
        title: journal.title,
        content: journal.content,
        created_at: journal.created_at,
        updated_at: journal.updated_at,
    }).collect();

    Ok(journal_responses)
}