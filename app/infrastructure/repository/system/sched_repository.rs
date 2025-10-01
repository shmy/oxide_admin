use domain::shared::port::domain_repository::DomainRepository;
use domain::shared::to_inner_vec::ToInnerVec;
use domain::system::port::sched_repository::SchedRepository;
use domain::system::value_object::sched_id::SchedId;
use domain::system::{entity::sched::Sched, error::SystemError};
use nject::injectable;
use sqlx::FromRow;
use std::result::Result;

use crate::shared::chrono_tz::ChronoTz;
use crate::shared::pg_pool::PgPool;

#[derive(Debug, Clone)]
#[injectable]
pub struct SchedRepositoryImpl {
    pool: PgPool,
    ct: ChronoTz,
}

impl DomainRepository for SchedRepositoryImpl {
    type Entity = Sched;

    type EntityId = SchedId;

    type Error = SystemError;

    #[tracing::instrument]
    async fn by_id(&self, id: &Self::EntityId) -> Result<Self::Entity, Self::Error> {
        let row_opt = sqlx::query_as!(
            SchedDto,
            r#"
        SELECT id as "id: SchedId", key, name, schedule, succeed, result, run_at, duration_ms FROM _scheds WHERE id = $1
        "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt.map(Into::into).ok_or(SystemError::SchedNotFound)
    }

    #[tracing::instrument]
    async fn save(&self, entity: Self::Entity) -> Result<Self::Entity, Self::Error> {
        let now = self.ct.now();

        sqlx::query!(
            r#"
            INSERT INTO _scheds (id, key, name, schedule, succeed, result, run_at, duration_ms, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                key = EXCLUDED.key,
                name = EXCLUDED.name,
                schedule = EXCLUDED.schedule,
                succeed = EXCLUDED.succeed,
                result = EXCLUDED.result,
                run_at = EXCLUDED.run_at,
                duration_ms = EXCLUDED.duration_ms,
                updated_at = EXCLUDED.updated_at
            "#,
            &entity.id,
            &entity.key,
            &entity.name,
            &entity.schedule,
            &entity.succeed,
            &entity.result,
            &entity.run_at,
            &entity.duration_ms,
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
            SchedDto,
            r#"
            DELETE FROM _scheds WHERE id = ANY($1) RETURNING id as "id: SchedId", key, name, schedule, succeed, result, run_at, duration_ms
            "#,
            &ids.inner_vec()
        )
        .fetch_all(&self.pool)
        .await?;
        let items = items.into_iter().map(Into::into).collect();
        Ok(items)
    }
}

impl SchedRepository for SchedRepositoryImpl {}

#[derive(FromRow)]
struct SchedDto {
    id: SchedId,
    key: String,
    name: String,
    schedule: String,
    succeed: bool,
    result: String,
    run_at: chrono::NaiveDateTime,
    duration_ms: i64,
}

impl From<SchedDto> for Sched {
    fn from(value: SchedDto) -> Self {
        Self::builder()
            .id(value.id)
            .key(value.key)
            .name(value.name)
            .schedule(value.schedule)
            .succeed(value.succeed)
            .result(value.result)
            .run_at(value.run_at)
            .duration_ms(value.duration_ms)
            .build()
    }
}
