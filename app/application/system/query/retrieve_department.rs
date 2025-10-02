use crate::shared::query_handler::QueryHandler;
use crate::system::dto::department::DepartmentDto;
use bon::Builder;
use domain::system::error::SystemError;
use domain::system::value_object::department_id::DepartmentId;
use infrastructure::shared::pg_pool::PgPool;
use nject::injectable;
use serde::Deserialize;
use single_flight::single_flight;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Builder)]
pub struct RetrieveDepartmentQuery {
    id: DepartmentId,
}

#[derive(Debug)]
#[injectable]
pub struct RetrieveDepartmentQueryHandler {
    pool: PgPool,
}

impl QueryHandler for RetrieveDepartmentQueryHandler {
    type Query = RetrieveDepartmentQuery;
    type Output = DepartmentDto;
    type Error = SystemError;

    #[single_flight]
    #[tracing::instrument]
    async fn query(&self, query: RetrieveDepartmentQuery) -> Result<DepartmentDto, SystemError> {
        let row_opt = sqlx::query_as!(
            DepartmentDto,
            r#"
            SELECT id, name, code, parent_id, created_at, updated_at
            FROM _departments
            WHERE id = $1
        "#,
            &query.id,
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt.ok_or(SystemError::DepartmentNotFound)
    }
}
