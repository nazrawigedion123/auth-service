use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};

// Define the type alias for the connection pool
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Initializes and returns the Diesel R2D2 connection pool.
/// Panics if the connection pool cannot be created.
pub fn init_pool(database_url: &str) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    // The result of .expect() is the last expression, implicitly returned
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB connection pool.")
}
