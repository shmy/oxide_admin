use crate::shared::{
    chrono_tz::ChronoTz, config::Config, feature_flag::FeatureFlag, pg_pool::PgPool,
};
use bg_worker_kit::queuer::Queuer;
use bon::Builder;
use kvdb_kit::Kvdb;
use nject::provider;
use object_storage_kit::ObjectStorage;

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
    #[provide(FeatureFlag, |dep| dep.clone())]
    feature_flag: FeatureFlag,
    #[provide(ChronoTz, |dep| dep.clone())]
    chrono_tz: ChronoTz,
}
