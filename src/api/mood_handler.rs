use axum::{
    extract::{State, Json, Path, Query},
    response::IntoResponse,
};
use diesel::{r2d2, PgConnection};
use serde::Deserialize;
use chrono::NaiveDate;

use crate::{
    errors::app_error::AppError,
    middleware::auth_middleware::AuthenticatedUser,
    models::mood::{CreateMoodRequest, UpdateMoodRequest},
    service::mood_service::{
        create_mood, get_mood_by_id, get_user_moods, get_mood_by_date,
        get_moods_by_date_range, update_mood, delete_mood, get_recent_moods, 
        get_mood_stats_count, get_mood_streak,
        get_all_user_moods, get_mood_stats_with_scores // NEW functions added
    },
};

// Type alias agar lebih singkat
type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>;

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Deserialize)]
#[serde(try_from = "DateRangeQueryRaw")]
pub struct DateRangeQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Deserialize)]
struct DateRangeQueryRaw {
    pub start_date: String,
    pub end_date: String,
}

impl TryFrom<DateRangeQueryRaw> for DateRangeQuery {
    type Error = AppError;

    fn try_from(raw: DateRangeQueryRaw) -> Result<Self, Self::Error> {
        let start_date = NaiveDate::parse_from_str(&raw.start_date, "%m-%d-%Y")
            .map_err(|_| AppError::BadRequest("Invalid start_date format. Use MM-DD-YYYY".to_string()))?;
        let end_date = NaiveDate::parse_from_str(&raw.end_date, "%m-%d-%Y")
            .map_err(|_| AppError::BadRequest("Invalid end_date format. Use MM-DD-YYYY".to_string()))?;
        
        Ok(DateRangeQuery {
            start_date,
            end_date,
        })
    }
}

#[derive(Deserialize)]
pub struct RecentQuery {
    pub days: Option<i32>,
}

/// Handler untuk membuat mood baru
pub async fn create_mood_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Json(data): Json<CreateMoodRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let mood_response = create_mood(
        &pool,
        user_id,
        &data.mood,
        &data.emoji,
        data.notes,
        None, // Date will be auto-generated
    )?;

    Ok(Json(mood_response))
}

/// Handler untuk mengambil mood berdasarkan ID
pub async fn get_mood_by_id_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Path(mood_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let mood_response = get_mood_by_id(&pool, mood_id, user_id)?;
    Ok(Json(mood_response))
}

/// Handler untuk mengambil semua mood user dengan pagination
pub async fn get_user_moods_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let moods = get_user_moods(&pool, user_id, pagination.limit, pagination.offset)?;
    Ok(Json(moods))
}

/// Handler untuk mengambil mood berdasarkan tanggal
pub async fn get_mood_by_date_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Path(date): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let parsed_date = NaiveDate::parse_from_str(&date, "%m-%d-%Y")
        .map_err(|_| AppError::BadRequest("Invalid date format. Use MM-DD-YYYY".to_string()))?;

    let mood_response = get_mood_by_date(&pool, user_id, parsed_date)?;
    Ok(Json(mood_response))
}

/// Handler untuk mengambil mood dalam rentang tanggal
pub async fn get_moods_by_date_range_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Query(range): Query<DateRangeQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let moods = get_moods_by_date_range(&pool, user_id, range.start_date, range.end_date)?;
    Ok(Json(moods))
}

/// Handler untuk mengupdate mood
pub async fn update_mood_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Path(mood_id): Path<i32>,
    Json(data): Json<UpdateMoodRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let updated_mood = update_mood(&pool, mood_id, user_id, data.mood, data.emoji, data.notes)?;
    Ok(Json(updated_mood))
}

/// Handler untuk menghapus mood
pub async fn delete_mood_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Path(mood_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    delete_mood(&pool, mood_id, user_id)?;
    Ok(Json("Mood deleted successfully"))
}

/// Handler untuk mengambil mood terbaru
pub async fn get_recent_moods_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Query(query): Query<RecentQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let moods = get_recent_moods(&pool, user_id, query.days)?;
    Ok(Json(moods))
}

/// Handler untuk mendapatkan statistik mood sederhana
pub async fn get_mood_stats_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let count = get_mood_stats_count(&pool, user_id)?;
    Ok(Json(serde_json::json!({
        "total_entries": count
    })))
}

/// Handler untuk mendapatkan streak mood
pub async fn get_mood_streak_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let streak = get_mood_streak(&pool, user_id)?;
    Ok(Json(serde_json::json!({
        "streak": streak
    })))
}

/// NEW: Handler untuk mendapatkan SEMUA mood user tanpa pagination
/// USES get_all_user_moods function
pub async fn get_all_moods_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let moods = get_all_user_moods(&pool, user_id)?;
    Ok(Json(moods))
}

/// NEW: Handler untuk mendapatkan statistik mood dengan scores
/// USES get_mood_stats_with_scores function (which uses score() method)
pub async fn get_advanced_mood_stats_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let stats = get_mood_stats_with_scores(&pool, user_id)?;
    Ok(Json(stats))
}