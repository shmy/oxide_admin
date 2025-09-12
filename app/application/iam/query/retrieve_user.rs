use crate::iam::dto::user::UserDto;
use bon::Builder;
use domain::iam::value_object::user_id::UserId;
use domain::iam::{error::IamError, value_object::role_id::RoleId};
use infrastructure::shared::pg_pool::PgPool;
use nject::injectable;
use serde::Deserialize;
use single_flight::single_flight;

#[derive(Clone, PartialEq, Eq, Hash, Deserialize, Builder)]
pub struct RetrieveUserQuery {
    id: UserId,
}

#[derive(Debug)]
#[injectable]
pub struct RetrieveUserQueryHandler {
    pool: PgPool,
}

impl RetrieveUserQueryHandler {
    #[single_flight]
    #[tracing::instrument]
    pub async fn query(&self, query: RetrieveUserQuery) -> Result<UserDto, IamError> {
        let row_opt = sqlx::query_as!(
            UserDto,
            r#"
        SELECT
            u.id as id,
            u.account as account,
            u.portrait as portrait,
            u.name as name,
            u.role_ids as "role_ids: Vec<RoleId>",
            u.privileged as privileged,
            u.enabled as enabled,
            u.created_at as created_at,
            u.updated_at as updated_at,
            COALESCE(array_agg(r.name) FILTER (WHERE r.name IS NOT NULL), '{}') as "role_names!: Vec<String>"
        FROM _users as u
        LEFT JOIN _roles as r ON r.id = ANY(u.role_ids)
        WHERE u.id = $1
        GROUP BY u.id
        LIMIT 1
        "#,
            &query.id,
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt.ok_or(IamError::UserNotFound)
    }
}
