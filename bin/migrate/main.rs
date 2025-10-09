use migrate_kit::{Migration, Migrator, embed_dir};

const MIGRATIONS: &[Migration] =
    embed_dir!("$CARGO_MANIFEST_DIR/../../app/infrastructure/migration/versions");

#[tokio::main]
async fn main() {
    dotenvy::dotenv_override().ok();
    tracing_subscriber::fmt().init();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    Migrator::builder()
        .pool(pool.clone())
        .build()
        .migrate(MIGRATIONS)
        .await
        .expect("Failed to migrate");
}
