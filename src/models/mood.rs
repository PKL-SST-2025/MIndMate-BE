use diesel::prelude::*;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Debug, Serialize)]
#[diesel(table_name = crate::schema::moods)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Mood {
    pub id: i32,
    pub user_id: i32,
    pub date: chrono::NaiveDate,
    pub mood: String,
    pub emoji: String,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime, 
}

#[derive(Insertable, Debug, Deserialize)]
#[diesel(table_name = crate::schema::moods)]
pub struct NewMood {
    pub user_id: i32,
    pub date: chrono::NaiveDate,
    pub mood: String,
    pub emoji: String,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime, // Changed from Option<NaiveDateTime> to NaiveDateTime
}

#[derive(Serialize)]
pub struct MoodResponse {
    pub id: i32,
    pub user_id: i32,
    #[serde(serialize_with = "serialize_date")]
    pub date: chrono::NaiveDate,
    pub mood: String,
    pub emoji: String,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime, 
}

fn serialize_date<S>(date: &chrono::NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let formatted = date.format("%m-%d-%Y").to_string();
    serializer.serialize_str(&formatted)
}

#[derive(Debug, Deserialize)]
pub struct CreateMoodRequest {
    pub mood: String,
    pub emoji: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMoodRequest {
    pub mood: Option<String>,
    pub emoji: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MoodStats {
    pub total_entries: i64,
    pub mood_distribution: Vec<MoodCount>,
    pub average_mood_score: f64,
}

#[derive(Debug, Serialize)]
pub struct MoodCount {
    pub mood: String,
    pub count: i64,
    pub percentage: f64,
}

// Enum untuk validasi mood
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoodType {
    #[serde(rename = "very sad")]
    VerySad,
    #[serde(rename = "sad")]
    Sad,
    #[serde(rename = "neutral")]
    Neutral,
    #[serde(rename = "happy")]
    Happy,
    #[serde(rename = "very happy")]
    VeryHappy,
}

impl MoodType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MoodType::VerySad => "very sad",
            MoodType::Sad => "sad",
            MoodType::Neutral => "neutral",
            MoodType::Happy => "happy",
            MoodType::VeryHappy => "very happy",
        }
    }

    pub fn score(&self) -> i32 {
        match self {
            MoodType::VerySad => 1,
            MoodType::Sad => 2,
            MoodType::Neutral => 3,
            MoodType::Happy => 4,
            MoodType::VeryHappy => 5,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "very sad" => Some(MoodType::VerySad),
            "sad" => Some(MoodType::Sad),
            "neutral" => Some(MoodType::Neutral),
            "happy" => Some(MoodType::Happy),
            "very happy" => Some(MoodType::VeryHappy),
            _ => None,
        }
    }
}