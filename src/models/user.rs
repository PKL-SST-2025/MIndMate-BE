use diesel::prelude::*;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Debug, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub settings: Option<String>, 
    pub age: Option<i32>,
    pub gender: Option<String>,
    pub avatar: Option<String>, 
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub settings: Option<String>,
    pub age: Option<i32>,
    pub gender: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub age: Option<i32>,
    pub gender: Option<String>,
    pub avatar: Option<String>,
    pub settings: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}