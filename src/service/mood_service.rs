use crate::models::mood::{Mood, MoodResponse, MoodType}; // Now Mood will be used
use crate::db::mood_query;
use crate::errors::app_error::AppError;
use diesel::r2d2;
use diesel::SqliteConnection;
use chrono::NaiveDate;

pub fn create_mood(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    mood: &str,
    emoji: &str,
    notes: Option<String>,
    date: Option<NaiveDate>,
) -> Result<MoodResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // Validate mood type and USE as_str() method
    let mood_type = MoodType::from_str(mood)
        .ok_or_else(|| AppError::BadRequest(format!("Invalid mood type: {}", mood)))?;
    
    // Now USE as_str() method to ensure consistency
    let validated_mood = mood_type.as_str();

    // Check if mood already exists for the date
    let mood_date = date.unwrap_or_else(|| chrono::Utc::now().date_naive());
    if mood_query::check_mood_exists_for_date(&mut conn, user_id, mood_date)? {
        return Err(AppError::BadRequest("Mood already exists for this date".to_string()));
    }

    let mood_data = mood_query::create_mood(&mut conn, user_id, validated_mood, emoji, notes, date)?;

    Ok(MoodResponse {
        id: mood_data.id,
        user_id: mood_data.user_id,
        date: mood_data.date,
        mood: mood_data.mood,
        emoji: mood_data.emoji,
        notes: mood_data.notes,
        created_at: mood_data.created_at,
        updated_at: mood_data.updated_at,
    })
}

pub fn get_mood_by_id(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    mood_id: i32,
    user_id: i32,
) -> Result<MoodResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let mood = mood_query::find_mood_by_id(&mut conn, mood_id)
        .map_err(|_| AppError::NotFound("Mood not found".to_string()))?;

    // Check if user owns this mood
    if mood.user_id != user_id {
        return Err(AppError::BadRequest("Unauthorized access to mood".to_string()));
    }

    Ok(MoodResponse {
        id: mood.id,
        user_id: mood.user_id,
        date: mood.date,
        mood: mood.mood,
        emoji: mood.emoji,
        notes: mood.notes,
        created_at: mood.created_at,
        updated_at: mood.updated_at,
    })
}

pub fn get_user_moods(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<MoodResponse>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let moods = mood_query::find_moods_by_user(&mut conn, user_id, limit, offset)?;

    let mood_responses = moods.into_iter().map(|mood| MoodResponse {
        id: mood.id,
        user_id: mood.user_id,
        date: mood.date,
        mood: mood.mood,
        emoji: mood.emoji,
        notes: mood.notes,
        created_at: mood.created_at,
        updated_at: mood.updated_at,
    }).collect();

    Ok(mood_responses)
}

pub fn get_mood_by_date(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    date: NaiveDate,
) -> Result<MoodResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let mood = mood_query::find_mood_by_user_and_date(&mut conn, user_id, date)?;

    Ok(MoodResponse {
        id: mood.id,
        user_id: mood.user_id,
        date: mood.date,
        mood: mood.mood,
        emoji: mood.emoji,
        notes: mood.notes,
        created_at: mood.created_at,
        updated_at: mood.updated_at,
    })
}

pub fn get_moods_by_date_range(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<MoodResponse>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    if start_date > end_date {
        return Err(AppError::BadRequest("Start date cannot be after end date".to_string()));
    }

    let moods = mood_query::find_moods_by_date_range(&mut conn, user_id, start_date, end_date)?;

    let mood_responses = moods.into_iter().map(|mood| MoodResponse {
        id: mood.id,
        user_id: mood.user_id,
        date: mood.date,
        mood: mood.mood,
        emoji: mood.emoji,
        notes: mood.notes,
        created_at: mood.created_at,
        updated_at: mood.updated_at,
    }).collect();

    Ok(mood_responses)
}

