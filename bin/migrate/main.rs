use include_dir::{Dir, include_dir};
use migrate_kit::Migrator;

const MIGRATIONS_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/../../app/infrastructure/migration/versions");

#[tokio::main]
async fn main() {
    dotenvy::dotenv_override().ok();
    tracing_subscriber::fmt().init();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    Migrator::new(pool.clone())
        .migrate(&MIGRATIONS_DIR)
        .await
        .expect("Failed to migrate");
}
