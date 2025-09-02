use bon::Builder;

use nject::provider;

use crate::shared::{config::Config, kv::Kv, pg_pool::PgPool, sqlite_pool::SqlitePool};

#[derive(Clone, Builder)]
#[provider]
pub struct Provider {
    #[provide(PgPool, |dep| dep.clone())]
    pg_pool: PgPool,
    #[provide(SqlitePool, |dep| dep.clone())]
    sqlite_pool: SqlitePool,
    #[provide(Kv, |dep| dep.clone())]
    kv: Kv,
    #[provide(Config, |dep| dep.clone())]
    config: Config,
}
