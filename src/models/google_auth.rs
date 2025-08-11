use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    // Removed unused fields: expires_in, refresh_token, scope, token_type, id_token
}

#[derive(Deserialize)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub given_name: Option<String>,
    pub picture: Option<String>,
}

#[derive(Serialize)]
pub struct GoogleLoginResponse {
    pub token: String,
    pub user: crate::models::user::UserResponse,
    pub is_new_user: bool,
}