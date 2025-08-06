use crate::models::mood::{
    Mood, MoodResponse, MoodType, MoodTrendResponse, MoodTrendData,
    MoodDistributionResponse, MoodDistributionItem, AverageMoodResponse
};
use crate::db::mood_query;
use crate::errors::app_error::AppError;
use diesel::r2d2;
use diesel::SqliteConnection;
use chrono::{NaiveDate, Datelike};
use std::collections::HashMap;

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

    let mood_type = MoodType::from_str(mood)
        .ok_or_else(|| AppError::BadRequest(format!("Invalid mood type: {}", mood)))?;
    
    let validated_mood = mood_type.as_str();

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

    let validated_mood = if let Some(ref mood) = new_mood {
        let mood_type = MoodType::from_str(mood)
            .ok_or_else(|| AppError::BadRequest(format!("Invalid mood type: {}", mood)))?;
        Some(mood_type.as_str().to_string())
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
            break;
        }
    }

    Ok(streak)
}

pub fn get_all_user_moods(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
) -> Result<Vec<MoodResponse>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

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

pub fn get_mood_stats_with_scores(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
) -> Result<serde_json::Value, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let moods: Vec<Mood> = mood_query::get_all_moods_by_user(&mut conn, user_id)?;

    if moods.is_empty() {
        return Ok(serde_json::json!({
            "total_entries": 0,
            "average_score": 0.0,
            "mood_distribution": {}
        }));
    }

    let mut total_score = 0i32;
    let mut mood_counts: HashMap<String, i32> = HashMap::new();

    for mood in &moods {
        if let Some(mood_type) = MoodType::from_str(&mood.mood) {
            total_score += mood_type.score();
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

// NEW: Get average mood with different periods
pub fn get_average_mood(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
) -> Result<AverageMoodResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // Get all moods
    let all_moods = mood_query::get_all_moods_by_user(&mut conn, user_id)?;
    
    if all_moods.is_empty() {
        return Ok(AverageMoodResponse {
            overall_average: 0.0,
            weekly_average: None,
            monthly_average: None,
            yearly_average: None,
            total_entries: 0,
            mood_interpretation: "Tidak ada data".to_string(),
        });
    }

    // Calculate overall average
    let overall_score = calculate_average_score(&all_moods);

    // Get period-specific moods
    let weekly_moods = mood_query::get_moods_by_period(&mut conn, user_id, "week")?;
    let monthly_moods = mood_query::get_moods_by_period(&mut conn, user_id, "month")?;
    let yearly_moods = mood_query::get_moods_by_period(&mut conn, user_id, "year")?;

    let weekly_average = if !weekly_moods.is_empty() {
        Some(calculate_average_score(&weekly_moods))
    } else {
        None
    };

    let monthly_average = if !monthly_moods.is_empty() {
        Some(calculate_average_score(&monthly_moods))
    } else {
        None
    };

    let yearly_average = if !yearly_moods.is_empty() {
        Some(calculate_average_score(&yearly_moods))
    } else {
        None
    };

    Ok(AverageMoodResponse {
        overall_average: overall_score,
        weekly_average,
        monthly_average,
        yearly_average,
        total_entries: all_moods.len() as i64,
        mood_interpretation: MoodType::interpret_average_score(overall_score),
    })
}

// NEW: Get mood trend data
pub fn get_mood_trend(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    days: Option<i32>,
    group_by: Option<String>,
) -> Result<MoodTrendResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let moods = mood_query::get_moods_for_trend(&mut conn, user_id, days)?;

    if moods.is_empty() {
        return Ok(MoodTrendResponse {
            trend_data: vec![],
            average_score: 0.0,
            trend_direction: "stable".to_string(),
        });
    }

    let mut trend_data = Vec::new();
    let mut scores = Vec::new();

    // Process grouping if specified
    match group_by.as_deref() {
        Some("week") => {
            // Group by week
            let mut weekly_data: HashMap<String, Vec<(i32, String)>> = HashMap::new();
            
            for mood in moods {
                if let Some(mood_type) = MoodType::from_str(&mood.mood) {
                    let score = mood_type.score();
                    let week_key = format!("{}-W{}", mood.date.year(), mood.date.iso_week().week());
                    
                    weekly_data.entry(week_key)
                        .or_insert(Vec::new())
                        .push((score, mood.mood.clone()));
                }
            }
            
            for (week, week_data) in weekly_data {
                let week_scores: Vec<i32> = week_data.iter().map(|(score, _)| *score).collect();
                let avg_score = week_scores.iter().sum::<i32>() as f64 / week_scores.len() as f64;
                scores.push(avg_score);
                
                // Get most common mood for the week
                let mut mood_counts: HashMap<String, usize> = HashMap::new();
                for (_, mood) in &week_data {
                    *mood_counts.entry(mood.clone()).or_insert(0) += 1;
                }
                let most_common_mood = mood_counts
                    .into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(mood, _)| mood)
                    .unwrap_or_else(|| "neutral".to_string());
                
                // Parse week back to approximate date
                if let Some(year_week) = week.split('-').next() {
                    if let Ok(year) = year_week.parse::<i32>() {
                        let approx_date = NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or_else(|| chrono::Utc::now().date_naive());
                        trend_data.push(MoodTrendData {
                            date: approx_date,
                            score: avg_score as i32,
                            mood: format!("{} (avg: {:.1})", most_common_mood, avg_score),
                        });
                    }
                }
            }
        },
        Some("month") => {
            // Group by month
            let mut monthly_data: HashMap<String, Vec<(i32, String)>> = HashMap::new();
            
            for mood in moods {
                if let Some(mood_type) = MoodType::from_str(&mood.mood) {
                    let score = mood_type.score();
                    let month_key = format!("{}-{:02}", mood.date.year(), mood.date.month());
                    
                    monthly_data.entry(month_key)
                        .or_insert(Vec::new())
                        .push((score, mood.mood.clone()));
                }
            }
            
            for (month, month_data) in monthly_data {
                let month_scores: Vec<i32> = month_data.iter().map(|(score, _)| *score).collect();
                let avg_score = month_scores.iter().sum::<i32>() as f64 / month_scores.len() as f64;
                scores.push(avg_score);
                
                // Get most common mood for the month
                let mut mood_counts: HashMap<String, usize> = HashMap::new();
                for (_, mood) in &month_data {
                    *mood_counts.entry(mood.clone()).or_insert(0) += 1;
                }
                let most_common_mood = mood_counts
                    .into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(mood, _)| mood)
                    .unwrap_or_else(|| "neutral".to_string());
                
                // Parse month back to date
                let parts: Vec<&str> = month.split('-').collect();
                if parts.len() == 2 {
                    if let (Ok(year), Ok(month_num)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
                        let date = NaiveDate::from_ymd_opt(year, month_num, 1).unwrap_or_else(|| chrono::Utc::now().date_naive());
                        trend_data.push(MoodTrendData {
                            date,
                            score: avg_score as i32,
                            mood: format!("{} (avg: {:.1})", most_common_mood, avg_score),
                        });
                    }
                }
            }
        },
        _ => {
            // Default: daily data (no grouping)
            for mood in moods {
                if let Some(mood_type) = MoodType::from_str(&mood.mood) {
                    let score = mood_type.score();
                    scores.push(score as f64);
                    
                    trend_data.push(MoodTrendData {
                        date: mood.date,
                        score,
                        mood: mood.mood,
                    });
                }
            }
        }
    }

    // Sort by date
    trend_data.sort_by(|a, b| a.date.cmp(&b.date));

    let average_score = if scores.is_empty() { 0.0 } else { scores.iter().sum::<f64>() / scores.len() as f64 };
    let trend_direction = MoodType::determine_trend(&scores);

    Ok(MoodTrendResponse {
        trend_data,
        average_score,
        trend_direction,
    })
}

