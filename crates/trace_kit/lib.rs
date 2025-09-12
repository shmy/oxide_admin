use bon::Builder;
use tracing_subscriber::{
    EnvFilter, Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

#[derive(Builder)]
pub struct TracingConfig<'a> {
    pub level: &'a str,
    #[cfg(feature = "rolling")]
    pub rolling_kind: &'a str,
    #[cfg(feature = "rolling")]
    pub rolling_dir: &'a std::path::Path,
}

pub fn init_tracing(config: TracingConfig) -> TracingGuard {
    #[cfg(feature = "console")]
    let console_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_filter(EnvFilter::new(config.level));
    #[cfg(feature = "rolling")]
    let (rolling_layer, guard) = {
        use tracing_appender::rolling::Rotation;

        let rolling_kind = match config.rolling_kind {
            "minutely" => Rotation::MINUTELY,
            "hourly" => Rotation::HOURLY,
            "daily" => Rotation::DAILY,
            _ => Rotation::NEVER,
        };
        let file_appender = tracing_appender::rolling::RollingFileAppender::new(
            rolling_kind,
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
            .with_filter(EnvFilter::new(config.level));
        (rolling_layer, guard)
    };

    let builder = tracing_subscriber::registry();
    #[cfg(feature = "console")]
    let builder = builder.with(console_layer);
    #[cfg(feature = "rolling")]
    let builder = builder.with(rolling_layer);
    builder.init();
    #[cfg(feature = "rolling")]
    return TracingGuard::Rolling(guard);
    #[cfg(not(feature = "rolling"))]
    TracingGuard::None
}

pub enum TracingGuard {
    #[cfg(feature = "rolling")]
    Rolling(tracing_appender::non_blocking::WorkerGuard),
    None,
}
