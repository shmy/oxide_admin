use crate::error::{InfrastructureError, InfrastructureResult};
use bon::Builder;
use domain::auth::port::permission_resolver::PermissionResolver;
use domain::auth::value_object::permission::{ALL_PERMISSIONS, Permission};
use domain::auth::value_object::permission_group::PermissionGroup;
use domain::organization::value_object::role_id::RoleId;
use domain::organization::value_object::user_id::UserId;
use domain::shared::to_inner_vec::ToInnerVec as _;
use domain::system::error::SystemError;
use kvdb_kit::{Kvdb, KvdbTrait as _};
use nject::injectable;
use single_flight::single_flight;
use sqlx::prelude::FromRow;
use std::collections::HashSet;
use std::time::Duration;

use crate::shared::pg_pool::PgPool;

const KEY_PREFIX: &str = "permission:";

#[derive(Debug, Clone, Builder)]
#[injectable]
pub struct PermissionResolverImpl {
    pool: PgPool,
    kvdb: Kvdb,
}

impl PermissionResolverImpl {
    #[tracing::instrument]
    async fn solve(&self, id: &UserId) -> InfrastructureResult<PermissionGroup> {
        let res = async move {
            if let Ok(permission_group) = self.find_from_db(id.clone()).await {
                return permission_group;
            }
            PermissionGroup::default()
        }
        .await;
        Ok(res)
    }
}
impl PermissionResolver for PermissionResolverImpl {
    type Error = InfrastructureError;

    #[tracing::instrument]
    async fn resolve(&self, id: &UserId) -> PermissionGroup {
        match self.kvdb.get(&self.full_key(id)).await {
            Some(cache) => cache,
            None => match self.solve(id).await {
                Ok(cache) => {
                    let _ = self
                        .kvdb
                        .set_with_ex(
                            &self.full_key(id),
                            cache.clone(),
                            Duration::from_secs(30 * 60),
                        )
                        .await;
                    cache
                }
                Err(_) => Default::default(),
            },
        }
    }

    #[tracing::instrument]
    async fn refresh(&self) -> Result<(), Self::Error> {
        self.kvdb.delete_prefix(KEY_PREFIX).await?;
        Ok(())
    }
}

impl PermissionResolverImpl {
    #[single_flight]
    pub async fn find_from_db(&self, id: UserId) -> Result<PermissionGroup, SystemError> {
        let user_record = sqlx::query!(
            r#"SELECT privileged, role_ids as "role_ids: Vec<RoleId>" from _users WHERE id = $1"#,
            &id
        )
        .fetch_one(&self.pool)
        .await?;
        if user_record.privileged {
            return Ok(PermissionGroup::new(
                ALL_PERMISSIONS.iter().cloned().collect(),
            ));
        }

        let mut permissions = HashSet::new();
        let role_records = sqlx::query_as!(RoleRecord,r#"
            SELECT privileged, permissions as "permissions: Vec<Permission>" from _roles WHERE id = ANY($1) AND enabled = true
            "#,
            &user_record.role_ids.inner_vec()
        ).fetch_all(&self.pool).await?;

        for role in role_records {
            if role.privileged {
                permissions.extend(ALL_PERMISSIONS.to_vec());
            } else {
                permissions.extend(role.permissions);
            }
        }
        Ok(PermissionGroup::new(permissions))
    }

    fn full_key(&self, id: &UserId) -> String {
        format!("{}{}", KEY_PREFIX, &**id)
    }
}

#[derive(FromRow)]
struct RoleRecord {
    privileged: bool,
    permissions: Vec<Permission>,
}

#[cfg(test)]
mod tests {
    use domain::{
        organization::{
            entity::{role::Role, user::User},
            value_object::hashed_password::HashedPassword,
        },
        shared::port::domain_repository::DomainRepository as _,
    };

    use crate::{
        repository::organization::{
            role_repository::RoleRepositoryImpl, user_repository::UserRepositoryImpl,
        },
        shared::chrono_tz::ChronoTz,
        test_utils::{setup_database, setup_kvdb},
    };

    use super::*;

    async fn build_permission_resolver(pool: PgPool) -> PermissionResolverImpl {
        setup_database(pool.clone()).await;
        let kvdb = setup_kvdb().await;
        PermissionResolverImpl::builder()
            .pool(pool.clone())
            .kvdb(kvdb)
            .build()
    }

    #[sqlx::test]
    async fn test_resolve_privileged_user(pool: PgPool) {
        #[derive(FromRow)]
        struct UserRow {
            id: UserId,
        }
        let permission_resolver = build_permission_resolver(pool.clone()).await;
        let row: UserRow =
            sqlx::query_as(r#"SELECT id from _users WHERE privileged = true LIMIT 1"#)
                .fetch_one(&pool)
                .await
                .unwrap();
        let group = permission_resolver.resolve(&row.id).await;
        assert!(!group.is_empty());
        let group = permission_resolver.resolve(&UserId::generate()).await;
        assert!(group.is_empty());
    }

    #[sqlx::test]
    async fn test_resolve_non_privileged_user(pool: PgPool) {
        #[derive(FromRow)]
        struct RoleRow {
            id: RoleId,
        }
        let permission_resolver = build_permission_resolver(pool.clone()).await;
        let role = Role::builder()
            .id(RoleId::generate())
            .name("test".to_string())
            .enabled(true)
            .privileged(false)
            .menus(vec![])
            .permissions(vec![Permission::new(100), Permission::new(101)])
            .build();
        let user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .name("Test".to_string())
            .enabled(true)
            .privileged(false)
            .role_ids(vec![role.id.clone()])
            .build();
        let role_repository = RoleRepositoryImpl::builder()
            .pool(pool.clone())
            .ct(ChronoTz::default())
            .build();
        let user_repository = UserRepositoryImpl::builder()
            .pool(pool.clone())
            .ct(ChronoTz::default())
            .build();
        let mut user = user_repository.save(user).await.unwrap();
        let group = permission_resolver.resolve(&user.id).await;
        assert!(group.is_empty());
        assert!(role_repository.save(role).await.is_ok());
        let group = permission_resolver.resolve(&user.id).await;
        assert!(group.is_empty()); // because cached
        assert!(permission_resolver.refresh().await.is_ok()); // refresh cache
        let group = permission_resolver.resolve(&user.id).await;
        assert!(!group.is_empty());
        // add privileged user to the user
        let row: RoleRow =
            sqlx::query_as(r#"SELECT id from _roles WHERE privileged = true LIMIT 1"#)
                .fetch_one(&pool)
                .await
                .unwrap();
        let mut role_ids = user.role_ids.clone();
        role_ids.extend_from_slice(&[row.id]);
        user.update_role_ids(role_ids);
        let user = user_repository.save(user).await.unwrap();
        assert!(permission_resolver.refresh().await.is_ok()); // refresh cache
        let group = permission_resolver.resolve(&user.id).await;
        assert!(!group.is_empty());
    }
}
