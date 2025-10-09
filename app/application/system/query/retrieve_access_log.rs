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
            WHERE a.id = $1
        "#,
            &query.id,
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt.ok_or(SystemError::AccessLogNotFound)
    }
}
