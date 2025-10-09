use crate::shared::{
    chrono_tz::ChronoTz, config::ConfigRef, feature_flag::FeatureFlag, pg_pool::PgPool,
    workspace::WorkspaceRef,
};
use bon::Builder;
use kvdb_kit::Kvdb;
use nject::provider;
use object_storage_kit::ObjectStorage;

#[derive(Clone, Builder)]
#[provider]
pub struct Provider {
    #[provide(PgPool, |dep| dep.clone())]
    pg_pool: PgPool,
    #[provide(ObjectStorage, |dep| dep.clone())]
    object_storage: ObjectStorage,
    #[provide(Kvdb, |dep| dep.clone())]
    kvdb: Kvdb,
    #[provide(ConfigRef, |dep| dep.clone())]
    config: ConfigRef,
    #[provide(FeatureFlag, |dep| dep.clone())]
    feature_flag: FeatureFlag,
    #[provide(ChronoTz, |dep| dep.clone())]
    chrono_tz: ChronoTz,
    #[provide(WorkspaceRef, |dep| dep.clone())]
    workspace: WorkspaceRef,
}
