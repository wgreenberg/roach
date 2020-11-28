use diesel::sqlite::SqliteConnection;
use diesel::r2d2;

pub type DBPool = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;

pub fn create_db_pool(db_url: &str) -> DBPool {
    let manager = r2d2::ConnectionManager::new(db_url);
    r2d2::Pool::builder()
        .max_size(15)
        .build(manager)
        .unwrap()
}
