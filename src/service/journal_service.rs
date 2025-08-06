use crate::models::journal::{JournalResponse, JournalAdvancedStats}; 
use crate::db::journal_query;
use crate::errors::app_error::AppError;
use diesel::r2d2;
use diesel::SqliteConnection;
use chrono::{NaiveDate, Utc, Duration};
use std::collections::HashSet;

pub fn create_journal(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    title: &str,
    content: &str,
    created_at: Option<NaiveDate>,
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

    // created_at is now required - frontend must provide the date
    if created_at.is_none() {
        return Err(AppError::BadRequest("created_at date is required".to_string()));
    }

    let journal_data = journal_query::create_journal(&mut conn, user_id, title, content, created_at)?;

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

// FIXED: Helper function to calculate journal streak
fn calculate_journal_streak(journal_dates: Vec<NaiveDate>) -> i32 {
    if journal_dates.is_empty() {
        return 0;
    }

    let today = Utc::now().date_naive();
    let mut streak = 0;
    
    // Create a set of dates for quick lookup
    let date_set: HashSet<NaiveDate> = journal_dates.into_iter().collect();
    
    // PERBAIKAN: Mulai cek dari hari ini
    let mut current_date = today;
    
    // PERBAIKAN: Jika ada journal hari ini, mulai hitung dari hari ini
    // Jika tidak ada hari ini, cek kemarin dulu
    if date_set.contains(&current_date) {
        // Ada journal hari ini, mulai hitung streak dari hari ini
        while date_set.contains(&current_date) {
            streak += 1;
            current_date = current_date - Duration::days(1);
        }
    } else {
        // Tidak ada journal hari ini, cek kemarin
        current_date = today - Duration::days(1);
        if date_set.contains(&current_date) {
            // Ada journal kemarin, hitung streak dari kemarin
            while date_set.contains(&current_date) {
                streak += 1;
                current_date = current_date - Duration::days(1);
            }
        } else {
            // Tidak ada journal hari ini dan kemarin, streak = 0
            return 0;
        }
    }
    
    streak
}

// Function to get advanced statistics with streak
pub fn get_journal_advanced_stats(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
) -> Result<JournalAdvancedStats, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // Get total entries
    let total_entries = journal_query::get_journal_stats_simple(&mut conn, user_id)?;
    
    // Get entries in last 30 days
    let entries_last_30_days = journal_query::get_journal_count_last_days(&mut conn, user_id, 30)?;
    
    // Use the efficient get_journal_dates_by_user function
    let journal_dates = journal_query::get_journal_dates_by_user(&mut conn, user_id)?;
    
    // Calculate streak
    let current_streak = calculate_journal_streak(journal_dates);

    Ok(JournalAdvancedStats {
        total_entries,
        entries_last_30_days,
        current_streak,
    })
}

// Function to get simple stats (using the previously unused function)
pub fn get_journal_simple_stats(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
) -> Result<i64, AppError> {
    get_journal_stats_count(pool, user_id)
}

// Function to get streak information specifically 
pub fn get_journal_streak(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
) -> Result<i32, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // Use the efficient get_journal_dates_by_user function
    let journal_dates = journal_query::get_journal_dates_by_user(&mut conn, user_id)?;

    Ok(calculate_journal_streak(journal_dates))
}

// Function to get recent journals for streak tracking (using get_journals_for_streak)
pub fn get_journals_for_streak_analysis(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    days: Option<i32>,
) -> Result<Vec<JournalResponse>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let days = days.unwrap_or(30); // Default 30 days for streak analysis
    
    if days <= 0 || days > 365 {
        return Err(AppError::BadRequest("Days must be between 1 and 365".to_string()));
    }

    // Use the get_journals_for_streak function
    let journals = journal_query::get_journals_for_streak(&mut conn, user_id, days)?;

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