use bon::Builder;
use cache_kit::{Cache, cached_impl};
use domain::{
    auth::value_object::{menu::Menu, permission::Permission},
    organization::error::OrganizationError,
};
use infrastructure::shared::pg_pool::PgPool;
use nject::injectable;
use serde::Deserialize;
use serde_with::{NoneAsEmptyString, serde_as};
use single_flight::single_flight;
use utoipa::IntoParams;

use crate::{
    organization::dto::role::RoleDto,
    shared::{paging_query::PagingQuery, paging_result::PagingResult, query_handler::QueryHandler},
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
    cache: Cache,
}

#[cached_impl]
impl QueryHandler for SearchRolesQueryHandler {
    type Query = SearchRolesQuery;
    type Output = PagingResult<RoleDto>;
    type Error = OrganizationError;

    #[tracing::instrument]
    #[single_flight]
    #[cached(prefix = "organization:search_roles:", ttl = "30min")]
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
