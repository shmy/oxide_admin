use crate::shared::query_handler::QueryHandler;
use crate::system::dto::access_log::AccessLogDto;
use bon::Builder;
use domain::system::error::SystemError;
use domain::system::value_object::access_log_id::AccessLogId;
use infrastructure::shared::pg_pool::PgPool;
use nject::injectable;
use serde::Deserialize;
use single_flight::single_flight;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Builder)]
pub struct RetrieveAccessLogQuery {
    id: AccessLogId,
}

#[derive(Debug)]
#[injectable]
pub struct RetrieveAccessLogQueryHandler {
    pool: PgPool,
}

impl QueryHandler for RetrieveAccessLogQueryHandler {
    type Query = RetrieveAccessLogQuery;
    type Output = AccessLogDto;
    type Error = SystemError;

    #[single_flight]
    #[tracing::instrument]
    async fn query(&self, query: RetrieveAccessLogQuery) -> Result<AccessLogDto, SystemError> {
        let row_opt = sqlx::query_as!(
            AccessLogDto,
            r#"
            SELECT id, user_id, method, uri, user_agent, ip, status, elapsed, occurred_at, created_at, updated_at
            FROM _access_logs
            WHERE id = $1
        "#,
            &query.id,
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt.ok_or(SystemError::AccessLogNotFound)
    }
}
