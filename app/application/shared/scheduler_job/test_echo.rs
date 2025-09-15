use anyhow::Result;
use nject::injectable;
use sched_kit::ScheduledJob;
use tracing::info;

#[derive(Clone)]
#[injectable]
pub struct TestEcho {}

impl ScheduledJob for TestEcho {
    const SCHEDULER: &'static str = "every 3 seconds";

    async fn run(&self) -> Result<()> {
        info!("TestEcho runing");
        Ok(())
    }
}
