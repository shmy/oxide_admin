use bon::Builder;
use domain::iam::{error::IamError, value_object::permission_code::PermissionCode};
use infrastructure::shared::pg_pool::PgPool;
use kvdb::Kvdb;
use nject::{inject, injectable};
use serde::Deserialize;
use serde_with::{NoneAsEmptyString, serde_as};
use single_flight::single_flight;
use std::time::Duration;

use crate::{
    iam::dto::role::RoleDto,
    shared::{
        cache_provider::CacheProvider, paging_query::PagingQuery, paging_result::PagingResult,
    },
};

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Builder)]
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

#[derive(Debug, Clone)]
#[injectable]
pub struct SearchRolesQueryHandler {
    pool: PgPool,
    #[inject(|kvdb: Kvdb| CacheProvider::builder().prefix("iam_search_roles:").ttl(Duration::from_secs(15 * 60)).kvdb(kvdb).build())]
    cache_provider: CacheProvider,
}

impl SearchRolesQueryHandler {
    #[single_flight]
    #[tracing::instrument]
    pub async fn query(&self, query: SearchRolesQuery) -> Result<PagingResult<RoleDto>, IamError> {
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

    #[tracing::instrument]
    pub async fn clean_cache(&self) -> anyhow::Result<()> {
        self.cache_provider.clear().await
    }

    #[tracing::instrument]
    pub async fn query_cached(
        &self,
        query: SearchRolesQuery,
    ) -> Result<PagingResult<RoleDto>, IamError> {
        self.cache_provider.get_with(query, |q| self.query(q)).await
    }
}
