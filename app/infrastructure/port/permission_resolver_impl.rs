use anyhow::Result;
use domain::iam::error::IamError;
use domain::iam::value_object::permission_code::{ALL_PERMISSIONS, PermissionCode};
use domain::iam::value_object::permission_group::PermissionGroup;
use domain::iam::value_object::role_id::RoleId;
use domain::iam::value_object::user_id::UserId;
use domain::shared::port::permission_resolver::PermissionResolver;
use domain::shared::to_inner_vec::ToInnerVec;
use nject::injectable;
use single_flight::single_flight;
use sqlx::prelude::FromRow;
use std::collections::HashSet;
use std::time::Duration;

use crate::shared::kv::{Kv, KvTrait as _};
use crate::shared::pg_pool::PgPool;

#[derive(Debug, Clone)]
#[injectable]
pub struct PermissionResolverImpl {
    pool: PgPool,
    kv: Kv,
}

impl PermissionResolverImpl {
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
    async fn resolve(&self, id: &UserId) -> PermissionGroup {
        match self.kv.get(&self.full_key(id)).await {
            Some(cache) => cache,
            None => match self.solve(id).await {
                Ok(cache) => {
                    let _ = self
                        .kv
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

    async fn refresh(&self) -> Result<(), Self::Error> {
        todo!();
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
        format!("permission:{}", id.to_string())
    }
}

#[derive(FromRow)]
struct RoleRecord {
    privileged: bool,
    permission_ids: Vec<PermissionCode>,
}
