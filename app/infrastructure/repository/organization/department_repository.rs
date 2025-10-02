use domain::organization::port::department_repository::DepartmentRepository;
use domain::organization::value_object::department_id::DepartmentId;
use domain::organization::{entity::department::Department, error::OrganizationError};
use domain::shared::port::domain_repository::DomainRepository;
use domain::shared::to_inner_vec::ToInnerVec;
use nject::injectable;
use sqlx::FromRow;
use std::result::Result;

use crate::shared::chrono_tz::ChronoTz;
use crate::shared::pg_pool::PgPool;

#[derive(Debug)]
#[injectable]
pub struct DepartmentRepositoryImpl {
    pool: PgPool,
    ct: ChronoTz,
}

impl DomainRepository for DepartmentRepositoryImpl {
    type Entity = Department;

    type EntityId = DepartmentId;

    type Error = OrganizationError;

    #[tracing::instrument]
    async fn by_id(&self, id: &Self::EntityId) -> Result<Self::Entity, Self::Error> {
        let row_opt = sqlx::query_as!(
            DepartmentDto,
            r#"
        SELECT id as "id: DepartmentId", name, code, parent_id, enabled FROM _departments WHERE id = $1
        "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt
            .map(Into::into)
            .ok_or(OrganizationError::DepartmentNotFound)
    }

    #[tracing::instrument]
    async fn save(&self, entity: Self::Entity) -> Result<Self::Entity, Self::Error> {
        let now = self.ct.now();

        sqlx::query!(
            r#"
            INSERT INTO _departments (id, name, code, parent_id, enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                code = EXCLUDED.code,
                parent_id = EXCLUDED.parent_id,
                enabled = EXCLUDED.enabled,
                updated_at = EXCLUDED.updated_at
            "#,
            &entity.id,
            &entity.name,
            &entity.code,
            entity.parent_id,
            &entity.enabled,
            &now,
            &now,
        )
        .execute(&self.pool)
        .await?;
        Ok(entity)
    }

    #[tracing::instrument]
    async fn batch_delete(&self, ids: &[Self::EntityId]) -> Result<Vec<Self::Entity>, Self::Error> {
        if ids.is_empty() {
            return Ok(Vec::with_capacity(0));
        }
        let items = sqlx::query_as!(
            DepartmentDto,
            r#"
            DELETE FROM _departments WHERE id = ANY($1) RETURNING id as "id: DepartmentId", name, code, parent_id, enabled
            "#,
            &ids.inner_vec()
        )
        .fetch_all(&self.pool)
        .await?;
        let items = items.into_iter().map(Into::into).collect();
        Ok(items)
    }
}

impl DepartmentRepository for DepartmentRepositoryImpl {}

#[derive(FromRow)]
struct DepartmentDto {
    id: DepartmentId,
    name: String,
    code: String,
    parent_id: Option<String>,
    enabled: bool,
}

impl From<DepartmentDto> for Department {
    fn from(value: DepartmentDto) -> Self {
        Self::builder()
            .id(value.id)
            .name(value.name)
            .code(value.code)
            .maybe_parent_id(value.parent_id)
            .enabled(value.enabled)
            .build()
    }
}
