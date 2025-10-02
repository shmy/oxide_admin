use crate::shared::paging_query::PagingQuery;
use crate::shared::paging_result::PagingResult;
use crate::shared::query_handler::QueryHandler;
use crate::system::dto::sched::SchedRecordDto;
use bon::Builder;
use domain::system::error::SystemError;
use infrastructure::shared::pg_pool::PgPool;
use nject::injectable;
use serde::Deserialize;
use serde_with::serde_as;
use utoipa::IntoParams;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, IntoParams, Builder)]
pub struct PagingSchedRecordsQuery {
    #[serde(flatten)]
    #[param(inline)]
    paging: PagingQuery,
    key: String,
}

#[derive(Debug, Clone)]
#[injectable]
pub struct PagingSchedRecordsQueryHandler {
    pool: PgPool,
}

impl QueryHandler for PagingSchedRecordsQueryHandler {
    type Query = PagingSchedRecordsQuery;
    type Output = PagingResult<SchedRecordDto>;
    type Error = SystemError;

    #[tracing::instrument]
    async fn query(
        &self,
        query: PagingSchedRecordsQuery,
    ) -> Result<PagingResult<SchedRecordDto>, SystemError> {
        let total_future = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM _scheds
            WHERE ($1::text IS NULL OR key = $1)
            "#,
            query.key,
        )
        .fetch_one(&self.pool);
        let page = query.paging.page();
        let page_size = query.paging.page_size();
        let offset = (page - 1) * page_size;
        let rows_future = sqlx::query_as!(
            SchedRecordDto,
            r#"
        SELECT id, key, name, expr, succeed, result, run_at, duration_ms, created_at, updated_at
        FROM _scheds
        WHERE ($1::text IS NULL OR key = $1)
        ORDER BY run_at DESC LIMIT $2 OFFSET $3
        "#,
            query.key,
            page_size,
            offset,
        )
        .fetch_all(&self.pool);
        let (total, rows) = tokio::try_join!(total_future, rows_future)?;
        Ok(PagingResult { total, items: rows })
    }
}
