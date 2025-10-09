use crate::shared::query_handler::QueryHandler;
use crate::{
    error::ApplicationResult,
    shared::{
        cache_provider::CacheProvider, paging_query::PagingQuery, paging_result::PagingResult,
    },
    system::dto::access_log::AccessLogDto,
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
pub struct SearchAccessLogsQuery {
    #[serde(flatten)]
    #[param(inline)]
    paging: PagingQuery,
}

#[derive(Debug, Clone)]
#[injectable]
pub struct SearchAccessLogsQueryHandler {
    pool: PgPool,
    #[inject(|kvdb: Kvdb| CacheProvider::builder().key("system_search_access_logs:").ttl(Duration::from_secs(15 * 60)).kvdb(kvdb).build())]
    cache_provider: CacheProvider,
}

impl QueryHandler for SearchAccessLogsQueryHandler {
    type Query = SearchAccessLogsQuery;
    type Output = PagingResult<AccessLogDto>;
    type Error = SystemError;

    #[single_flight]
    #[tracing::instrument]
    async fn query(
        &self,
        query: SearchAccessLogsQuery,
    ) -> Result<PagingResult<AccessLogDto>, SystemError> {
        let total_future = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM _access_logs
            "#,
        )
        .fetch_one(&self.pool);
        let page = query.paging.page();
        let page_size = query.paging.page_size();
        let offset = (page - 1) * page_size;
        let rows_future = sqlx::query_as!(
            AccessLogDto,
            r#"
        SELECT id, user_id, method, uri, user_agent, ip, ip_region, status, elapsed, created_at, updated_at
        FROM _access_logs
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

impl SearchAccessLogsQueryHandler {
    #[tracing::instrument]
    pub async fn clean_cache(&self) -> ApplicationResult<()> {
        self.cache_provider.clear().await
    }

    #[tracing::instrument]
    pub async fn query_cached(
        &self,
        query: SearchAccessLogsQuery,
    ) -> Result<PagingResult<AccessLogDto>, SystemError> {
        self.cache_provider.get_with(query, |q| self.query(q)).await
    }
}
