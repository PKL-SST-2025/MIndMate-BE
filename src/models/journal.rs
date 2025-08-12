use diesel::prelude::*;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Debug, Serialize)]
#[diesel(table_name = crate::schema::journals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Journal {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub content: String,
    pub created_at: NaiveDateTime,
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
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJournalRequest {
    pub title: String,
    pub content: String,
    pub created_at: Option<String>, // Changed from NaiveDate to String for MM-DD-YYYY format
}

#[derive(Debug, Deserialize)]
pub struct UpdateJournalRequest {
    pub title: Option<String>,
    pub content: Option<String>,
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
    pub created_at: NaiveDateTime,
}