use crate::iam::dto::role::RoleDto;
use crate::shared::query_handler::QueryHandler;
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::value_object::menu::Menu;
use domain::iam::value_object::permission::Permission;
use domain::iam::value_object::role_id::RoleId;
use infrastructure::shared::pg_pool::PgPool;
use nject::injectable;
use serde::Deserialize;
use single_flight::single_flight;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Builder)]
pub struct RetrieveRoleQuery {
    id: RoleId,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct RetrieveRoleQueryHandler {
    pool: PgPool,
}

impl QueryHandler for RetrieveRoleQueryHandler {
    type Query = RetrieveRoleQuery;
    type Output = RoleDto;
    type Error = IamError;

    #[single_flight]
    async fn query(&self, query: RetrieveRoleQuery) -> Result<RoleDto, IamError> {
        let row_opt = sqlx::query_as!(
            RoleDto,
            r#"
            SELECT id, name, menus as "menus: Vec<Menu>", permissions as "permissions: Vec<Permission>", privileged, enabled, created_at, updated_at
            FROM _roles
            WHERE id = $1
            LIMIT 1
        "#,
            &query.id,
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt.ok_or(IamError::RoleNotFound)
    }
}
