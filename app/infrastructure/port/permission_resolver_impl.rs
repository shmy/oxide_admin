use anyhow::Result;
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::value_object::permission_code::{ALL_PERMISSIONS, PermissionCode};
use domain::iam::value_object::permission_group::PermissionGroup;
use domain::iam::value_object::role_id::RoleId;
use domain::iam::value_object::user_id::UserId;
use domain::shared::port::permission_resolver::PermissionResolver;
use domain::shared::to_inner_vec::ToInnerVec;
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
    async fn solve(&self, id: &UserId) -> Result<PermissionGroup> {
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
    type Error = anyhow::Error;

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
    pub async fn find_from_db(&self, id: UserId) -> Result<PermissionGroup, IamError> {
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
            SELECT privileged, permission_ids as "permission_ids: Vec<PermissionCode>" from _roles WHERE id = ANY($1)
            "#,
            &user_record.role_ids.inner_vec()
        ).fetch_all(&self.pool).await?;

        for role in role_records {
            if role.privileged {
                permissions.extend(ALL_PERMISSIONS.to_vec());
            } else {
                permissions.extend(role.permission_ids);
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
    permission_ids: Vec<PermissionCode>,
}

#[cfg(test)]
mod tests {
    use domain::{
        iam::{
            entity::{role::Role, user::User},
            value_object::hashed_password::HashedPassword,
        },
        shared::port::domain_repository::DomainRepository as _,
    };

    use crate::{
        repository::iam::{
            role_repository::RoleRepositoryImpl, user_repository::UserRepositoryImpl,
        },
        shared::chrono_tz::ChronoTz,
        test::{setup_database, setup_kvdb},
    };

    use super::*;
    #[sqlx::test]
    async fn test_resolve_privileged_user(pool: PgPool) {
        #[derive(FromRow)]
        struct UserRow {
            id: UserId,
        }
        setup_database(pool.clone()).await;
        let kvdb = setup_kvdb().await;
        let resolver = PermissionResolverImpl::builder()
            .pool(pool.clone())
            .kvdb(kvdb)
            .build();
        let row: UserRow = sqlx::query_as(
            r#"SELECT id AS "id: UserId" from _users WHERE privileged = true LIMIT 1"#,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let group = resolver.resolve(&row.id).await;
        assert!(!group.is_empty());
        let group = resolver.resolve(&UserId::generate()).await;
        assert!(group.is_empty());
    }

    #[sqlx::test]
    async fn test_resolve_non_privileged_user(pool: PgPool) {
        #[derive(FromRow)]
        struct RoleRow {
            id: RoleId,
        }
        setup_database(pool.clone()).await;
        let kvdb = setup_kvdb().await;
        let resolver = PermissionResolverImpl::builder()
            .pool(pool.clone())
            .kvdb(kvdb)
            .build();

        let role = Role::builder()
            .id(RoleId::generate())
            .name("test".to_string())
            .enabled(true)
            .privileged(false)
            .permission_ids(vec![PermissionCode::new(100), PermissionCode::new(101)])
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
        let group = resolver.resolve(&user.id).await;
        assert!(group.is_empty());
        assert!(role_repository.save(role).await.is_ok());
        let group = resolver.resolve(&user.id).await;
        assert!(group.is_empty()); // because cached
        assert!(resolver.refresh().await.is_ok()); // refresh cache
        let group = resolver.resolve(&user.id).await;
        assert!(!group.is_empty());
        // add privileged user to the user
        let row: RoleRow = sqlx::query_as(
            r#"SELECT id AS "id: RoleId" from _roles WHERE privileged = true LIMIT 1"#,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let mut role_ids = user.role_ids.clone();
        role_ids.extend_from_slice(&[row.id]);
        user.update_role_ids(role_ids);
        let user = user_repository.save(user).await.unwrap();
        assert!(resolver.refresh().await.is_ok()); // refresh cache
        let group = resolver.resolve(&user.id).await;
        assert!(!group.is_empty());
    }
}
