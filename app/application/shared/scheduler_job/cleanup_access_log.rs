use infrastructure::shared::pg_pool::PgPool;
use nject::injectable;
use sched_kit::ScheduledJob;
use sched_kit::error::{Result, SchedError};

#[derive(Clone)]
#[injectable]
pub struct CleanupAccessLog {
    pool: PgPool,
}

impl ScheduledJob for CleanupAccessLog {
    const EXPR: &'static str = "at 02:01";
    const NAME: &'static str = "CleanupAccessLog";

    async fn run(&self) -> Result<()> {
        sqlx::query!("DELETE FROM _access_logs WHERE occurred_at < NOW() - INTERVAL '7 days'")
            .execute(&self.pool)
            .await
            .map_err(|e| SchedError::Custom(e.to_string()))?;
        Ok(())
    }
}
