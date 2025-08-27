use bon::Builder;
use domain::iam::value_object::permission_code::PermissionCode;
use domain::iam::{error::IamError, value_object::role_id::RoleId};
use infrastructure::shared::cloneable_error::CloneableError;
use infrastructure::shared::pool::Pool;
use nject::{inject, injectable};
use serde::Deserialize;
use serde_with::{NoneAsEmptyString, serde_as};
use single_flight_derive::single_flight;
use std::hash::Hash;
use std::time::Duration;

use crate::impl_static_cache;
use crate::shared::cache_type::{CacheType, hash_encode};
use crate::shared::dto::OptionDto;
use crate::{
    iam::dto::role::RoleDto,
    shared::{paging_query::PagingQuery, paging_result::PagingResult},
};

const CACHE_CAPACITY: u64 = 100;
const CACHE_TTL: Duration = Duration::from_secs(15 * 60);

impl_static_cache!(
    SEARCH_CACHE,
    PagingResult<RoleDto>,
    CACHE_CAPACITY,
    CACHE_TTL
);

#[derive(Clone)]
#[injectable]
pub struct RoleService {
    pool: Pool,
    #[inject(&SEARCH_CACHE)]
    cache: &'static CacheType<PagingResult<RoleDto>>,
}

impl RoleService {
    pub fn clean_cache(&self) {
        self.cache.invalidate_all();
    }

    pub async fn search_cached(
        &self,
        query: SearchRolesQuery,
    ) -> Result<PagingResult<RoleDto>, CloneableError> {
        let key = hash_encode(&query);
        self.cache.get_with(key, self.search(query)).await
    }

    #[single_flight]
    pub async fn search(
        &self,
        query: SearchRolesQuery,
    ) -> Result<PagingResult<RoleDto>, CloneableError> {
        let total_future = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM _roles
            WHERE ($1::text IS NULL OR name LIKE CONCAT('%', $1, '%'))
                AND ($2::boolean IS NULL OR privileged = $2)
                AND ($3::boolean IS NULL OR enabled = $3)
                AND ($4::integer IS NULL OR $4 = ANY(permission_ids))
            "#,
            query.name,
            query.privileged,
            query.enabled,
            query.permission_id,
        )
        .fetch_one(&self.pool);
        let page = query.paging.page();
        let page_size = query.paging.page_size();
        let offset = (page - 1) * page_size;
        let rows_future = sqlx::query_as!(
            RoleDto,
            r#"
        SELECT id, name, permission_ids as "permission_ids: Vec<PermissionCode>", privileged, enabled, created_at, updated_at
        FROM _roles
        WHERE ($1::text IS NULL OR name LIKE CONCAT('%', $1, '%'))
            AND ($2::boolean IS NULL OR privileged = $2)
            AND ($3::boolean IS NULL OR enabled = $3)
            AND ($4::integer IS NULL OR $4 = ANY(permission_ids))
        ORDER BY updated_at DESC
        LIMIT $5 OFFSET $6
        "#,
            query.name,
            query.privileged,
            query.enabled,
            query.permission_id,
            page_size,
            offset,
        )
        .fetch_all(&self.pool);
        let (total, rows) = tokio::try_join!(total_future, rows_future)?;
        Ok(PagingResult { total, items: rows })
    }

    #[single_flight]
    pub async fn retrieve(&self, id: RoleId) -> Result<RoleDto, CloneableError> {
        let row_opt = sqlx::query_as!(
            RoleDto,
            r#"
        SELECT id, name, permission_ids as "permission_ids: Vec<PermissionCode>", privileged, enabled, created_at, updated_at
        FROM _roles
        WHERE id = $1
        LIMIT 1
        "#,
            &id,
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt.ok_or(anyhow::anyhow!(IamError::RoleNotFound).into())
    }

    #[single_flight]
    pub async fn get_all(&self) -> Result<Vec<OptionDto>, CloneableError> {
        let options = sqlx::query_as!(
            OptionDto::<String>,
            r#"
        SELECT name as label, id as value FROM _roles ORDER BY updated_at DESC
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(options)
    }
}

#[serde_as]
#[derive(Clone, Eq, PartialEq, Hash, Deserialize, Builder)]
pub struct SearchRolesQuery {
    #[serde(flatten)]
    paging: PagingQuery,
    name: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    privileged: Option<bool>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    enabled: Option<bool>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    permission_id: Option<i32>,
}
