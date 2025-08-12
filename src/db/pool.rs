use diesel::r2d2::{self, ConnectionManager};
use diesel::pg::PgConnection;

pub fn create_pool(database_url: String) -> r2d2::Pool<ConnectionManager<PgConnection>> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}