use anyhow::Result;
use domain::iam::entity::user::User;
use domain::iam::value_object::hashed_password::HashedPassword;
use domain::iam::value_object::user_id::UserId;
use domain::iam::{entity::role::Role, value_object::role_id::RoleId};
use domain::shared::domain_repository::DomainRepository;
use tracing::info;

use crate::repository::iam::role_repository::RoleRepositoryImpl;
use crate::repository::iam::user_repository::UserRepositoryImpl;
use crate::shared::kv::{Kv, KvTrait as _};
use crate::shared::pg_pool::PgPool;
use crate::shared::provider::Provider;

const INSERT_USER_ROLE_KEY: &str = "insert_user_role";

pub async fn migrate(provider: &Provider) -> Result<()> {
    let pool = provider.provide::<PgPool>();
    let kv = provider.provide::<Kv>();
    sqlx::migrate!("./migration/sql").run(&pool).await?;
    insert_user_role(&pool, &kv, provider).await?;
    Ok(())
}

async fn insert_user_role(pool: &PgPool, kv: &Kv, provider: &Provider) -> Result<()> {
    let inserted = kv.get::<bool>(INSERT_USER_ROLE_KEY).unwrap_or_default();
    if inserted {
        info!("User role already inserted");
        return Ok(());
    }
    let (role_opt, user_opt) = tokio::try_join!(
        sqlx::query!("SELECT id from _roles WHERE privileged = true").fetch_optional(pool),
        sqlx::query!("SELECT id from _users WHERE privileged = true").fetch_optional(pool),
    )?;

    if role_opt.is_none() {
        let role_repository = provider.provide::<RoleRepositoryImpl>();
        let role_id = RoleId::generate();
        let role = Role::builder()
            .id(role_id)
            .name("admin".to_string())
            .enabled(true)
            .privileged(true)
            .permission_ids(vec![])
            .build();
        let role = role_repository.save(role).await?;

        if user_opt.is_none() {
            let user_repository = provider.provide::<UserRepositoryImpl>();
            let user = User::builder()
                .id(UserId::generate())
                .account("admin".to_string())
                .password(HashedPassword::try_new("123456".to_string())?)
                .name("Admin".to_string())
                .enabled(true)
                .privileged(true)
                .role_ids(vec![role.id.clone()])
                .build();
            user_repository.save(user).await?;
        }
    }
    kv.set::<bool>(INSERT_USER_ROLE_KEY, true)?;
    Ok(())
}
