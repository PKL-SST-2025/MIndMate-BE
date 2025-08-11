use crate::models::google_auth::{GoogleTokenResponse, GoogleUserInfo, GoogleLoginResponse};
use crate::db::user_query;
use crate::errors::app_error::AppError;
use crate::utils::jwt::generate_token;
use diesel::r2d2;
use diesel::SqliteConnection;
use reqwest;
use url::Url;
use rand::Rng;
use bcrypt;

pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl GoogleOAuthConfig {
    pub fn from_env() -> Result<Self, AppError> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .map_err(|_| AppError::InternalServerError("GOOGLE_CLIENT_ID not set".to_string()))?;
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .map_err(|_| AppError::InternalServerError("GOOGLE_CLIENT_SECRET not set".to_string()))?;
        let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
            .map_err(|_| AppError::InternalServerError("GOOGLE_REDIRECT_URI not set".to_string()))?;

        Ok(GoogleOAuthConfig {
            client_id,
            client_secret,
            redirect_uri,
        })
    }
}

pub fn generate_google_auth_url(config: &GoogleOAuthConfig) -> Result<String, AppError> {
    let state = generate_random_state();
    
    let mut url = Url::parse("https://accounts.google.com/o/oauth2/auth")
        .map_err(|_| AppError::InternalServerError("Failed to parse Google OAuth URL".to_string()))?;

    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", &config.redirect_uri)
        .append_pair("scope", "openid email profile")
        .append_pair("response_type", "code")
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("state", &state);

    Ok(url.to_string())
}

pub async fn exchange_code_for_token(
    config: &GoogleOAuthConfig,
    code: &str,
) -> Result<GoogleTokenResponse, AppError> {
    let client = reqwest::Client::new();
    
    let params = [
        ("client_id", config.client_id.as_str()),
        ("client_secret", config.client_secret.as_str()),
        ("code", code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", config.redirect_uri.as_str()),
    ];

    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to exchange code for token: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::InternalServerError(format!("Google OAuth error: {}", error_text)));
    }

    let token_response: GoogleTokenResponse = response
        .json()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to parse token response: {}", e)))?;

    Ok(token_response)
}

pub async fn get_user_info(access_token: &str) -> Result<GoogleUserInfo, AppError> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to get user info: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::InternalServerError(format!("Failed to get user info: {}", error_text)));
    }

    let user_info: GoogleUserInfo = response
        .json()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to parse user info: {}", e)))?;

    Ok(user_info)
}

pub async fn google_login(
    pool: &r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    code: &str,
    _state: Option<&str>, // Menggunakan state untuk validasi
) -> Result<GoogleLoginResponse, AppError> {
    let config = GoogleOAuthConfig::from_env()?;
    
    // Validasi state jika diperlukan (untuk security)
    // Untuk sekarang kita skip validasi state, tapi parameter tetap ada
    
    // Exchange code for token
    let token_response = exchange_code_for_token(&config, code).await?;
    
    // Get user info from Google
    let google_user = get_user_info(&token_response.access_token).await?;
    
    // Log informasi user untuk debugging (opsional)
    println!("Google user info: ID={}, Name={}, Email={}, Verified={}", 
             google_user.id, google_user.name, google_user.email, google_user.verified_email);
    
    let mut conn = pool
        .get()
        .map_err(|_| AppError::InternalServerError("Failed to get DB connection".to_string()))?;

    // Check if user already exists
    let (user, is_new_user) = match user_query::find_user_by_email(&mut conn, &google_user.email) {
        Ok(existing_user) => {
            // User exists, update avatar if available
            if let Some(_picture) = &google_user.picture {
                // You might want to add an update_user_avatar function
                // user_query::update_user_avatar(&mut conn, existing_user.id, picture)?;
                println!("User {} has profile picture: {}", google_user.email, _picture);
            }
            (existing_user, false)
        },
        Err(_) => {
            // User doesn't exist, create new user
            let username = generate_username_from_google_user(&google_user);
            let random_password = generate_random_password();
            
            // Hash the random password (user won't use it for Google login)
            let hashed_password = bcrypt::hash(&random_password, bcrypt::DEFAULT_COST)
                .map_err(|_| AppError::InternalServerError("Failed to hash password".to_string()))?;
            
            let new_user = user_query::create_user(
                &mut conn,
                &username,
                &google_user.email,
                &hashed_password,
                None, // age - you might want to prompt for this later
                None, // gender - you might want to prompt for this later
                None, // settings
            )?;
            
            println!("Created new user: {} with username: {}", google_user.email, username);
            (new_user, true)
        }
    };

    // Generate JWT token
    let jwt_token = generate_token(&user.id.to_string())
        .map_err(|_| AppError::InternalServerError("Failed to generate token".to_string()))?;

    Ok(GoogleLoginResponse {
        token: jwt_token,
        user: crate::models::user::UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            password: user.password,
            age: user.age,
            gender: user.gender,
            avatar: user.avatar,
            settings: user.settings,
            created_at: user.created_at,
            updated_at: user.updated_at,
        },
        is_new_user,
    })
}

pub fn get_google_auth_url() -> Result<String, AppError> {
    let config = GoogleOAuthConfig::from_env()?;
    generate_google_auth_url(&config)
}

fn generate_random_state() -> String {
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect()
}

// Menggunakan informasi lebih lengkap dari Google user untuk generate username
fn generate_username_from_google_user(google_user: &GoogleUserInfo) -> String {
    let base_username = if let Some(given_name) = &google_user.given_name {
        // Gunakan given_name jika ada
        given_name.to_lowercase().replace(' ', "")
    } else {
        // Fallback ke bagian email
        google_user.email.split('@').next().unwrap_or("user").to_string()
    };
    
    let random_suffix: u32 = rand::thread_rng().gen_range(1000..9999);
    format!("{}{}", base_username, random_suffix)
}

fn generate_random_password() -> String {
    let mut rng = rand::thread_rng();
    (0..16)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect()
}