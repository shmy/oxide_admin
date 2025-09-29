use bon::Builder;
use chrono::NaiveDateTime;
use domain::shared::event_util::UpdatedEvent;
use domain::shared::to_inner_vec::ToInnerVec;
use domain::system::value_object::role_id::RoleId;
use domain::{
    shared::port::domain_repository::DomainRepository,
    system::{
        entity::user::User,
        error::SystemError,
        port::user_repository::UserRepository,
        value_object::{hashed_password::HashedPassword, user_id::UserId},
    },
};
use nject::injectable;
use sqlx::prelude::FromRow;

use crate::shared::chrono_tz::ChronoTz;
use crate::shared::error_util::is_unique_constraint_error;
use crate::shared::pg_pool::PgPool;

#[derive(Debug, Builder)]
#[injectable]
pub struct UserRepositoryImpl {
    pool: PgPool,
    ct: ChronoTz,
}

impl DomainRepository for UserRepositoryImpl {
    type Entity = User;

    type EntityId = UserId;

    type Error = SystemError;

    #[tracing::instrument]
    async fn by_id(&self, id: &Self::EntityId) -> Result<Self::Entity, Self::Error> {
        let row_opt = sqlx::query_as!(
            UserDto,
            r#"
        SELECT id as "id: UserId", account, portrait, name, privileged, password as "password: HashedPassword", role_ids as "role_ids: Vec<RoleId>", enabled, refresh_token, refresh_token_expired_at
        FROM _users WHERE id = $1
        "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt.map(Into::into).ok_or(SystemError::UserNotFound)
    }

    #[tracing::instrument]
    async fn save(&self, entity: Self::Entity) -> Result<Self::Entity, Self::Error> {
        let now = self.ct.now();
        sqlx::query!(
            r#"
            INSERT INTO _users (id, account, portrait, name, privileged, password, role_ids, enabled, refresh_token, refresh_token_expired_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (id) DO UPDATE SET
                account = EXCLUDED.account,
                portrait = EXCLUDED.portrait,
                name = EXCLUDED.name,
                privileged = EXCLUDED.privileged,
                password = EXCLUDED.password,
                role_ids = EXCLUDED.role_ids,
                enabled = EXCLUDED.enabled,
                refresh_token = EXCLUDED.refresh_token,
                refresh_token_expired_at = EXCLUDED.refresh_token_expired_at,
                updated_at = EXCLUDED.updated_at
            "#,
            &entity.id,
            &entity.account,
            entity.portrait,
            &entity.name,
            &entity.privileged,
            &entity.password,
            &entity.role_ids.inner_vec(),
            &entity.enabled,
            entity.refresh_token,
            entity.refresh_token_expired_at,
            &now,
            &now
        )
        .execute(&self.pool)
        .await.map_err(|e| {
            if is_unique_constraint_error(&e, "_users", "account") {
                return SystemError::UserDuplicated;
            }
            SystemError::from(e)
        })?;
        Ok(entity)
    }

    #[tracing::instrument]
    async fn batch_delete(&self, ids: &[Self::EntityId]) -> Result<Vec<Self::Entity>, Self::Error> {
        if ids.is_empty() {
            return Ok(Vec::with_capacity(0));
        }

        let items = sqlx::query_as!(
            UserDto,
            r#"
            DELETE FROM _users WHERE id = ANY($1) AND privileged != true RETURNING id as "id: UserId", account, portrait, name, privileged, password as "password: HashedPassword", role_ids as "role_ids: Vec<RoleId>", enabled, refresh_token, refresh_token_expired_at
            "#,
            &ids.inner_vec()
        )
        .fetch_all(&self.pool)
        .await?;
        let items = items.into_iter().map(Into::into).collect();
        Ok(items)
    }
}

impl UserRepository for UserRepositoryImpl {
    #[tracing::instrument]
    async fn by_account(&self, account: String) -> Result<Self::Entity, Self::Error> {
        let row_opt = sqlx::query_as!(
            UserDto,
            r#"
        SELECT id as "id: UserId", account, portrait, name, privileged, password as "password: HashedPassword", role_ids as "role_ids: Vec<RoleId>", enabled, refresh_token, refresh_token_expired_at
        FROM _users WHERE account = $1
        "#,
            account
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt.map(Into::into).ok_or(SystemError::UserNotFound)
    }

    #[tracing::instrument]
    async fn by_refresh_token(&self, refresh_token: String) -> Result<Self::Entity, Self::Error> {
        let row_opt = sqlx::query_as!(
            UserDto,
            r#"
        SELECT id as "id: UserId", account, portrait, name, privileged, password as "password: HashedPassword", role_ids as "role_ids: Vec<RoleId>", enabled, refresh_token, refresh_token_expired_at
        FROM _users WHERE refresh_token = $1
        "#,
            refresh_token
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt.map(Into::into).ok_or(SystemError::UserNotFound)
    }

    #[tracing::instrument]
    async fn toggle_enabled(
        &self,
        ids: &[UserId],
        enabled: bool,
    ) -> Result<Vec<UpdatedEvent<Self::Entity>>, Self::Error> {
        if ids.is_empty() {
            return Ok(Vec::with_capacity(0));
        }
        let items = sqlx::query!(
            r#"
            WITH before AS (
                SELECT * FROM _users WHERE id = ANY($1) AND privileged != true
            ),
            updated AS (
                UPDATE _users SET enabled = $2
                WHERE id = ANY($1) AND privileged != true
                RETURNING *
            )
            SELECT
            before.id as "before_id: UserId", before.account as before_account, before.portrait as before_portrait, before.name as before_name, before.privileged as before_privileged, before.password as "before_password: HashedPassword", before.role_ids as "before_role_ids: Vec<RoleId>", before.enabled as before_enabled, before.refresh_token as before_refresh_token, before.refresh_token_expired_at as before_refresh_token_expired_at,
            updated.id as "updated_id: UserId", updated.account as updated_account, updated.portrait as updated_portrait, updated.name as updated_name, updated.privileged as updated_privileged, updated.password as "updated_password: HashedPassword", updated.role_ids as "updated_role_ids: Vec<RoleId>", updated.enabled as updated_enabled, updated.refresh_token as updated_refresh_token, updated.refresh_token_expired_at as updated_refresh_token_expired_at
            FROM before
            JOIN updated ON before.id = updated.id;
            "#,
            &ids.inner_vec(),
            enabled,
        )
        .fetch_all(&self.pool)
        .await?;
        let items = items
            .into_iter()
            .map(|row| UpdatedEvent {
                before: User::builder()
                    .id(row.before_id)
                    .account(row.before_account)
                    .maybe_portrait(row.before_portrait)
                    .privileged(row.before_privileged)
                    .name(row.before_name)
                    .password(row.before_password)
                    .role_ids(row.before_role_ids)
                    .enabled(row.before_enabled)
                    .maybe_refresh_token(row.before_refresh_token)
                    .maybe_refresh_token_expired_at(row.before_refresh_token_expired_at)
                    .build(),
                after: User::builder()
                    .id(row.updated_id)
                    .account(row.updated_account)
                    .maybe_portrait(row.updated_portrait)
                    .privileged(row.updated_privileged)
                    .name(row.updated_name)
                    .password(row.updated_password)
                    .role_ids(row.updated_role_ids)
                    .enabled(row.updated_enabled)
                    .maybe_refresh_token(row.updated_refresh_token)
                    .maybe_refresh_token_expired_at(row.updated_refresh_token_expired_at)
                    .build(),
            })
            .collect();
        Ok(items)
    }
}

#[derive(FromRow)]
struct UserDto {
    id: UserId,
    account: String,
    portrait: Option<String>,
    name: String,
    privileged: bool,
    password: HashedPassword,
    role_ids: Vec<RoleId>,
    enabled: bool,
    refresh_token: Option<String>,
    refresh_token_expired_at: Option<NaiveDateTime>,
}

impl From<UserDto> for User {
    fn from(value: UserDto) -> Self {
        Self::builder()
            .id(value.id)
            .account(value.account)
            .maybe_portrait(value.portrait)
            .privileged(value.privileged)
            .name(value.name)
            .password(value.password)
            .role_ids(value.role_ids)
            .enabled(value.enabled)
            .maybe_refresh_token(value.refresh_token)
            .maybe_refresh_token_expired_at(value.refresh_token_expired_at)
            .build()
    }
}

#[cfg(test)]
mod tests {

    use crate::test_utils::setup_database;

    use super::*;

    async fn build_user_repository(pool: PgPool) -> UserRepositoryImpl {
        setup_database(pool.clone()).await;
        let ct = ChronoTz::default();
        UserRepositoryImpl::builder().pool(pool).ct(ct).build()
    }

    #[sqlx::test]
    async fn test_create_and_fetch(pool: PgPool) {
        let user_repository = build_user_repository(pool.clone()).await;
        let id = UserId::generate();
        let user = User::builder()
            .id(id.clone())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(false)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .refresh_token("refresh_token".to_string())
            .enabled(true)
            .build();
        assert!(user_repository.save(user).await.is_ok());
        let user = user_repository.by_id(&id).await.unwrap();
        assert_eq!(user.id, id);
        assert_eq!(user.account, "test");
        assert_eq!(user.name, "test");
        assert_eq!(user.privileged, false);
        assert_eq!(user.role_ids, vec![]);
        assert_eq!(user.enabled, true);

        let user = user_repository
            .by_account("test".to_string())
            .await
            .unwrap();
        assert_eq!(user.id, id);
        assert_eq!(user.account, "test");
        assert_eq!(user.name, "test");
        assert_eq!(user.privileged, false);
        assert_eq!(user.role_ids, vec![]);
        assert_eq!(user.enabled, true);

        let user = user_repository
            .by_refresh_token("refresh_token".to_string())
            .await
            .unwrap();
        assert_eq!(user.id, id);
        assert_eq!(user.account, "test");
        assert_eq!(user.name, "test");
        assert_eq!(user.privileged, false);
        assert_eq!(user.role_ids, vec![]);
        assert_eq!(user.enabled, true);
    }

    #[sqlx::test]
    async fn test_toggle_enabled(pool: PgPool) {
        let user_repository = build_user_repository(pool.clone()).await;
        let id = UserId::generate();
        let user = User::builder()
            .id(id.clone())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(false)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .refresh_token("refresh_token".to_string())
            .enabled(true)
            .build();
        assert!(user_repository.save(user).await.is_ok());
        let user = user_repository.by_id(&id).await.unwrap();
        assert_eq!(user.id, id);
        assert_eq!(user.account, "test");
        assert_eq!(user.name, "test");
        assert_eq!(user.privileged, false);
        assert_eq!(user.role_ids, vec![]);
        assert_eq!(user.enabled, true);
        assert!(
            user_repository
                .toggle_enabled(&[id.clone()], false)
                .await
                .is_ok()
        );
        let user = user_repository.by_id(&id).await.unwrap();
        assert_eq!(user.id, id);
        assert_eq!(user.account, "test");
        assert_eq!(user.name, "test");
        assert_eq!(user.privileged, false);
        assert_eq!(user.role_ids, vec![]);
        assert_eq!(user.enabled, false);
    }

    #[sqlx::test]
    async fn test_batch_delete(pool: PgPool) {
        #[derive(FromRow)]
        struct UserRow {
            id: UserId,
        }
        let user_repository = build_user_repository(pool.clone()).await;
        let user1 = User::builder()
            .id(UserId::generate())
            .account("test1".to_string())
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .name("Test".to_string())
            .enabled(true)
            .privileged(false)
            .role_ids(vec![])
            .build();
        let user2 = User::builder()
            .id(UserId::generate())
            .account("test2".to_string())
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .name("Test2".to_string())
            .enabled(true)
            .privileged(false)
            .role_ids(vec![])
            .build();
        assert!(user_repository.save(user1).await.is_ok());
        assert!(user_repository.save(user2).await.is_ok());
        let rows: Vec<UserRow> = sqlx::query_as(r#"SELECT id from _users"#)
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(rows.len(), 3); // because the have a privileged user
        let ids = rows.into_iter().map(|row| row.id).collect::<Vec<_>>();
        assert!(user_repository.batch_delete(&ids).await.is_ok());
        let rows: Vec<UserRow> = sqlx::query_as(r#"SELECT id from _users"#)
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1); // because privileged user cannot be deleted
    }

    #[sqlx::test]
    async fn test_duplicated_account(pool: PgPool) {
        let user_repository = build_user_repository(pool.clone()).await;
        let user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(false)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(true)
            .build();
        assert!(user_repository.save(user).await.is_ok());
        let user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(false)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(true)
            .build();
        assert_eq!(
            user_repository.save(user).await.err(),
            Some(SystemError::UserDuplicated)
        );
    }
}
