use crate::shared::query_handler::QueryHandler;
use crate::{
    error::ApplicationResult,
    shared::{
        cache_provider::CacheProvider, paging_query::PagingQuery, paging_result::PagingResult,
    },
    system::dto::sched::SchedDto,
};
use bon::Builder;
use domain::system::error::SystemError;
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
pub struct SearchSchedsQuery {
    #[serde(flatten)]
    #[param(inline)]
    paging: PagingQuery,
}

#[derive(Debug, Clone)]
#[injectable]
pub struct SearchSchedsQueryHandler {
    pool: PgPool,
    #[inject(|kvdb: Kvdb| CacheProvider::builder().key("system_search_scheds:").ttl(Duration::from_secs(15 * 60)).kvdb(kvdb).build())]
    cache_provider: CacheProvider,
}

impl QueryHandler for SearchSchedsQueryHandler {
    type Query = SearchSchedsQuery;
    type Output = PagingResult<SchedDto>;
    type Error = SystemError;

    #[single_flight]
    #[tracing::instrument]
    async fn query(&self, query: SearchSchedsQuery) -> Result<PagingResult<SchedDto>, SystemError> {
        let total_future = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM _scheds
            "#,
        )
        .fetch_one(&self.pool);
        let page = query.paging.page();
        let page_size = query.paging.page_size();
        let offset = (page - 1) * page_size;
        let rows_future = sqlx::query_as!(
            SchedDto,
            r#"
        SELECT id, key, name, schedule, succeed, result, run_at, duration_ms, created_at, updated_at
        FROM _scheds
        ORDER BY created_at DESC LIMIT $1 OFFSET $2
        "#,
            page_size,
            offset,
        )
        .fetch_all(&self.pool);
        let (total, rows) = tokio::try_join!(total_future, rows_future)?;
        Ok(PagingResult { total, items: rows })
    }
}

impl SearchSchedsQueryHandler {
    #[tracing::instrument]
    pub async fn clean_cache(&self) -> ApplicationResult<()> {
        self.cache_provider.clear().await
    }

    #[tracing::instrument]
    pub async fn query_cached(
        &self,
        query: SearchSchedsQuery,
    ) -> Result<PagingResult<SchedDto>, SystemError> {
        self.cache_provider.get_with(query, |q| self.query(q)).await
    }
}
