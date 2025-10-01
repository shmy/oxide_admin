use domain::shared::port::domain_repository::DomainRepository;
use domain::shared::to_inner_vec::ToInnerVec;
use domain::system::port::file_repository::FileRepository;
use domain::system::value_object::file_id::FileId;
use domain::system::{entity::file::File, error::SystemError};
use nject::injectable;
use sqlx::FromRow;
use std::result::Result;

use crate::shared::chrono_tz::ChronoTz;
use crate::shared::pg_pool::PgPool;

#[derive(Debug, Clone)]
#[injectable]
pub struct FileRepositoryImpl {
    pool: PgPool,
    ct: ChronoTz,
}

impl DomainRepository for FileRepositoryImpl {
    type Entity = File;

    type EntityId = FileId;

    type Error = SystemError;

    #[tracing::instrument]
    async fn by_id(&self, id: &Self::EntityId) -> Result<Self::Entity, Self::Error> {
        let row_opt = sqlx::query_as!(
            FileDto,
            r#"
        SELECT id as "id: FileId", name, path, size, used FROM _files WHERE id = $1
        "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt.map(Into::into).ok_or(SystemError::FileNotFound)
    }

    #[tracing::instrument]
    async fn save(&self, entity: Self::Entity) -> Result<Self::Entity, Self::Error> {
        let now = self.ct.now();

        sqlx::query!(
            r#"
            INSERT INTO _files (id, name, path, size, used, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                path = EXCLUDED.path,
                size = EXCLUDED.size,
                used = EXCLUDED.used,
                updated_at = EXCLUDED.updated_at
            "#,
            &entity.id,
            &entity.name,
            &entity.path,
            &entity.size,
            &entity.used,
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
            FileDto,
            r#"
            DELETE FROM _files WHERE id = ANY($1) RETURNING id as "id: FileId", name, path, size, used
            "#,
            &ids.inner_vec()
        )
        .fetch_all(&self.pool)
        .await?;
        let items = items.into_iter().map(Into::into).collect();
        Ok(items)
    }
}

impl FileRepository for FileRepositoryImpl {}

#[derive(FromRow)]
struct FileDto {
    id: FileId,
    name: String,
    path: String,
    size: i64,
    used: bool,
}

impl From<FileDto> for File {
    fn from(value: FileDto) -> Self {
        Self::builder()
            .id(value.id)
            .name(value.name)
            .path(value.path)
            .size(value.size)
            .used(value.used)
            .build()
    }
}
