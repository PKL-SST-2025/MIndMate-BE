use axum::{Router, routing::{get, post, put, delete}};
use diesel::pg::PgConnection;
use diesel::r2d2;
use crate::api::mood_handler;

pub fn mood_routes() -> Router<r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>> {
    Router::new()
        // CRUD Operations
        .route(
            "/moods/all", 
            get(mood_handler::get_all_moods_handler)
        )
        .route(
            "/moods/stats/advanced", 
            get(mood_handler::get_advanced_mood_stats_handler)
        )
        .route(
            "/moods",
            post(mood_handler::create_mood_handler)
        )
        .route(
            "/moods",
            get(mood_handler::get_user_moods_handler)
        )
        .route(
            "/moods/:id",
            get(mood_handler::get_mood_by_id_handler)
        )
        .route(
            "/moods/:id",
            put(mood_handler::update_mood_handler)
        )
        .route(
            "/moods/:id",
            delete(mood_handler::delete_mood_handler)
        )
        
        // Query Operations
        .route(
            "/moods/date/:date",
            get(mood_handler::get_mood_by_date_handler)
        )
        .route(
            "/moods/range",
            get(mood_handler::get_moods_by_date_range_handler)
        )
        .route(
            "/moods/recent",
            get(mood_handler::get_recent_moods_handler)
        )
        
        // Special Operations
        .route(
            "/moods/stats",
            get(mood_handler::get_mood_stats_handler)
        )
        .route(
            "/moods/streak",
            get(mood_handler::get_mood_streak_handler)
        )
}