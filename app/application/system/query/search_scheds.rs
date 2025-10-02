use std::collections::HashMap;

use crate::shared::paging_query::PagingQuery;
use crate::shared::paging_result::PagingResult;
use crate::shared::query_handler::QueryHandler;
use crate::shared::scheduler_job_impl::SCHEDULER_JOBS;
use crate::system::dto::sched::SchedDto;
use bon::Builder;
use domain::system::error::SystemError;
use infrastructure::shared::config::ConfigRef;
use infrastructure::shared::pg_pool::PgPool;
use nject::injectable;
use sched_kit::cron_tab::next_tick;
use serde::Deserialize;
use serde_with::serde_as;
use utoipa::IntoParams;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, IntoParams, Builder)]
pub struct SearchSchedsQuery {
    #[serde(flatten)]
    #[param(inline)]
    paging: PagingQuery,
}

#[derive(Debug, Clone)]
#[injectable]
pub struct SearchSchedsQueryHandler {
    pool: PgPool,
    config: ConfigRef,
}

impl QueryHandler for SearchSchedsQueryHandler {
    type Query = SearchSchedsQuery;
    type Output = PagingResult<SchedDto>;
    type Error = SystemError;

    #[tracing::instrument]
    async fn query(&self, query: SearchSchedsQuery) -> Result<PagingResult<SchedDto>, SystemError> {
        let total = SCHEDULER_JOBS.len();
        let page = query.paging.page();
        let page_size = query.paging.page_size();
        let start = (((page - 1) * page_size) as usize).min(total);
        let end = (start + page_size as usize).min(total);
        let page_jobs = &SCHEDULER_JOBS[start..end];
        let keys = page_jobs
            .iter()
            .map(|x| x.key.to_string())
            .collect::<Vec<_>>();

        let record = sqlx::query!(
            r#"
            SELECT DISTINCT ON (key) 
                key, succeed, result, run_at, duration_ms
            FROM _scheds
            WHERE key = ANY($1)
            ORDER BY key, run_at DESC
            "#,
            &keys,
        )
        .fetch_all(&self.pool)
        .await?;
        let record_map = record
            .into_iter()
            .map(|row| (row.key.to_string(), row))
            .collect::<HashMap<_, _>>();
        let items = page_jobs
            .iter()
            .map(|job| SchedDto {
                key: job.key.to_string(),
                name: job.name.to_string(),
                expr: job.expr.to_string(),
                last_succeed: record_map.get(job.key).map(|row| row.succeed),
                last_result: record_map.get(job.key).map(|row| row.result.clone()),
                last_run_at: record_map.get(job.key).map(|row| row.run_at),
                next_run_at: next_tick(job.expr, self.config.timezone).map(|d| d.naive_local()),
                last_duration_ms: record_map.get(job.key).map(|row| row.duration_ms),
            })
            .collect();
        Ok(PagingResult {
            total: total as i64,
            items,
        })
    }
}
