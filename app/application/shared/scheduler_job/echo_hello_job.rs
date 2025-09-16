use anyhow::Result;
use nject::injectable;
use sched_kit::ScheduledJob;
use tracing::info;

#[derive(Clone)]
#[injectable]
pub struct EchoHelloJob {}

impl ScheduledJob for EchoHelloJob {
    const SCHEDULER: &'static str = "every 3 seconds";

    async fn run(&self) -> Result<()> {
        info!("EchoHelloJob runing");
        Ok(())
    }
}
