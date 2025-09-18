use sqlx::PgPool;

use crate::{
    migration::migrate,
    repository::iam::{role_repository::RoleRepositoryImpl, user_repository::UserRepositoryImpl},
    shared::chrono_tz::ChronoTz,
};

pub async fn setup_database(pool: PgPool) {
    let ct = ChronoTz::builder().tz(chrono_tz::Asia::Shanghai).build();
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
