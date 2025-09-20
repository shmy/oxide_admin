use std::time::Duration;

use kvdb_kit::Kvdb;
use object_storage_kit::{FsConfig, ObjectStorage};
use sqlx::PgPool;

use crate::{
    migration::migrate,
    repository::iam::{role_repository::RoleRepositoryImpl, user_repository::UserRepositoryImpl},
    shared::chrono_tz::ChronoTz,
};

pub async fn setup_database(pool: PgPool) {
    let ct = ChronoTz::default();
    let user_repository = UserRepositoryImpl::builder()
        .pool(pool.clone())
        .ct(ct.clone())
        .build();
    let role_repository = RoleRepositoryImpl::builder()
        .pool(pool.clone())
        .ct(ct.clone())
        .build();
    migrate(pool.clone(), user_repository, role_repository)
        .await
        .unwrap();
}

pub async fn setup_kvdb() -> Kvdb {
    let dir = tempfile::tempdir().unwrap();
    Kvdb::try_new(dir.path().join("kvdb")).await.unwrap()
}

pub async fn setup_object_storage() -> ObjectStorage {
    let dir = tempfile::tempdir().unwrap();
    ObjectStorage::try_new(
        FsConfig::builder()
            .root(dir.path().to_string_lossy().to_string())
            .basepath("/uploads".to_string())
            .hmac_secret(b"secret")
            .link_period(Duration::from_secs(60))
            .build(),
    )
    .unwrap()
}
