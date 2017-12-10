use diesel::pg::PgConnection;

use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

use std::env;
use std::sync::Arc;

pub struct ConnectionPool {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

pub fn init() -> ConnectionPool {
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in the environment.");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    let pool = Pool::builder().build(manager).expect(
        "Failed to create pool.",
    );

    ConnectionPool { pool: Arc::new(pool) }
}
