#[cfg(feature = "test")]
pub async fn setup_database(pool: sqlx::PgPool) {
    use crate::{
        migration::migrate,
        repository::system::{
            role_repository::RoleRepositoryImpl, user_repository::UserRepositoryImpl,
        },
        shared::chrono_tz::ChronoTz,
    };

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

#[cfg(feature = "test")]
pub async fn setup_kvdb() -> kvdb_kit::Kvdb {
    use kvdb_kit::Kvdb;
    let dir = tempfile::tempdir().unwrap();
    Kvdb::try_new(dir.path().join("kvdb")).await.unwrap()
}

#[cfg(feature = "test")]
pub async fn setup_object_storage() -> object_storage_kit::ObjectStorage {
    use object_storage_kit::{FsConfig, ObjectStorage};
    use std::time::Duration;

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
