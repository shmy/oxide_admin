use crate::iam::dto::role::RoleDto;
use crate::shared::query_handler::QueryHandler;
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::value_object::permission_code::PermissionCode;
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
            SELECT id, name, permission_ids as "permission_ids: Vec<PermissionCode>", privileged, enabled, created_at, updated_at
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

#[cfg(test)]
mod tests {
    use super::*;
    use domain::iam::entity::role::Role;
    use domain::shared::port::domain_repository::DomainRepository;
    use infrastructure::{
        repository::iam::role_repository::RoleRepositoryImpl, shared::chrono_tz::ChronoTz,
        test_utils::setup_database,
    };

    #[sqlx::test]
    async fn test_retrieve_role(pool: PgPool) {
        setup_database(pool.clone()).await;
        let role_repository = RoleRepositoryImpl::builder()
            .pool(pool.clone())
            .ct(ChronoTz::default())
            .build();
        let role_id = RoleId::generate();
        let role = Role::builder()
            .id(role_id.clone())
            .name("Test".to_string())
            .privileged(false)
            .permission_ids(vec![])
            .enabled(true)
            .build();
        assert!(role_repository.save(role).await.is_ok());
        let handler = RetrieveRoleQueryHandler::builder()
            .pool(pool.clone())
            .build();

        assert!(
            handler
                .query(RetrieveRoleQuery::builder().id(role_id).build())
                .await
                .is_ok()
        );
    }

    #[sqlx::test]
    async fn test_retrieve_role_return_err_given_role_not_found(pool: PgPool) {
        setup_database(pool.clone()).await;
        let handler = RetrieveRoleQueryHandler::builder().pool(pool).build();
        let result = handler
            .query(RetrieveRoleQuery::builder().id(RoleId::generate()).build())
            .await;
        assert_eq!(result.err(), Some(IamError::RoleNotFound));
    }

    #[sqlx::test]
    async fn test_retrieve_role_return_err_given_pool_is_closed(pool: PgPool) {
        setup_database(pool.clone()).await;
        pool.close().await;
        let handler = RetrieveRoleQueryHandler::builder().pool(pool).build();
        let result = handler
            .query(RetrieveRoleQuery::builder().id(RoleId::generate()).build())
            .await;
        assert!(result.is_err());
    }
}