// NEW: Get mood distribution
pub fn get_mood_distribution(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    user_id: i32,
    period: Option<String>,
) -> Result<MoodDistributionResponse, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    let period_str = period.as_deref();
    let distribution_data = mood_query::get_mood_distribution_data(&mut conn, user_id, period_str)?;

    if distribution_data.is_empty() {
        return Ok(MoodDistributionResponse {
            distribution: vec![],
            total_entries: 0,
            most_common_mood: "".to_string(),
            average_score: 0.0,
        });
    }

    let total_entries: i64 = distribution_data.iter().map(|(_, count)| count).sum();
    let mut distribution = Vec::new();
    let mut total_score = 0i32;
    let mut most_common_mood = String::new();
    let mut max_count = 0i64;

    for (mood, count) in distribution_data {
        let percentage = (count as f64 / total_entries as f64) * 100.0;
        
        if let Some(mood_type) = MoodType::from_str(&mood) {
            let score = mood_type.score();
            total_score += score * (count as i32);
            
            if count > max_count {
                max_count = count;
                most_common_mood = mood.clone();
            }
            
            distribution.push(MoodDistributionItem {
                mood,
                count,
                percentage,
                score,
            });
        }
    }

    // Sort by count descending
    distribution.sort_by(|a, b| b.count.cmp(&a.count));

    let average_score = total_score as f64 / total_entries as f64;

    Ok(MoodDistributionResponse {
        distribution,
        total_entries,
        most_common_mood,
        average_score,
    })
}

// Helper function to calculate average score from moods
fn calculate_average_score(moods: &[Mood]) -> f64 {
    if moods.is_empty() {
        return 0.0;
    }

    let total_score: i32 = moods
        .iter()
        .filter_map(|mood| MoodType::from_str(&mood.mood))
        .map(|mood_type| mood_type.score())
        .sum();

    total_score as f64 / moods.len() as f64
}