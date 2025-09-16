use anyhow::Result;
use infrastructure::shared::feature_flag::FeatureFlag;
use nject::injectable;
use sched_kit::ScheduledJob;
use tracing::info;

#[derive(Clone)]
#[injectable]
pub struct TestEcho {
    feature_flag: FeatureFlag,
}

impl ScheduledJob for TestEcho {
    const SCHEDULER: &'static str = "every 3 seconds";

    async fn run(&self) -> Result<()> {
        let show_banner = self
            .feature_flag
            .get_bool_value("show-banner", None, None)
            .await
            .unwrap_or(false);
        info!("TestEcho runing {}", show_banner);
        Ok(())
    }
}
