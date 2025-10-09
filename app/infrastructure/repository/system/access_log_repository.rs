use domain::shared::port::domain_repository::DomainRepository;
use domain::shared::to_inner_vec::ToInnerVec;
use domain::system::port::access_log_repository::AccessLogRepository;
use domain::system::value_object::access_log_id::AccessLogId;
use domain::system::{entity::access_log::AccessLog, error::SystemError};
use nject::injectable;
use sqlx::FromRow;
use std::result::Result;

use crate::shared::chrono_tz::ChronoTz;
use crate::shared::pg_pool::PgPool;

#[derive(Clone, Debug)]
#[injectable]
pub struct AccessLogRepositoryImpl {
    pool: PgPool,
    ct: ChronoTz,
}

impl DomainRepository for AccessLogRepositoryImpl {
    type Entity = AccessLog;

    type EntityId = AccessLogId;

    type Error = SystemError;

    #[tracing::instrument]
    async fn by_id(&self, id: &Self::EntityId) -> Result<Self::Entity, Self::Error> {
        let row_opt = sqlx::query_as!(
            AccessLogDto,
            r#"
        SELECT id as "id: AccessLogId", user_id, method, uri, user_agent, ip, ip_region, status, elapsed FROM _access_logs WHERE id = $1
        "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt
            .map(Into::into)
            .ok_or(SystemError::AccessLogNotFound)
    }

    #[tracing::instrument]
    async fn save(&self, entity: Self::Entity) -> Result<Self::Entity, Self::Error> {
        let now = self.ct.now();

        sqlx::query!(
            r#"
            INSERT INTO _access_logs (id, user_id, method, uri, user_agent, ip, ip_region, status, elapsed, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id) DO UPDATE SET
                user_id = EXCLUDED.user_id,
                method = EXCLUDED.method,
                uri = EXCLUDED.uri,
                user_agent = EXCLUDED.user_agent,
                ip = EXCLUDED.ip,
                ip_region = EXCLUDED.ip_region,
                status = EXCLUDED.status,
                elapsed = EXCLUDED.elapsed,
                updated_at = EXCLUDED.updated_at
            "#,
            &entity.id,
               &entity.user_id,
               &entity.method,
               &entity.uri,
               entity.user_agent,
               entity.ip,
               entity.ip_region,
               &entity.status,
               &entity.elapsed,
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
            AccessLogDto,
            r#"
            DELETE FROM _access_logs WHERE id = ANY($1) RETURNING id as "id: AccessLogId", user_id, method, uri, user_agent, ip, ip_region, status, elapsed
            "#,
            &ids.inner_vec()
        )
        .fetch_all(&self.pool)
        .await?;
        let items = items.into_iter().map(Into::into).collect();
        Ok(items)
    }
}

impl AccessLogRepository for AccessLogRepositoryImpl {}

#[derive(FromRow)]
struct AccessLogDto {
    id: AccessLogId,
    user_id: String,
    method: String,
    uri: String,
    user_agent: Option<String>,
    ip: Option<String>,
    ip_region: Option<String>,
    status: i16,
    elapsed: i64,
}

impl From<AccessLogDto> for AccessLog {
    fn from(value: AccessLogDto) -> Self {
        Self::builder()
            .id(value.id)
            .user_id(value.user_id)
            .method(value.method)
            .uri(value.uri)
            .maybe_user_agent(value.user_agent)
            .maybe_ip(value.ip)
            .maybe_ip_region(value.ip_region)
            .status(value.status)
            .elapsed(value.elapsed)
            .build()
    }
}
