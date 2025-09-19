use crate::iam::dto::user::UserDto;
use crate::shared::query_handler::QueryHandler;
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

#[derive(Debug, Builder)]
#[injectable]
pub struct RetrieveUserQueryHandler {
    pool: PgPool,
}

impl QueryHandler for RetrieveUserQueryHandler {
    type Query = RetrieveUserQuery;
    type Output = UserDto;
    type Error = IamError;

    #[single_flight]
    #[tracing::instrument]
    async fn query(&self, query: RetrieveUserQuery) -> Result<UserDto, IamError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use domain::iam::entity::user::User;
    use domain::iam::value_object::hashed_password::HashedPassword;
    use domain::shared::port::domain_repository::DomainRepository;
    use infrastructure::{
        repository::iam::user_repository::UserRepositoryImpl, shared::chrono_tz::ChronoTz,
        test_utils::setup_database,
    };

    #[sqlx::test]
    async fn test_retrieve_user(pool: PgPool) {
        setup_database(pool.clone()).await;
        let user_repository = UserRepositoryImpl::builder()
            .pool(pool.clone())
            .ct(ChronoTz::default())
            .build();
        let user_id = UserId::generate();
        let user = User::builder()
            .id(user_id.clone())
            .account("test".to_string())
            .name("Test".to_string())
            .password(HashedPassword::try_new("123123".to_string()).unwrap())
            .privileged(false)
            .role_ids(vec![])
            .enabled(true)
            .build();
        assert!(user_repository.save(user).await.is_ok());
        let handler = RetrieveUserQueryHandler::builder()
            .pool(pool.clone())
            .build();

        assert!(
            handler
                .query(RetrieveUserQuery::builder().id(user_id).build())
                .await
                .is_ok()
        );
    }

    #[sqlx::test]
    async fn test_retrieve_user_return_err_given_role_not_found(pool: PgPool) {
        setup_database(pool.clone()).await;
        let handler = RetrieveUserQueryHandler::builder().pool(pool).build();
        let result = handler
            .query(RetrieveUserQuery::builder().id(UserId::generate()).build())
            .await;
        assert_eq!(result.err(), Some(IamError::UserNotFound));
    }

    #[sqlx::test]
    async fn test_retrieve_user_return_err_given_pool_is_closed(pool: PgPool) {
        setup_database(pool.clone()).await;
        pool.close().await;
        let handler = RetrieveUserQueryHandler::builder().pool(pool).build();
        let result = handler
            .query(RetrieveUserQuery::builder().id(UserId::generate()).build())
            .await;
        assert!(result.is_err());
    }
}
