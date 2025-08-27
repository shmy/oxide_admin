use anyhow::Result;
use domain::iam::value_object::role_id::RoleId;
use domain::shared::event_util::UpdatedEvent;
use domain::shared::to_inner_vec::ToInnerVec;
use domain::{
    iam::{
        entity::user::User,
        error::IamError,
        repository::user_repository::UserRepository,
        value_object::{hashed_password::HashedPassword, user_id::UserId},
    },
    shared::domain_repository::DomainRepository,
};
use nject::injectable;

use crate::shared::chrono_tz::ChronoTz;
use crate::shared::error_util::is_unique_constraint_error;
use crate::shared::pool::Pool;

#[injectable]
pub struct UserRepositoryImpl {
    pool: Pool,
    ct: ChronoTz,
}

impl DomainRepository for UserRepositoryImpl {
    type Entity = User;

    type EntityId = UserId;

    type Error = IamError;

    async fn by_id(&self, id: &Self::EntityId) -> Result<Self::Entity, IamError> {
        let row_opt = sqlx::query!(
            r#"
        SELECT id, account, portrait, name, privileged, password, role_ids as "role_ids: Vec<RoleId>", enabled, refresh_token, refresh_token_expired_at
        FROM _users WHERE id = $1
        "#,
            id
        )
        .fetch_optional(&self.pool)
        .await.map_err(IamError::DatabaseError)?;
        row_opt
            .map(|row| {
                User::builder()
                    .id(UserId::new_unchecked(row.id))
                    .account(row.account)
                    .maybe_portrait(row.portrait)
                    .privileged(row.privileged)
                    .name(row.name)
                    .password(HashedPassword::new_unchecked(row.password))
                    .role_ids(row.role_ids)
                    .enabled(row.enabled)
                    .maybe_refresh_token(row.refresh_token)
                    .maybe_refresh_token_expired_at(row.refresh_token_expired_at)
                    .build()
            })
            .ok_or(IamError::UserNotFound)
    }

    async fn save(&self, entity: Self::Entity) -> Result<Self::Entity, IamError> {
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
                return IamError::UserDuplicated;
            }
            IamError::DatabaseError(e)
        })?;
        Ok(entity)
    }

    async fn batch_delete(
        &self,
        ids: &[Self::EntityId],
    ) -> std::result::Result<Vec<Self::Entity>, Self::Error> {
        if ids.is_empty() {
            return Ok(Vec::with_capacity(0));
        }

        let items = sqlx::query!(
            r#"
            DELETE FROM _users WHERE id = ANY($1) AND privileged != true RETURNING id, account, portrait, name, privileged, password, role_ids as "role_ids: Vec<RoleId>", enabled, refresh_token, refresh_token_expired_at
            "#,
            &ids.inner_vec()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(IamError::DatabaseError)?;
        let items = items
            .into_iter()
            .map(|row| {
                User::builder()
                    .id(UserId::new_unchecked(row.id))
                    .account(row.account)
                    .maybe_portrait(row.portrait)
                    .privileged(row.privileged)
                    .name(row.name)
                    .password(HashedPassword::new_unchecked(row.password))
                    .role_ids(row.role_ids)
                    .enabled(row.enabled)
                    .maybe_refresh_token(row.refresh_token)
                    .maybe_refresh_token_expired_at(row.refresh_token_expired_at)
                    .build()
            })
            .collect();
        Ok(items)
    }
}

impl UserRepository for UserRepositoryImpl {
    async fn by_account(&self, account: String) -> Result<Self::Entity, Self::Error> {
        let row_opt = sqlx::query!(
            r#"
        SELECT id, account, portrait, name, privileged, password, role_ids as "role_ids: Vec<RoleId>", enabled, refresh_token, refresh_token_expired_at
        FROM _users WHERE account = $1
        "#,
            account
        )
        .fetch_optional(&self.pool)
        .await.map_err(IamError::DatabaseError)?;
        row_opt
            .map(|row| {
                User::builder()
                    .id(UserId::new_unchecked(row.id))
                    .account(row.account)
                    .maybe_portrait(row.portrait)
                    .privileged(row.privileged)
                    .name(row.name)
                    .password(HashedPassword::new_unchecked(row.password))
                    .role_ids(row.role_ids)
                    .enabled(row.enabled)
                    .maybe_refresh_token(row.refresh_token)
                    .maybe_refresh_token_expired_at(row.refresh_token_expired_at)
                    .build()
            })
            .ok_or(IamError::UserNotFound)
    }

    async fn by_refresh_token(&self, refresh_token: String) -> Result<Self::Entity, Self::Error> {
        let row_opt = sqlx::query!(
            r#"
        SELECT id, account, portrait, name, privileged, password, role_ids as "role_ids: Vec<RoleId>", enabled, refresh_token, refresh_token_expired_at
        FROM _users WHERE refresh_token = $1
        "#,
            refresh_token
        )
        .fetch_optional(&self.pool)
        .await.map_err(IamError::DatabaseError)?;
        row_opt
            .map(|row| {
                User::builder()
                    .id(UserId::new_unchecked(row.id))
                    .account(row.account)
                    .maybe_portrait(row.portrait)
                    .privileged(row.privileged)
                    .name(row.name)
                    .password(HashedPassword::new_unchecked(row.password))
                    .role_ids(row.role_ids)
                    .enabled(row.enabled)
                    .maybe_refresh_token(row.refresh_token)
                    .maybe_refresh_token_expired_at(row.refresh_token_expired_at)
                    .build()
            })
            .ok_or(IamError::UserNotFound)
    }

    async fn toggle_enabled(
        &self,
        ids: &[UserId],
        enabled: bool,
    ) -> std::result::Result<Vec<UpdatedEvent<Self::Entity>>, Self::Error> {
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
            before.id as before_id, before.account as before_account, before.portrait as before_portrait, before.name as before_name, before.privileged as before_privileged, before.password as before_password, before.role_ids as "before_role_ids: Vec<RoleId>", before.enabled as before_enabled, before.refresh_token as before_refresh_token, before.refresh_token_expired_at as before_refresh_token_expired_at,
            updated.id as updated_id, updated.account as updated_account, updated.portrait as updated_portrait, updated.name as updated_name, updated.privileged as updated_privileged, updated.password as updated_password, updated.role_ids as "updated_role_ids: Vec<RoleId>", updated.enabled as updated_enabled, updated.refresh_token as updated_refresh_token, updated.refresh_token_expired_at as updated_refresh_token_expired_at
            FROM before
            JOIN updated ON before.id = updated.id;
            "#,
            &ids.inner_vec(),
            enabled,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(IamError::DatabaseError)?;
        let items = items
            .into_iter()
            .map(|row| UpdatedEvent {
                before: User::builder()
                    .id(UserId::new_unchecked(row.before_id))
                    .account(row.before_account)
                    .maybe_portrait(row.before_portrait)
                    .privileged(row.before_privileged)
                    .name(row.before_name)
                    .password(HashedPassword::new_unchecked(row.before_password))
                    .role_ids(row.before_role_ids)
                    .enabled(row.before_enabled)
                    .maybe_refresh_token(row.before_refresh_token)
                    .maybe_refresh_token_expired_at(row.before_refresh_token_expired_at)
                    .build(),
                after: User::builder()
                    .id(UserId::new_unchecked(row.updated_id))
                    .account(row.updated_account)
                    .maybe_portrait(row.updated_portrait)
                    .privileged(row.updated_privileged)
                    .name(row.updated_name)
                    .password(HashedPassword::new_unchecked(row.updated_password))
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
