use axum::{
    extract::{State, Json, Path, Query},
    response::IntoResponse,
};
use diesel::{r2d2, SqliteConnection};
use serde::Deserialize;
use chrono::NaiveDate;

use crate::{
    errors::app_error::AppError,
    middleware::auth_middleware::AuthenticatedUser,
    models::journal::{CreateJournalRequest, UpdateJournalRequest},
    service::journal_service::{
        create_journal, get_journal_by_id, get_user_journals, get_journal_by_date,
        get_journals_by_date_range, update_journal, delete_journal, get_recent_journals,
        get_journal_stats_count, get_all_user_journals, search_journals, get_journal_advanced_stats,
        get_journal_simple_stats, get_journal_streak, get_journals_for_streak_analysis
    },
};

// Type alias agar lebih singkat
type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Deserialize)]
pub struct DateRangeQuery {
    pub start_date: String, // Changed to String to handle mm-dd-yyyy format
    pub end_date: String,   // Changed to String to handle mm-dd-yyyy format
}

#[derive(Deserialize)]
pub struct RecentQuery {
    pub days: Option<i32>,
}

#[derive(Deserialize)]
pub struct StreakAnalysisQuery {
    pub days: Option<i32>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Helper function to parse date in mm-dd-yyyy format
fn parse_date_mmddyyyy(date_str: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(date_str, "%m-%d-%Y")
        .map_err(|_| AppError::BadRequest("Invalid date format. Use MM-DD-YYYY".to_string()))
}

/// Handler untuk membuat journal baru
pub async fn create_journal_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Json(data): Json<CreateJournalRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    // Validate that created_at is provided (required field now)
    let created_at = data.created_at
        .ok_or_else(|| AppError::BadRequest("created_at date is required".to_string()))?;

    let journal_response = create_journal(
        &pool,
        user_id,
        &data.title,
        &data.content,
        Some(created_at), // Always pass Some since it's required
    )?;

    Ok(Json(journal_response))
}

/// Handler untuk mengambil journal berdasarkan ID
pub async fn get_journal_by_id_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Path(journal_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let journal_response = get_journal_by_id(&pool, journal_id, user_id)?;
    Ok(Json(journal_response))
}

/// Handler untuk mengambil semua journal user dengan pagination
pub async fn get_user_journals_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let journals = get_user_journals(&pool, user_id, pagination.limit, pagination.offset)?;
    Ok(Json(journals))
}

/// Handler untuk mengambil journal berdasarkan tanggal (format mm-dd-yyyy)
pub async fn get_journal_by_date_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Path(date): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let parsed_date = parse_date_mmddyyyy(&date)?;

    let journal_response = get_journal_by_date(&pool, user_id, parsed_date)?;
    Ok(Json(journal_response))
}

/// Handler untuk mengambil journal dalam rentang tanggal (format mm-dd-yyyy)
pub async fn get_journals_by_date_range_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Query(range): Query<DateRangeQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let start_date = parse_date_mmddyyyy(&range.start_date)?;
    let end_date = parse_date_mmddyyyy(&range.end_date)?;

    let journals = get_journals_by_date_range(&pool, user_id, start_date, end_date)?;
    Ok(Json(journals))
}

/// Handler untuk mengupdate journal
pub async fn update_journal_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Path(journal_id): Path<i32>,
    Json(data): Json<UpdateJournalRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let updated_journal = update_journal(&pool, journal_id, user_id, data.title, data.content)?;
    Ok(Json(updated_journal))
}

/// Handler untuk menghapus journal
pub async fn delete_journal_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Path(journal_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    delete_journal(&pool, journal_id, user_id)?;
    Ok(Json("Journal deleted successfully"))
}

/// Handler untuk mengambil journal terbaru
pub async fn get_recent_journals_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Query(query): Query<RecentQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let journals = get_recent_journals(&pool, user_id, query.days)?;
    Ok(Json(journals))
}

/// Handler untuk mendapatkan statistik journal sederhana (backwards compatibility)
pub async fn get_journal_stats_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let count = get_journal_stats_count(&pool, user_id)?;
    Ok(Json(serde_json::json!({
        "total_entries": count
    })))
}

/// Handler untuk mendapatkan statistik journal simple
pub async fn get_journal_simple_stats_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let count = get_journal_simple_stats(&pool, user_id)?;
    Ok(Json(serde_json::json!({
        "total_entries": count
    })))
}

/// Handler untuk mendapatkan streak journal saja
pub async fn get_journal_streak_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let streak = get_journal_streak(&pool, user_id)?;
    Ok(Json(serde_json::json!({
        "current_streak": streak
    })))
}

/// Handler untuk mendapatkan statistik journal advanced dengan streak
pub async fn get_journal_advanced_stats_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let stats = get_journal_advanced_stats(&pool, user_id)?;
    Ok(Json(stats))
}

/// Handler untuk mendapatkan SEMUA journal user tanpa pagination
pub async fn get_all_journals_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let journals = get_all_user_journals(&pool, user_id)?;
    Ok(Json(journals))
}

/// Handler untuk mencari journal berdasarkan title atau content
pub async fn search_journals_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Query(search): Query<SearchQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let journals = search_journals(&pool, user_id, &search.query, search.limit, search.offset)?;
    Ok(Json(journals))
}

/// Handler untuk mendapatkan journals untuk analisis streak
pub async fn get_journals_for_streak_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Query(query): Query<StreakAnalysisQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let journals = get_journals_for_streak_analysis(&pool, user_id, query.days)?;
    Ok(Json(journals))
}