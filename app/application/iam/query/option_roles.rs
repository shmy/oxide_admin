use bon::Builder;
use domain::iam::error::IamError;
use infrastructure::shared::pg_pool::PgPool;
use nject::injectable;
use single_flight::single_flight;

use crate::shared::{dto::OptionStringDto, query_handler::QueryHandler};

#[derive(Debug, Builder)]
#[injectable]
pub struct OptionRolesQueryHandler {
    pool: PgPool,
}

impl QueryHandler for OptionRolesQueryHandler {
    type Query = ();
    type Output = Vec<OptionStringDto>;
    type Error = IamError;

    #[single_flight]
    #[tracing::instrument]
    async fn query(&self, _query: ()) -> Result<Vec<OptionStringDto>, IamError> {
        let options = sqlx::query_as!(
            OptionStringDto,
            r#"
        SELECT name as label, id as value FROM _roles ORDER BY updated_at DESC
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use infrastructure::test_utils::setup_database;

    use super::*;

    #[sqlx::test]
    async fn test_option_roles(pool: PgPool) {
        setup_database(pool.clone()).await;
        let handler = OptionRolesQueryHandler::builder().pool(pool).build();
        let result = handler.query(()).await;
        assert!(result.is_ok());
    }

    #[sqlx::test]
    async fn test_option_roles_return_err_given_pool_is_closed(pool: PgPool) {
        setup_database(pool.clone()).await;
        pool.close().await;
        let handler = OptionRolesQueryHandler::builder().pool(pool).build();
        let result = handler.query(()).await;
        assert!(result.is_err());
    }
}
