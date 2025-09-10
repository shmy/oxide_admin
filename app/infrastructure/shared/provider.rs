use bon::Builder;
use faktory_bg::publisher::Publisher;
use nject::provider;

use crate::shared::{config::Config, kv::Kv, pg_pool::PgPool};

#[derive(Clone, Builder)]
#[provider]
pub struct Provider {
    #[provide(PgPool, |dep| dep.clone())]
    pg_pool: PgPool,

    #[provide(Publisher, |dep| dep.clone())]
    publisher: Publisher,

    #[provide(Kv, |dep| dep.clone())]
    kv: Kv,
    #[provide(Config, |dep| dep.clone())]
    config: Config,
}