pub fn update_mood(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    mood_id: i32,
    user_id: i32,
    new_mood: Option<String>,
    new_emoji: Option<String>,
    new_notes: Option<String>,
) -> Result<MoodResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // Validate mood type if provided and USE as_str() method
    let validated_mood = if let Some(ref mood) = new_mood {
        let mood_type = MoodType::from_str(mood)
            .ok_or_else(|| AppError::BadRequest(format!("Invalid mood type: {}", mood)))?;
        Some(mood_type.as_str().to_string()) // USE as_str() method
    } else {
        None
    };

    let updated_mood = mood_query::update_mood(&mut conn, mood_id, user_id, validated_mood, new_emoji, new_notes)?;

    Ok(MoodResponse {
        id: updated_mood.id,
        user_id: updated_mood.user_id,
        date: updated_mood.date,
        mood: updated_mood.mood,
        emoji: updated_mood.emoji,
        notes: updated_mood.notes,
        created_at: updated_mood.created_at,
        updated_at: updated_mood.updated_at,
    })
}

pub fn delete_mood(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    mood_id: i32,
    user_id: i32,
) -> Result<(), AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let deleted = mood_query::delete_mood(&mut conn, mood_id, user_id)?;
    if !deleted {
        return Err(AppError::NotFound("Mood not found".to_string()));
    }

    Ok(())
}

pub fn get_recent_moods(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    days: Option<i32>,
) -> Result<Vec<MoodResponse>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let days = days.unwrap_or(7);
    
    if days <= 0 || days > 365 {
        return Err(AppError::BadRequest("Days must be between 1 and 365".to_string()));
    }

    let moods = mood_query::get_recent_moods(&mut conn, user_id, days)?;

    let mood_responses = moods.into_iter().map(|mood| MoodResponse {
        id: mood.id,
        user_id: mood.user_id,
        date: mood.date,
        mood: mood.mood,
        emoji: mood.emoji,
        notes: mood.notes,
        created_at: mood.created_at,
        updated_at: mood.updated_at,
    }).collect();

    Ok(mood_responses)
}

pub fn get_mood_stats_count(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
) -> Result<i64, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    mood_query::get_mood_stats_simple(&mut conn, user_id)
}

pub fn get_mood_streak(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
) -> Result<i32, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let recent_moods = mood_query::get_recent_moods(&mut conn, user_id, 30)?;
    
    if recent_moods.is_empty() {
        return Ok(0);
    }

    let today = chrono::Utc::now().date_naive();
    let mut streak = 0;
    let mut current_date = today;

    for mood in recent_moods {
        if mood.date == current_date {
            streak += 1;
            current_date = current_date.pred_opt().unwrap_or(current_date);
        } else if mood.date < current_date {
            // Gap in streak, break
            break;
        }
    }

    Ok(streak)
}

// NEW: Function to get ALL user moods (uses get_all_moods_by_user)
pub fn get_all_user_moods(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
) -> Result<Vec<MoodResponse>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // NOW USING get_all_moods_by_user function
    let moods = mood_query::get_all_moods_by_user(&mut conn, user_id)?;

    let mood_responses = moods.into_iter().map(|mood| MoodResponse {
        id: mood.id,
        user_id: mood.user_id,
        date: mood.date,
        mood: mood.mood,
        emoji: mood.emoji,
        notes: mood.notes,
        created_at: mood.created_at,
        updated_at: mood.updated_at,
    }).collect();

    Ok(mood_responses)
}

// NEW: Function to get mood statistics with scores (uses score() method)
pub fn get_mood_stats_with_scores(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
) -> Result<serde_json::Value, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // Use get_all_moods_by_user to get all moods
    let moods: Vec<Mood> = mood_query::get_all_moods_by_user(&mut conn, user_id)?; // NOW Mood is used!

    if moods.is_empty() {
        return Ok(serde_json::json!({
            "total_entries": 0,
            "average_score": 0.0,
            "mood_distribution": {}
        }));
    }

    // Calculate statistics using score() method
    let mut total_score = 0i32;
    let mut mood_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();

    for mood in &moods {
        // USE score() method here!
        if let Some(mood_type) = MoodType::from_str(&mood.mood) {
            total_score += mood_type.score(); // NOW score() method is used!
            *mood_counts.entry(mood.mood.clone()).or_insert(0) += 1;
        }
    }

    let average_score = total_score as f64 / moods.len() as f64;

    Ok(serde_json::json!({
        "total_entries": moods.len(),
        "average_score": average_score,
        "mood_distribution": mood_counts
    }))
}