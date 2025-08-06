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
    models::mood::{CreateMoodRequest, UpdateMoodRequest, TrendQuery, AnalyticsQuery},
    service::mood_service::{
        create_mood, get_mood_by_id, get_user_moods, get_mood_by_date,
        get_moods_by_date_range, update_mood, delete_mood, get_recent_moods, 
        get_mood_stats_count, get_mood_streak,
        get_all_user_moods, get_mood_stats_with_scores,
        // NEW analytics functions
        get_average_mood, get_mood_trend, get_mood_distribution
    },
};

type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Deserialize)]
pub struct DateRangeQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
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
        data.date,
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

    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid date format. Use YYYY-MM-DD".to_string()))?;

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

/// Handler untuk mendapatkan SEMUA mood user tanpa pagination
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

/// Handler untuk mendapatkan statistik mood dengan scores
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

// NEW: Handler untuk mendapatkan rata-rata mood
/// GET /moods/analytics/average
pub async fn get_average_mood_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let average_mood = get_average_mood(&pool, user_id)?;
    Ok(Json(average_mood))
}

// NEW: Handler untuk mendapatkan trend mood (untuk grafik)
/// GET /moods/analytics/trend?days=30&group_by=week
pub async fn get_mood_trend_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Query(query): Query<TrendQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let trend = get_mood_trend(&pool, user_id, query.days, query.group_by)?;
    Ok(Json(trend))
}

// NEW: Handler untuk mendapatkan distribusi mood (untuk grafik pie/bar)
/// GET /moods/analytics/distribution?period=month
pub async fn get_mood_distribution_handler(
    State(pool): State<DbPool>,
    user: AuthenticatedUser,
    Query(query): Query<AnalyticsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = user
        .user_id()
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let distribution = get_mood_distribution(&pool, user_id, query.period)?;
    Ok(Json(distribution))
}