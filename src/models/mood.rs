use diesel::prelude::*;
use chrono::{NaiveDateTime, NaiveDate};
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
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize)]
pub struct MoodResponse {
    pub id: i32,
    pub user_id: i32,
    pub date: chrono::NaiveDate,
    pub mood: String,
    pub emoji: String,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime, 
}

#[derive(Debug, Deserialize)]
pub struct CreateMoodRequest {
    pub mood: String,
    pub emoji: String,
    pub notes: Option<String>,
    pub date: Option<chrono::NaiveDate>,
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

// NEW: Struktur untuk trend data
#[derive(Debug, Serialize)]
pub struct MoodTrendData {
    pub date: NaiveDate,
    pub score: i32,
    pub mood: String,
}

#[derive(Debug, Serialize)]
pub struct MoodTrendResponse {
    pub trend_data: Vec<MoodTrendData>,
    pub average_score: f64,
    pub trend_direction: String, // "improving", "declining", "stable"
}

// NEW: Struktur untuk distribusi mood
#[derive(Debug, Serialize)]
pub struct MoodDistributionItem {
    pub mood: String,
    pub count: i64,
    pub percentage: f64,
    pub score: i32,
}

#[derive(Debug, Serialize)]
pub struct MoodDistributionResponse {
    pub distribution: Vec<MoodDistributionItem>,
    pub total_entries: i64,
    pub most_common_mood: String,
    pub average_score: f64,
}

// NEW: Struktur untuk average mood dengan periode
#[derive(Debug, Serialize)]
pub struct AverageMoodResponse {
    pub overall_average: f64,
    pub weekly_average: Option<f64>,
    pub monthly_average: Option<f64>,
    pub yearly_average: Option<f64>,
    pub total_entries: i64,
    pub mood_interpretation: String,
}

// NEW: Query parameters untuk trend dan analytics
#[derive(Debug, Deserialize)]
pub struct TrendQuery {
    pub days: Option<i32>,
    pub group_by: Option<String>, // "day", "week", "month"
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub period: Option<String>, // "week", "month", "year", "all"
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

    // NEW: Interpretasi mood berdasarkan score
    pub fn interpret_average_score(score: f64) -> String {
        match score {
            s if s >= 4.5 => "Sangat Bahagia".to_string(),
            s if s >= 3.5 => "Bahagia".to_string(),
            s if s >= 2.5 => "Netral".to_string(),
            s if s >= 1.5 => "Sedih".to_string(),
            _ => "Sangat Sedih".to_string(),
        }
    }

    // NEW: Determine trend direction
    pub fn determine_trend(scores: &[f64]) -> String {
        if scores.len() < 2 {
            return "stable".to_string();
        }

        let recent_avg = scores.iter().rev().take(scores.len() / 2).sum::<f64>() / (scores.len() / 2) as f64;
        let older_avg = scores.iter().take(scores.len() / 2).sum::<f64>() / (scores.len() / 2) as f64;

        let difference = recent_avg - older_avg;
        
        if difference > 0.3 {
            "improving".to_string()
        } else if difference < -0.3 {
            "declining".to_string()
        } else {
            "stable".to_string()
        }
    }
}