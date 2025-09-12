use std::path::Path;

use bon::Builder;
pub use tracing_appender::non_blocking::WorkerGuard;
pub use tracing_appender::rolling::Rotation;
use tracing_subscriber::{
    EnvFilter, Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

#[derive(Builder)]
pub struct TracingConfig<'a> {
    pub level: &'a str,
    pub rolling_kind: tracing_appender::rolling::Rotation,
    pub rolling_dir: &'a Path,
}

pub fn init_tracing(config: TracingConfig) -> WorkerGuard {
    let console_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_filter(EnvFilter::new(&config.level));
    let (rolling_layer, guard) = {
        let file_appender = tracing_appender::rolling::RollingFileAppender::new(
            config.rolling_kind,
            config.rolling_dir,
            "log",
        );
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let rolling_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(non_blocking)
            .with_line_number(true)
            .with_file(true)
            .with_ansi(false)
            .with_filter(EnvFilter::new(&config.level));
        (rolling_layer, guard)
    };

    tracing_subscriber::registry()
        .with(console_layer)
        .with(rolling_layer)
        .init();
    guard
}
