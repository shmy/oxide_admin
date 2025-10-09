use std::ops::Deref;

use crate::error::InfrastructureResult;
use crate::repository::organization::role_repository::RoleRepositoryImpl;
use crate::repository::organization::user_repository::UserRepositoryImpl;
use crate::shared::pg_pool::PgPool;
use domain::organization::entity::user::User;
use domain::organization::value_object::hashed_password::HashedPassword;
use domain::organization::value_object::user_id::UserId;
use domain::organization::{entity::role::Role, value_object::role_id::RoleId};
use domain::shared::port::domain_repository::DomainRepository;
use migrate_kit::{Migration, Migrator, embed_dir};
use tracing::info;

const MIGRATIONS: &[Migration] = embed_dir!("$CARGO_MANIFEST_DIR/migration/versions");

pub async fn migrate(
    pool: PgPool,
    user_repository: UserRepositoryImpl,
    role_repository: RoleRepositoryImpl,
) -> InfrastructureResult<()> {
    Migrator::builder()
        .pool(pool.clone())
        .build()
        .migrate(MIGRATIONS)
        .await?;
    insert_user_role(&pool, &user_repository, &role_repository).await?;
    Ok(())
}

async fn insert_user_role(
    pool: &PgPool,
    user_repository: &UserRepositoryImpl,
    role_repository: &RoleRepositoryImpl,
) -> InfrastructureResult<()> {
    let (role_opt, user_opt) = tokio::try_join!(
        sqlx::query!("SELECT id from _roles WHERE privileged = true").fetch_optional(pool),
        sqlx::query!("SELECT id from _users WHERE privileged = true").fetch_optional(pool),
    )?;

    if role_opt.is_none() {
        let role_id = RoleId::generate();
        let role = Role::builder()
            .id(role_id)
            .name("admin".to_string())
            .enabled(true)
            .privileged(true)
            .menus(vec![])
            .permissions(vec![])
            .build();
        let role = role_repository.save(role).await?;
        info!("Role inserted: {}", role.id.deref());
        if user_opt.is_none() {
            let user = User::builder()
                .id(UserId::generate())
                .account("admin".to_string())
                .password(HashedPassword::try_new("123456".to_string())?)
                .name("Admin".to_string())
                .enabled(true)
                .privileged(true)
                .role_ids(vec![role.id.clone()])
                .build();
            let user = user_repository.save(user).await?;
            info!("User inserted: {}", user.id.deref());
        }
    }
    Ok(())
}
