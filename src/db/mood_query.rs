use diesel::prelude::*;
use diesel::pg::PgConnection;
use chrono::{NaiveDate, Utc};
use crate::models::mood::{Mood, NewMood};
use crate::errors::app_error::AppError;
use crate::schema::moods;

pub fn create_mood(
    conn: &mut PgConnection,
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
        updated_at: Some(now), // Changed back to Some(now)
    };

    diesel::insert_into(moods::table)
        .values(&new_mood)
        .execute(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Get the created mood
    moods::table
        .filter(moods::user_id.eq(user_id))
        .filter(moods::date.eq(mood_date))
        .select(Mood::as_select())
        .first(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub fn find_mood_by_id(
    conn: &mut PgConnection,
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
    conn: &mut PgConnection,
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
    conn: &mut PgConnection,
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
    conn: &mut PgConnection,
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
    conn: &mut PgConnection,
    mood_id: i32,
    user_id: i32,
    new_mood: Option<String>,
    new_emoji: Option<String>,
    new_notes: Option<String>,
) -> Result<Mood, AppError> {
    // Check if mood exists and belongs to user
    let existing_mood = moods::table
        .filter(moods::id.eq(mood_id))
        .filter(moods::user_id.eq(user_id))
        .select(Mood::as_select())
        .first::<Mood>(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::NotFound("Mood not found".to_string()),
            _ => AppError::DatabaseError(e.to_string()),
        })?;

    // Build update query dynamically
    let mood_to_update = new_mood.unwrap_or(existing_mood.mood);
    let emoji_to_update = new_emoji.unwrap_or(existing_mood.emoji);
    let notes_to_update = if new_notes.is_some() { new_notes } else { existing_mood.notes };

    diesel::update(moods::table.filter(moods::id.eq(mood_id)))
        .set((
            moods::mood.eq(mood_to_update),
            moods::emoji.eq(emoji_to_update),
            moods::notes.eq(notes_to_update),
            moods::updated_at.eq(Some(Utc::now().naive_utc())), // Wrapped in Some()
        ))
        .execute(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    find_mood_by_id(conn, mood_id)
}

pub fn delete_mood(
    conn: &mut PgConnection,
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
    conn: &mut PgConnection,
    user_id: i32,
    days: i32,
) -> Result<Vec<Mood>, AppError> {
    let cutoff_date = Utc::now().date_naive() - chrono::Duration::days(days as i64);
    
    moods::table
        .filter(moods::user_id.eq(user_id))
        .filter(moods::date.ge(cutoff_date))
        .order(moods::date.desc())
        .select(Mood::as_select())
        .load::<Mood>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

// Function to get mood statistics (simplified version using Diesel)
pub fn get_mood_stats_simple(
    conn: &mut PgConnection,
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
    conn: &mut PgConnection,
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
    conn: &mut PgConnection,
    user_id: i32,
) -> Result<Vec<Mood>, AppError> {
    moods::table
        .filter(moods::user_id.eq(user_id))
        .order(moods::date.desc())
        .select(Mood::as_select())
        .load::<Mood>(conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}