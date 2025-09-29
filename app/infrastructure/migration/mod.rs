use std::ops::Deref;

use crate::error::InfrastructureResult;
use domain::system::entity::user::User;
use domain::system::value_object::hashed_password::HashedPassword;
use domain::system::value_object::user_id::UserId;
use domain::system::{entity::role::Role, value_object::role_id::RoleId};
use domain::shared::port::domain_repository::DomainRepository;
use tracing::info;

use crate::repository::system::role_repository::RoleRepositoryImpl;
use crate::repository::system::user_repository::UserRepositoryImpl;
use crate::shared::pg_pool::PgPool;

pub async fn migrate(
    pool: PgPool,
    user_repository: UserRepositoryImpl,
    role_repository: RoleRepositoryImpl,
) -> InfrastructureResult<()> {
    sqlx::migrate!("migration/sql").run(&pool).await?;
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
