use crate::iam::dto::role::RoleDto;
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::value_object::permission_code::PermissionCode;
use domain::iam::value_object::role_id::RoleId;
use infrastructure::shared::{cloneable_error::CloneableError, pg_pool::PgPool};
use nject::injectable;
use serde::Deserialize;
use single_flight::single_flight;

#[derive(Clone, PartialEq, Eq, Hash, Deserialize, Builder)]
pub struct RetrieveRoleQuery {
    id: RoleId,
}

#[injectable]
pub struct RetrieveRoleQueryHandler {
    pool: PgPool,
}

impl RetrieveRoleQueryHandler {
    #[single_flight]
    pub async fn query(&self, query: RetrieveRoleQuery) -> Result<RoleDto, CloneableError> {
        let row_opt = sqlx::query_as!(
            RoleDto,
            r#"
            SELECT id, name, permission_ids as "permission_ids: Vec<PermissionCode>", privileged, enabled, created_at, updated_at
            FROM _roles
            WHERE id = $1
            LIMIT 1
        "#,
            &query.id,
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt.ok_or(CloneableError::from(anyhow::anyhow!(
            IamError::RoleNotFound
        )))
    }
}
