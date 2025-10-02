use crate::shared::query_handler::QueryHandler;
use crate::{
    error::ApplicationResult,
    organization::dto::department::DepartmentDto,
    shared::{
        cache_provider::CacheProvider, paging_query::PagingQuery, paging_result::PagingResult,
    },
};
use bon::Builder;
use domain::organization::error::OrganizationError;
use infrastructure::shared::pg_pool::PgPool;
use kvdb_kit::Kvdb;
use nject::injectable;
use serde::Deserialize;
use serde_with::serde_as;
use single_flight::single_flight;
use std::time::Duration;
use utoipa::IntoParams;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, IntoParams, Builder)]
pub struct SearchDepartmentsQuery {
    #[serde(flatten)]
    #[param(inline)]
    paging: PagingQuery,
}

#[derive(Debug, Clone)]
#[injectable]
pub struct SearchDepartmentsQueryHandler {
    pool: PgPool,
    #[inject(|kvdb: Kvdb| CacheProvider::builder().key("organization_search_departments:").ttl(Duration::from_secs(15 * 60)).kvdb(kvdb).build())]
    cache_provider: CacheProvider,
}

impl QueryHandler for SearchDepartmentsQueryHandler {
    type Query = SearchDepartmentsQuery;
    type Output = PagingResult<DepartmentDto>;
    type Error = OrganizationError;

    #[single_flight]
    #[tracing::instrument]
    async fn query(
        &self,
        query: SearchDepartmentsQuery,
    ) -> Result<PagingResult<DepartmentDto>, OrganizationError> {
        let total_future = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM _departments
            "#,
        )
        .fetch_one(&self.pool);
        let page = query.paging.page();
        let page_size = query.paging.page_size();
        let offset = (page - 1) * page_size;
        let rows_future = sqlx::query_as!(
            DepartmentDto,
            r#"
        SELECT id, name, code, parent_id, created_at, updated_at
        FROM _departments
        LIMIT $1 OFFSET $2
        "#,
            page_size,
            offset,
        )
        .fetch_all(&self.pool);
        let (total, rows) = tokio::try_join!(total_future, rows_future)?;
        Ok(PagingResult { total, items: rows })
    }
}

impl SearchDepartmentsQueryHandler {
    #[tracing::instrument]
    pub async fn clean_cache(&self) -> ApplicationResult<()> {
        self.cache_provider.clear().await
    }

    #[tracing::instrument]
    pub async fn query_cached(
        &self,
        query: SearchDepartmentsQuery,
    ) -> Result<PagingResult<DepartmentDto>, OrganizationError> {
        self.cache_provider.get_with(query, |q| self.query(q)).await
    }
}
