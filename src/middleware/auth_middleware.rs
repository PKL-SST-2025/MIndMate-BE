use axum::{
    async_trait,
    extract::{FromRequestParts},
    http::{request::Parts},
};
use diesel::{r2d2, PgConnection};
use crate::utils::jwt::validate_token;
use crate::errors::app_error::AppError;

#[derive(Clone)]
pub struct AuthenticatedUser(pub String);

impl AuthenticatedUser {
    pub fn user_id(&self) -> &str {
        &self.0
    }
}

#[async_trait]
impl FromRequestParts<r2d2::Pool<r2d2::ConnectionManager<PgConnection>>> for AuthenticatedUser
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts, 
        state: &r2d2::Pool<r2d2::ConnectionManager<PgConnection>>
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers
            .get("Authorization")
            .ok_or_else(|| AppError::Unauthorized("Authorization header missing".to_string()))?;

        let auth_str = auth_header.to_str()
            .map_err(|_| AppError::Unauthorized("Invalid Authorization header".to_string()))?;

        if !auth_str.starts_with("Bearer ") {
            return Err(AppError::Unauthorized("Invalid Authorization scheme".to_string()));
        }

        let token = &auth_str[7..];

        let claims = validate_token(token)
            .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

        let mut conn = state
            .get()
            .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

        let is_blacklisted = crate::db::token_blacklist_query::is_token_blacklisted(&mut conn, token)
            .map_err(|_| AppError::InternalServerError("Failed to check token blacklist".to_string()))?;

        if is_blacklisted {
            return Err(AppError::Unauthorized("Token is blacklisted".to_string()));
        }

        Ok(AuthenticatedUser(claims.sub))
    }
}