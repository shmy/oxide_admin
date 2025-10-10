use bon::Builder;
use domain::{
    auth::value_object::{menu::Menu, permission::Permission},
    organization::error::OrganizationError,
};
use infrastructure::shared::pg_pool::PgPool;
use kvdb_kit::Kvdb;
use nject::{inject, injectable};
use serde::Deserialize;
use serde_with::{NoneAsEmptyString, serde_as};
use single_flight::single_flight;
use std::time::Duration;
use utoipa::IntoParams;

use crate::{
    error::ApplicationResult,
    organization::dto::role::RoleDto,
    shared::{
        cache_provider::CacheProvider, paging_query::PagingQuery, paging_result::PagingResult,
        query_handler::QueryHandler,
    },
};

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, IntoParams, Builder)]
pub struct SearchRolesQuery {
    #[serde(flatten)]
    #[param(inline)]
    paging: PagingQuery,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    name: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    privileged: Option<bool>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    enabled: Option<bool>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    menu: Option<i32>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    permission: Option<i32>,
}

#[derive(Debug, Clone, Builder)]
#[injectable]
pub struct SearchRolesQueryHandler {
    pool: PgPool,
    #[inject(|kvdb: Kvdb| CacheProvider::builder().prefix("organization:search_roles:").ttl(Duration::from_secs(15 * 60)).kvdb(kvdb).build())]
    cache_provider: CacheProvider,
}

impl QueryHandler for SearchRolesQueryHandler {
    type Query = SearchRolesQuery;
    type Output = PagingResult<RoleDto>;
    type Error = OrganizationError;

    #[single_flight]
    #[tracing::instrument]
    async fn query(
        &self,
        query: SearchRolesQuery,
    ) -> Result<PagingResult<RoleDto>, OrganizationError> {
        let total_future = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM _roles
            WHERE ($1::text IS NULL OR name LIKE CONCAT('%', $1, '%'))
                AND ($2::boolean IS NULL OR privileged = $2)
                AND ($3::boolean IS NULL OR enabled = $3)
                AND ($4::integer IS NULL OR $4 = ANY(menus))
                AND ($5::integer IS NULL OR $5 = ANY(permissions))
            "#,
            query.name,
            query.privileged,
            query.enabled,
            query.menu,
            query.permission,
        )
        .fetch_one(&self.pool);
        let page = query.paging.page();
        let page_size = query.paging.page_size();
        let offset = (page - 1) * page_size;
        let rows_future = sqlx::query_as!(
            RoleDto,
            r#"
        SELECT id, name, menus as "menus: Vec<Menu>", permissions as "permissions: Vec<Permission>", privileged, enabled, created_at, updated_at
        FROM _roles
        WHERE ($1::text IS NULL OR name LIKE CONCAT('%', $1, '%'))
            AND ($2::boolean IS NULL OR privileged = $2)
            AND ($3::boolean IS NULL OR enabled = $3)
            AND ($4::integer IS NULL OR $4 = ANY(menus))
            AND ($5::integer IS NULL OR $5 = ANY(permissions))
        ORDER BY created_at DESC
        LIMIT $6 OFFSET $7
        "#,
            query.name,
            query.privileged,
            query.enabled,
            query.menu,
            query.permission,
            page_size,
            offset,
        )
        .fetch_all(&self.pool);
        let (total, rows) = tokio::try_join!(total_future, rows_future)?;
        Ok(PagingResult { total, items: rows })
    }
}

impl SearchRolesQueryHandler {
    #[tracing::instrument]
    pub async fn clean_cache(&self) -> ApplicationResult<()> {
        self.cache_provider.clear().await
    }

    #[tracing::instrument]
    pub async fn query_cached(
        &self,
        query: SearchRolesQuery,
    ) -> Result<PagingResult<RoleDto>, OrganizationError> {
        self.cache_provider.get_with(query, |q| self.query(q)).await
    }
}
