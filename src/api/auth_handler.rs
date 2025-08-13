use axum::{
    extract::{State, Json, Query},
    response::{IntoResponse, Redirect},
    http::HeaderMap,
};
use crate::service::{
    auth_service::{register_user, login_user, logout_user},
    google_auth_service::{google_login, get_google_auth_url}
};
use crate::errors::app_error::AppError;
use crate::models::auth::{
    RegisterRequest, 
    LoginRequest, 
    LoginResponse, 
    GoogleCallbackRequest,
    GoogleAuthUrlResponse
};
use diesel::r2d2;
use diesel::pg::PgConnection;
use serde_json::json;
use std::env;

pub async fn register(
    State(pool): State<r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>>,
    Json(data): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = register_user(
        &pool, 
        &data.username,    
        &data.email,       
        &data.password,    
        data.age,          
        data.gender,     
        None              
    )?;
    
    Ok(Json(json!({
        "message": "User registered successfully",
        "user": {
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "age": user.age,
            "gender": user.gender,
            "password": user.password,
        }
    })))
}

pub async fn login(
    State(pool): State<r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>>,
    Json(data): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let login_response = login_user(&pool, &data.email, &data.password)?;
    
    Ok(Json(LoginResponse {
        token: login_response.token,
        user: login_response.user,
    }))
}

pub async fn logout(
    State(pool): State<r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    // Extract token dari Authorization header
    let auth_header = headers
        .get("Authorization")
        .ok_or_else(|| AppError::Unauthorized("Authorization header missing".to_string()))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid Authorization header".to_string()))?;

    if !auth_str.starts_with("Bearer ") {
        return Err(AppError::Unauthorized("Invalid Authorization scheme".to_string()));
    }

    let token = &auth_str[7..];

    // Proses logout (validasi token dan blacklist)
    logout_user(&pool, token)?;

    Ok(Json(json!({
        "message": "Successfully logged out"
    })))
}

// Google OAuth handlers
pub async fn google_auth_url() -> Result<impl IntoResponse, AppError> {
    let auth_url = get_google_auth_url()?;
    
    Ok(Json(GoogleAuthUrlResponse {
        auth_url,
    }))
}

pub async fn google_callback(
    State(pool): State<r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>>,
    Query(params): Query<GoogleCallbackRequest>,
) -> Result<impl IntoResponse, AppError> {
    let login_response = google_login(&pool, &params.code, params.state.as_deref()).await?;

    // Baca env var FRONTEND_URL, fallback ke localhost jika kosong
    let frontend_base_url = env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

    // Buat redirect URL sesuai apakah user baru atau tidak
    let redirect_url = if login_response.is_new_user {
        format!("{}/dashboard?welcome=1&token={}", frontend_base_url, login_response.token)
    } else {
        format!("{}/dashboard?token={}", frontend_base_url, login_response.token)
    };

    Ok(Redirect::permanent(&redirect_url))
}