use background_job::JobPool;
use bon::Builder;
use nject::provider;

use crate::shared::{config::Config, kv::Kv, pg_pool::PgPool};

#[derive(Clone, Builder)]
#[provider]
pub struct Provider {
    #[provide(PgPool, |dep| dep.clone())]
    pg_pool: PgPool,
    #[provide(JobPool, |dep| dep.clone())]
    job_pool: JobPool,
    #[provide(Kv, |dep| dep.clone())]
    kv: Kv,
    #[provide(Config, |dep| dep.clone())]
    config: Config,
}
