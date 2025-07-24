use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;

pub fn create_pool(database_url: String) -> r2d2::Pool<ConnectionManager<SqliteConnection>> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}