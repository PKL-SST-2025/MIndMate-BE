use axum::{Router, routing::{get, post, put, delete}};
use diesel::SqliteConnection;
use diesel::r2d2;
use crate::api::journal_handler;

pub fn journal_routes() -> Router<r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>> {
    Router::new()
        // Special Operations - put first to avoid path conflicts
        .route(
            "/journals/all", 
            get(journal_handler::get_all_journals_handler)
        )
        .route(
            "/journals/stats",
            get(journal_handler::get_journal_stats_handler)
        )
        // NEW: Simple stats route (using the previously unused function)
        .route(
            "/journals/stats/simple",
            get(journal_handler::get_journal_simple_stats_handler)
        )
        // NEW: Streak-only endpoint (using the previously unused get_journals_for_streak)
        .route(
            "/journals/streak",
            get(journal_handler::get_journal_streak_handler)
        )
        // Advanced stats route with streak calculation
        .route(
            "/journals/stats/advanced",
            get(journal_handler::get_journal_advanced_stats_handler)
        )
        .route(
            "/journals/search",
            get(journal_handler::search_journals_handler)
        )
        .route(
            "/journals/recent",
            get(journal_handler::get_recent_journals_handler)
        )
        // NEW: Route untuk mendapatkan journals untuk streak analysis (using get_journals_for_streak)
        .route(
            "/journals/streak-analysis",
            get(journal_handler::get_journals_for_streak_handler)
        )

        // CRUD Operations
        .route(
            "/journals",
            post(journal_handler::create_journal_handler)
        )
        .route(
            "/journals",
            get(journal_handler::get_user_journals_handler)
        )
        .route(
            "/journals/:id",
            get(journal_handler::get_journal_by_id_handler)
        )
        .route(
            "/journals/:id",
            put(journal_handler::update_journal_handler)
        )
        .route(
            "/journals/:id",
            delete(journal_handler::delete_journal_handler)
        )

        // Query Operations
        .route(
            "/journals/date/:date",
            get(journal_handler::get_journal_by_date_handler)
        )
        .route(
            "/journals/range",
            get(journal_handler::get_journals_by_date_range_handler)
        )
}