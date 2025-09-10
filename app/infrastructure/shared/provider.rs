use bon::Builder;
#[cfg(feature = "bg_faktory")]
use faktory_bg::queuer::Queuer;
use nject::provider;

use crate::shared::{config::Config, kv::Kv, pg_pool::PgPool};

#[cfg(feature = "bg_faktory")]
#[derive(Clone, Builder)]
#[provider]
pub struct Provider {
    #[provide(PgPool, |dep| dep.clone())]
    pg_pool: PgPool,
    #[provide(Queuer, |dep| dep.clone())]
    publisher: Queuer,
    #[provide(Kv, |dep| dep.clone())]
    kv: Kv,
    #[provide(Config, |dep| dep.clone())]
    config: Config,
}

#[cfg(not(feature = "bg_faktory"))]
#[derive(Clone, Builder)]
#[provider]
pub struct Provider {
    #[provide(PgPool, |dep| dep.clone())]
    pg_pool: PgPool,
    #[provide(Kv, |dep| dep.clone())]
    kv: Kv,
    #[provide(Config, |dep| dep.clone())]
    config: Config,
}
