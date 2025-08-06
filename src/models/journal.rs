use diesel::prelude::*;
use chrono::{NaiveDateTime, NaiveDate};
use serde::{Deserialize, Serialize, Serializer};

// Custom serializer for NaiveDateTime to format as mm-dd-yyyy
fn serialize_datetime_as_mmddyyyy<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let formatted = dt.format("%m-%d-%Y").to_string();
    serializer.serialize_str(&formatted)
}

// Custom serializer for Option<NaiveDateTime> to format as mm-dd-yyyy
fn serialize_optional_datetime_as_mmddyyyy<S>(dt: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match dt {
        Some(datetime) => {
            let formatted = datetime.format("%m-%d-%Y").to_string();
            serializer.serialize_str(&formatted)
        },
        None => serializer.serialize_none(),
    }
}

#[derive(Queryable, Selectable, Debug, Serialize)]
#[diesel(table_name = crate::schema::journals)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Journal {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub content: String,
    #[serde(serialize_with = "serialize_datetime_as_mmddyyyy")]
    pub created_at: NaiveDateTime,
    #[serde(serialize_with = "serialize_optional_datetime_as_mmddyyyy")]
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Debug, Deserialize)]
#[diesel(table_name = crate::schema::journals)]
pub struct NewJournal {
    pub user_id: i32,
    pub title: String,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize)]
pub struct JournalResponse {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub content: String,
    #[serde(serialize_with = "serialize_datetime_as_mmddyyyy")]
    pub created_at: NaiveDateTime,
    #[serde(serialize_with = "serialize_optional_datetime_as_mmddyyyy")]
    pub updated_at: Option<NaiveDateTime>,
}

// Custom deserializer for NaiveDate to accept mm-dd-yyyy format
fn deserialize_date_from_mmddyyyy<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    
    let date_str: String = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&date_str, "%m-%d-%Y")
        .map(Some)
        .map_err(|_| D::Error::custom("Invalid date format. Expected MM-DD-YYYY"))
}

#[derive(Debug, Deserialize)]
pub struct CreateJournalRequest {
    pub title: String,
    pub content: String,
    // Now required field that accepts mm-dd-yyyy format
    #[serde(deserialize_with = "deserialize_date_from_mmddyyyy")]
    pub created_at: Option<NaiveDate>, // This will be Some() after deserialization
}

#[derive(Debug, Deserialize)]
pub struct UpdateJournalRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    // Note: We don't allow updating the created_at date in journal updates
}

#[derive(Debug, Serialize)]
pub struct JournalStats {
    pub total_entries: i64,
    pub total_words: i64,
    pub average_words_per_entry: f64,
    pub entries_this_month: i64,
    pub longest_entry_id: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct JournalWordCount {
    pub journal_id: i32,
    pub title: String,
    pub word_count: usize,
    #[serde(serialize_with = "serialize_datetime_as_mmddyyyy")]
    pub created_at: NaiveDateTime,
}

// Struct for advanced statistics including streak
#[derive(Debug, Serialize)]
pub struct JournalAdvancedStats {
    pub total_entries: i64,
    pub entries_last_30_days: i64,
    pub current_streak: i32,
}

// Simple response struct for basic stats endpoint
#[derive(Debug, Serialize)]
pub struct SimpleStatsResponse {
    pub total_entries: i64,
}