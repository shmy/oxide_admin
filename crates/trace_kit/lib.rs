use bon::Builder;
use tracing_subscriber::{
    EnvFilter, Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

#[derive(Builder)]
pub struct TraceConfig<'a> {
    level: &'a str,
    #[cfg(feature = "rolling")]
    rolling_kind: &'a str,
    #[cfg(feature = "rolling")]
    rolling_dir: &'a std::path::Path,
    #[cfg(feature = "otlp")]
    otlp_service_name: &'a str,
    #[cfg(feature = "otlp")]
    otlp_endpoint: &'a str,
    #[cfg(feature = "otlp")]
    otlp_metadata: &'a str,
}

pub fn init_tracing(config: TraceConfig) -> TracingGuard {
    #[cfg(feature = "console")]
    let console_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_filter(EnvFilter::new(config.level));
    #[cfg(feature = "rolling")]
    let (rolling_layer, rolling_guard) = {
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

    #[cfg(feature = "otlp")]
    let (otlp_layer, otlp_guard) = {
        use std::collections::HashMap;

        use opentelemetry::{KeyValue, trace::TracerProvider as _};
        use opentelemetry_otlp::{
            WithExportConfig as _, WithTonicConfig as _, tonic_types::metadata::MetadataMap,
        };
        use opentelemetry_sdk::{
            Resource,
            trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
        };
        use opentelemetry_semantic_conventions::resource::SERVICE_VERSION;
        let resource = Resource::builder()
            .with_service_name(config.otlp_service_name.to_string())
            .with_attributes([KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION"))])
            .build();
        let map: HashMap<String, String> =
            serde_json::from_str(config.otlp_metadata).expect("parse otlp_metadata");
        let mut metadata = MetadataMap::new();
        for (key, value) in map {
            let key = Box::leak(Box::new(key));
            metadata.insert(&**key, value.parse().expect("parse metadata value"));
        }
        let budiler = opentelemetry_otlp::SpanExporter::builder().with_tonic();
        #[cfg(feature = "otlp_tls")]
        let budiler = budiler.with_tls_config(
            opentelemetry_otlp::tonic_types::transport::ClientTlsConfig::new().with_enabled_roots(),
        );
        let exporter = budiler
            .with_endpoint(config.otlp_endpoint)
            .with_metadata(metadata)
            .build()
            .expect("build otlp exporter");

        let tracer_provider = SdkTracerProvider::builder()
            .with_sampler(Sampler::AlwaysOn)
            .with_resource(resource)
            .with_id_generator(RandomIdGenerator::default())
            .with_batch_exporter(exporter)
            .build();

        let tracer = tracer_provider.tracer("otlp_tracer");
        let layer = tracing_opentelemetry::layer().with_tracer(tracer);
        (layer, OtlpGuard(tracer_provider))
    };

    let builder = tracing_subscriber::registry();
    #[cfg(feature = "console")]
    let builder = builder.with(console_layer);
    #[cfg(feature = "rolling")]
    let builder = builder.with(rolling_layer);
    #[cfg(feature = "otlp")]
    let builder = builder.with(otlp_layer);
    builder.init();
    #[cfg(all(feature = "rolling", feature = "otlp"))]
    return TracingGuard::RollingAndOtlp(rolling_guard, otlp_guard);
    #[cfg(feature = "rolling")]
    #[allow(unreachable_code)]
    return TracingGuard::Rolling(rolling_guard);
    #[cfg(feature = "otlp")]
    #[allow(unreachable_code)]
    return TracingGuard::Otlp(otlp_guard);
    #[allow(unreachable_code)]
    TracingGuard::None
}

pub enum TracingGuard {
    #[cfg(all(feature = "rolling", feature = "otlp"))]
    RollingAndOtlp(tracing_appender::non_blocking::WorkerGuard, OtlpGuard),
    #[cfg(feature = "rolling")]
    Rolling(tracing_appender::non_blocking::WorkerGuard),
    #[cfg(feature = "otlp")]
    Otlp(OtlpGuard),
    None,
}

#[cfg(feature = "otlp")]
pub struct OtlpGuard(opentelemetry_sdk::trace::SdkTracerProvider);

#[cfg(feature = "otlp")]
impl Drop for OtlpGuard {
    fn drop(&mut self) {
        if let Err(err) = self.0.shutdown() {
            eprintln!("{err:?}");
        }
    }
}
