use bg_worker::queuer::Queuer;
use bon::Builder;
use kvdb::Kvdb;
use nject::provider;
use object_storage_kit::ObjectStorage;

use crate::shared::{config::Config, pg_pool::PgPool};

#[derive(Clone, Builder)]
#[provider]
pub struct Provider {
    #[provide(PgPool, |dep| dep.clone())]
    pg_pool: PgPool,
    #[provide(Queuer, |dep| dep.clone())]
    queuer: Queuer,
    #[provide(ObjectStorage, |dep| dep.clone())]
    object_storage: ObjectStorage,
    #[provide(Kvdb, |dep| dep.clone())]
    kvdb: Kvdb,
    #[provide(Config, |dep| dep.clone())]
    config: Config,
}
