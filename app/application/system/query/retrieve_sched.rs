use crate::shared::query_handler::QueryHandler;
use crate::system::dto::sched::SchedDto;
use bon::Builder;
use domain::system::error::SystemError;
use domain::system::value_object::sched_id::SchedId;
use infrastructure::shared::pg_pool::PgPool;
use nject::injectable;
use serde::Deserialize;
use single_flight::single_flight;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Builder)]
pub struct RetrieveSchedQuery {
    id: SchedId,
}

#[derive(Debug)]
#[injectable]
pub struct RetrieveSchedQueryHandler {
    pool: PgPool,
}

impl QueryHandler for RetrieveSchedQueryHandler {
    type Query = RetrieveSchedQuery;
    type Output = SchedDto;
    type Error = SystemError;

    #[single_flight]
    #[tracing::instrument]
    async fn query(&self, query: RetrieveSchedQuery) -> Result<SchedDto, SystemError> {
        let row_opt = sqlx::query_as!(
            SchedDto,
            r#"
            SELECT id, key, name, schedule, succeed, result, run_at, duration_ms, created_at, updated_at
            FROM _scheds
            WHERE id = $1
        "#,
            &query.id,
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt.ok_or(SystemError::SchedNotFound)
    }
}
