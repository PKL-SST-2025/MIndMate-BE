use serde::{Deserialize, Serialize};
use crate::models::user::UserResponse;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub age: Option<i32>,
    pub gender: Option<String>,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct GoogleCallbackRequest {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Serialize)]
pub struct GoogleAuthUrlResponse {
    pub auth_url: String,
}