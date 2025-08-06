use diesel::prelude::*;
use diesel::SqliteConnection;
use chrono::{NaiveDate, Utc, Duration};
use crate::models::mood::{Mood, NewMood};
use crate::errors::app_error::AppError;
use crate::schema::moods;

pub fn create_mood(
    conn: &mut SqliteConnection,
    user_id: i32,
    mood: &str,
    emoji: &str,
    notes: Option<String>,
    date: Option<NaiveDate>,
) -> Result<Mood, AppError> {
    let mood_date = date.unwrap_or_else(|| Utc::now().date_naive());
    let now = Utc::now().naive_utc();
    
    let new_mood = NewMood {
        user_id,
        date: mood_date,
        mood: mood.to_string(),
        emoji: emoji.to_string(),
        notes,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(moods::table)
        .values(&new_mood)
        .execute(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    moods::table
        .filter(moods::user_id.eq(user_id))
        .filter(moods::date.eq(mood_date))
        .select(Mood::as_select())
        .first(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn find_mood_by_id(
    conn: &mut SqliteConnection,
    mood_id: i32,
) -> Result<Mood, AppError> {
    moods::table
        .filter(moods::id.eq(mood_id))
        .select(Mood::as_select())
        .first(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::NotFound("Mood not found".to_string()),
            _ => AppError::DatabaseError(e.to_string()),
        })
}

pub fn find_moods_by_user(
    conn: &mut SqliteConnection,
    user_id: i32,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<Mood>, AppError> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    moods::table
        .filter(moods::user_id.eq(user_id))
        .order(moods::date.desc())
        .limit(limit as i64)
        .offset(offset as i64)
        .select(Mood::as_select())
        .load::<Mood>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn find_mood_by_user_and_date(
    conn: &mut SqliteConnection,
    user_id: i32,
    date: NaiveDate,
) -> Result<Mood, AppError> {
    moods::table
        .filter(moods::user_id.eq(user_id))
        .filter(moods::date.eq(date))
        .select(Mood::as_select())
        .first(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::NotFound("Mood not found for this date".to_string()),
            _ => AppError::DatabaseError(e.to_string()),
        })
}

pub fn find_moods_by_date_range(
    conn: &mut SqliteConnection,
    user_id: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<Mood>, AppError> {
    moods::table
        .filter(moods::user_id.eq(user_id))
        .filter(moods::date.between(start_date, end_date))
        .order(moods::date.asc())
        .select(Mood::as_select())
        .load::<Mood>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn update_mood(
    conn: &mut SqliteConnection,
    mood_id: i32,
    user_id: i32,
    new_mood: Option<String>,
    new_emoji: Option<String>,
    new_notes: Option<String>,
) -> Result<Mood, AppError> {
    let existing_mood = moods::table
        .filter(moods::id.eq(mood_id))
        .filter(moods::user_id.eq(user_id))
        .select(Mood::as_select())
        .first::<Mood>(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::NotFound("Mood not found".to_string()),
            _ => AppError::DatabaseError(e.to_string()),
        })?;

    let mood_to_update = new_mood.unwrap_or(existing_mood.mood);
    let emoji_to_update = new_emoji.unwrap_or(existing_mood.emoji);
    let notes_to_update = if new_notes.is_some() { new_notes } else { existing_mood.notes };

    diesel::update(moods::table.filter(moods::id.eq(mood_id)))
        .set((
            moods::mood.eq(mood_to_update),
            moods::emoji.eq(emoji_to_update),
            moods::notes.eq(notes_to_update),
            moods::updated_at.eq(Utc::now().naive_utc()),
        ))
        .execute(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    find_mood_by_id(conn, mood_id)
}

pub fn delete_mood(
    conn: &mut SqliteConnection,
    mood_id: i32,
    user_id: i32,
) -> Result<bool, AppError> {
    let result = diesel::delete(
        moods::table
            .filter(moods::id.eq(mood_id))
            .filter(moods::user_id.eq(user_id))
    )
    .execute(conn)
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(result > 0)
}

pub fn get_recent_moods(
    conn: &mut SqliteConnection,
    user_id: i32,
    days: i32,
) -> Result<Vec<Mood>, AppError> {
    let cutoff_date = Utc::now().date_naive() - Duration::days(days as i64);
    
    moods::table
        .filter(moods::user_id.eq(user_id))
        .filter(moods::date.ge(cutoff_date))
        .order(moods::date.desc())
        .select(Mood::as_select())
        .load::<Mood>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn get_mood_stats_simple(
    conn: &mut SqliteConnection,
    user_id: i32,
) -> Result<i64, AppError> {
    use diesel::dsl::count;
    
    moods::table
        .filter(moods::user_id.eq(user_id))
        .select(count(moods::id))
        .first(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn check_mood_exists_for_date(
    conn: &mut SqliteConnection,
    user_id: i32,
    date: NaiveDate,
) -> Result<bool, AppError> {
    use diesel::dsl::exists;
    use diesel::select;
    
    select(exists(
        moods::table
            .filter(moods::user_id.eq(user_id))
            .filter(moods::date.eq(date))
    ))
    .get_result(conn)
    .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn get_all_moods_by_user(
    conn: &mut SqliteConnection,
    user_id: i32,
) -> Result<Vec<Mood>, AppError> {
    moods::table
        .filter(moods::user_id.eq(user_id))
        .order(moods::date.desc())
        .select(Mood::as_select())
        .load::<Mood>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

// NEW: Get moods for trend analysis (chronologically ordered)
pub fn get_moods_for_trend(
    conn: &mut SqliteConnection,
    user_id: i32,
    days: Option<i32>,
) -> Result<Vec<Mood>, AppError> {
    let mut query = moods::table
        .filter(moods::user_id.eq(user_id))
        .into_boxed();

    if let Some(days) = days {
        let cutoff_date = Utc::now().date_naive() - Duration::days(days as i64);
        query = query.filter(moods::date.ge(cutoff_date));
    }

    query
        .order(moods::date.asc()) // Ascending for trend analysis
        .select(Mood::as_select())
        .load::<Mood>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

// NEW: Get moods grouped by period for analytics
pub fn get_moods_by_period(
    conn: &mut SqliteConnection,
    user_id: i32,
    period: &str,
) -> Result<Vec<Mood>, AppError> {
    let cutoff_date = match period {
        "week" => Utc::now().date_naive() - Duration::days(7),
        "month" => Utc::now().date_naive() - Duration::days(30),
        "year" => Utc::now().date_naive() - Duration::days(365),
        _ => NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(), // All time
    };

    moods::table
        .filter(moods::user_id.eq(user_id))
        .filter(moods::date.ge(cutoff_date))
        .order(moods::date.asc())
        .select(Mood::as_select())
        .load::<Mood>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

// NEW: Get mood distribution data - SIMPLIFIED VERSION
pub fn get_mood_distribution_data(
    conn: &mut SqliteConnection,
    user_id: i32,
    period: Option<&str>,
) -> Result<Vec<(String, i64)>, AppError> {
    // For now, let's get all moods and count them manually to avoid complex Diesel queries
    let mut moods_query = moods::table
        .filter(moods::user_id.eq(user_id))
        .into_boxed();

    if let Some(period) = period {
        let cutoff_date = match period {
            "week" => Utc::now().date_naive() - Duration::days(7),
            "month" => Utc::now().date_naive() - Duration::days(30),
            "year" => Utc::now().date_naive() - Duration::days(365),
            _ => NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
        };
        moods_query = moods_query.filter(moods::date.ge(cutoff_date));
    }

    let all_moods = moods_query
        .select(Mood::as_select())
        .load::<Mood>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Count moods manually
    let mut mood_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    
    for mood in all_moods {
        *mood_counts.entry(mood.mood).or_insert(0) += 1;
    }

    let result: Vec<(String, i64)> = mood_counts.into_iter().collect();
    Ok(result)
}