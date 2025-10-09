use crate::shared::query_handler::QueryHandler;
use crate::{
    shared::{paging_query::PagingQuery, paging_result::PagingResult},
    system::dto::access_log::AccessLogDto,
};
use bon::Builder;
use domain::system::error::SystemError;
use infrastructure::shared::pg_pool::PgPool;
use nject::injectable;
use serde::Deserialize;
use serde_with::serde_as;
use single_flight::single_flight;
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
        SELECT a.id as id,
            a.user_id as user_id,
            a.method as method, 
            a.uri as uri,
            a.user_agent as user_agent,
            a.ip as ip,
            a.status as status,
            a.elapsed as elapsed, 
            a.occurred_at as occurred_at, 
            a.created_at as created_at, 
            a.updated_at as updated_at,
            u.name as "user_name?"
        FROM _access_logs as a
        LEFT JOIN _users as u ON u.id = a.user_id
        ORDER BY a.occurred_at DESC
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
