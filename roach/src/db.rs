use diesel::sqlite::SqliteConnection;
use diesel::r2d2::{Pool, ConnectionManager};

pub type DBPool = Pool<ConnectionManager<SqliteConnection>>;

pub fn create_db_pool(db_url: &str) -> DBPool {
    Pool::builder()
        .max_size(15)
        .build(ConnectionManager::new(db_url))
        .unwrap()
}