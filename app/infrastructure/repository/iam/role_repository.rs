use std::result::Result;

use domain::iam::port::role_repository::RoleRepository;
use domain::iam::value_object::permission_code::PermissionCode;
use domain::iam::value_object::role_id::RoleId;
use domain::iam::{entity::role::Role, error::IamError};
use domain::shared::event_util::UpdatedEvent;
use domain::shared::port::domain_repository::DomainRepository;
use domain::shared::to_inner_vec::ToInnerVec;
use nject::injectable;
use sqlx::prelude::FromRow;

use crate::shared::chrono_tz::ChronoTz;
use crate::shared::error_util::is_unique_constraint_error;
use crate::shared::pg_pool::PgPool;

#[injectable]
pub struct RoleRepositoryImpl {
    pool: PgPool,
    ct: ChronoTz,
}

impl DomainRepository for RoleRepositoryImpl {
    type Entity = Role;

    type EntityId = RoleId;

    type Error = IamError;

    async fn by_id(&self, id: &Self::EntityId) -> Result<Self::Entity, IamError> {
        let row_opt = sqlx::query_as!(
            RoleDto,
            r#"
        SELECT id as "id: RoleId", name, privileged, permission_ids as "permission_ids: Vec<PermissionCode>", enabled
        FROM _roles WHERE id = $1
        "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(IamError::DatabaseError)?;
        row_opt.map(Into::into).ok_or(IamError::RoleNotFound)
    }

    async fn save(&self, entity: Self::Entity) -> Result<Self::Entity, IamError> {
        let now = self.ct.now();

        sqlx::query!(
            r#"
            INSERT INTO _roles (id, name, privileged, permission_ids, enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                privileged = EXCLUDED.privileged,
                permission_ids = EXCLUDED.permission_ids,
                enabled = EXCLUDED.enabled,
                updated_at = EXCLUDED.updated_at
            "#,
            &entity.id,
            &entity.name,
            &entity.privileged,
            &entity.permission_ids.inner_vec(),
            &entity.enabled,
            &now,
            &now
        )
        .execute(&self.pool)
        .await.map_err(|e| {
            if is_unique_constraint_error(&e, "_roles", "name") {
                return IamError::RoleDuplicated;
            }
            IamError::DatabaseError(e)
        })?;
        Ok(entity)
    }

    async fn batch_delete(&self, ids: &[Self::EntityId]) -> Result<Vec<Self::Entity>, Self::Error> {
        if ids.is_empty() {
            return Ok(Vec::with_capacity(0));
        }
        let items = sqlx::query_as!(
            RoleDto,
            r#"
            DELETE FROM _roles WHERE id = ANY($1) AND privileged != true RETURNING id as "id: RoleId", name, privileged, permission_ids as "permission_ids: Vec<PermissionCode>", enabled
            "#,
            &ids.inner_vec()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(IamError::DatabaseError)?;
        let items = items.into_iter().map(Into::into).collect();
        Ok(items)
    }
}

impl RoleRepository for RoleRepositoryImpl {
    async fn toggle_enabled(
        &self,
        ids: &[RoleId],
        enabled: bool,
    ) -> std::result::Result<Vec<UpdatedEvent<Self::Entity>>, Self::Error> {
        if ids.is_empty() {
            return Ok(Vec::with_capacity(0));
        }
        let items = sqlx::query!(
            r#"
            WITH before AS (
                SELECT * FROM _roles WHERE id = ANY($1) AND privileged != true
            ),
            updated AS (
                UPDATE _roles SET enabled = $2
                WHERE id = ANY($1) AND privileged != true
                RETURNING *
            )
            SELECT
            before.id as "before_id: RoleId", before.name as before_name, before.privileged as before_privileged, before.permission_ids as "before_permission_ids: Vec<PermissionCode>", before.enabled as before_enabled,
            updated.id as "updated_id: RoleId", updated.name as updated_name, updated.privileged as updated_privileged, updated.permission_ids as "updated_permission_ids: Vec<PermissionCode>", updated.enabled as updated_enabled
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
                before: Role::builder()
                    .id(row.before_id)
                    .enabled(row.before_enabled)
                    .name(row.before_name)
                    .privileged(row.before_privileged)
                    .permission_ids(row.before_permission_ids)
                    .build(),
                after: Role::builder()
                    .id(row.updated_id)
                    .enabled(row.updated_enabled)
                    .name(row.updated_name)
                    .privileged(row.updated_privileged)
                    .permission_ids(row.updated_permission_ids)
                    .build(),
            })
            .collect();
        Ok(items)
    }
}

#[derive(FromRow)]
struct RoleDto {
    id: RoleId,
    name: String,
    privileged: bool,
    permission_ids: Vec<PermissionCode>,
    enabled: bool,
}

impl From<RoleDto> for Role {
    fn from(value: RoleDto) -> Self {
        Self::builder()
            .id(value.id)
            .enabled(value.enabled)
            .name(value.name)
            .privileged(value.privileged)
            .permission_ids(value.permission_ids)
            .build()
    }
}
