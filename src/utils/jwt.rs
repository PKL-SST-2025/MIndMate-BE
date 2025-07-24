use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::models::auth::Claims;
use crate::errors::app_error::AppError;

pub fn validate_token(token: &str) -> Result<Claims, AppError> {
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(std::env::var("JWT_SECRET").expect("JWT_SECRET must be set").as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?
    .claims;
    Ok(claims)
}