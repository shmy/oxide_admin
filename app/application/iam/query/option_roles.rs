use domain::iam::error::IamError;
use infrastructure::shared::pg_pool::PgPool;
use nject::injectable;
use single_flight::single_flight;

use crate::shared::dto::OptionDto;

#[derive(Debug)]
#[injectable]
pub struct OptionRolesQueryHandler {
    pool: PgPool,
}

impl OptionRolesQueryHandler {
    #[single_flight]
    #[tracing::instrument]
    pub async fn query(&self) -> Result<Vec<OptionDto>, IamError> {
        let options = sqlx::query_as!(
            OptionDto::<String>,
            r#"
        SELECT name as label, id as value FROM _roles ORDER BY updated_at DESC
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(options)
    }
}
