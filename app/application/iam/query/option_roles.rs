use infrastructure::shared::{cloneable_error::CloneableError, pool::Pool};
use nject::injectable;
use single_flight::single_flight;

use crate::shared::dto::OptionDto;

#[injectable]
pub struct OptionRolesQueryHandler {
    pool: Pool,
}

impl OptionRolesQueryHandler {
    #[single_flight]
    pub async fn query(&self) -> Result<Vec<OptionDto>, CloneableError> {
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
