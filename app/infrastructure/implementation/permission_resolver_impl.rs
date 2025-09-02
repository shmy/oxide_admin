use anyhow::Result;
use domain::iam::value_object::permission_code::{ALL_PERMISSIONS, PermissionCode};
use domain::iam::value_object::permission_group::PermissionGroup;
use domain::iam::value_object::role_id::RoleId;
use domain::iam::value_object::user_id::UserId;
use domain::shared::permission_resolver::PermissionResolver;
use domain::shared::to_inner_vec::ToInnerVec;
use moka::future::Cache;
use nject::injectable;
use single_flight::single_flight;
use sqlx::prelude::FromRow;
use std::collections::HashSet;
use std::sync::LazyLock;
use std::time::Duration;

use crate::shared::cloneable_error::CloneableError;
use crate::shared::pool::PgPool;

static PERMISSION_MAP: LazyLock<Cache<UserId, PermissionGroup>> = LazyLock::new(|| {
    Cache::<UserId, PermissionGroup>::builder()
        .max_capacity(100)
        .time_to_live(Duration::from_secs(60 * 30))
        .build()
});

#[derive(Debug, Clone)]
#[injectable]
pub struct PermissionResolverImpl {
    pool: PgPool,
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
        PERMISSION_MAP
            .try_get_with_by_ref(id, self.solve(id))
            .await
            .unwrap_or_default()
    }

    async fn refresh(&self) -> Result<(), Self::Error> {
        PERMISSION_MAP.invalidate_all();
        Ok(())
    }
}

impl PermissionResolverImpl {
    #[single_flight]
    pub async fn find_from_db(&self, id: UserId) -> Result<PermissionGroup, CloneableError> {
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
}

#[derive(FromRow)]
struct RoleRecord {
    privileged: bool,
    permission_ids: Vec<PermissionCode>,
}
