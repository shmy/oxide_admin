use std::sync::Arc;

use chrono_tz::Tz;
use clap::{Parser, Subcommand};
use humantime::parse_duration;
use infrastructure::shared::config::{Config, ConfigRef, Database, Jwt, Log, Openapi, Server};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
/// Oxide admin server
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    /// Log level: trace, debug, info, warn, error
    #[arg(long, default_value = "info", env = "LOG_LEVEL")]
    pub log_level: String,

    #[cfg(feature = "trace_rolling")]
    /// Log file rotation strategy: minutely, hourly, daily, never
    #[arg(long, default_value = "daily", env = "LOG_ROLLING_KIND")]
    pub log_rolling_kind: String,

    #[cfg(feature = "trace_otlp")]
    /// OTLP reporting service name
    #[arg(long, default_value = "oxide_admin", env = "OTLP_SERVICE_NAME")]
    pub otlp_service_name: String,

    #[cfg(feature = "trace_otlp")]
    /// OTLP gRPC collector endpoint
    #[arg(long, default_value = "http://localhost:4317", env = "OTLP_ENDPOINT")]
    pub otlp_endpoint: String,

    #[cfg(feature = "trace_otlp")]
    /// OTLP gRPC metadata (JSON format)
    #[arg(long, default_value = "{}", env = "OTLP_METADATA")]
    pub otlp_metadata: String,

    /// Time zone
    #[arg(long, default_value = "Asia/Shanghai", env = "TIMEZONE")]
    pub timezone: Tz,

    /// Enable OpenAPI documentation
    #[arg(long, action = clap::ArgAction::Set, default_value_t = true, env = "OPENAPI_ENABLED")]
    pub openapi_enabled: bool,

    /// Database connection DSN
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,

    /// Maximum number of database connections
    #[arg(long, default_value = "100", env = "DATABASE_MAX_CONNECTIONS")]
    pub database_max_connections: u32,

    /// Minimum number of database connections
    #[arg(long, default_value = "1", env = "DATABASE_MIN_CONNECTIONS")]
    pub database_min_connections: u32,

    /// Maximum lifetime of a database connection
    #[arg(long, default_value = "15min", env = "DATABASE_MAX_LIFETIME")]
    pub database_max_lifetime: String,

    /// Maximum idle timeout of a database connection
    #[arg(long, default_value = "5min", env = "DATABASE_IDLE_TIMEOUT")]
    pub database_idle_timeout: String,

    /// Maximum wait time to acquire a database connection
    #[arg(long, default_value = "3s", env = "DATABASE_ACQUIRE_TIMEOUT")]
    pub database_acquire_timeout: String,

    #[cfg(feature = "kv_redis")]
    /// Redis connection DSN
    #[arg(long, env = "REDIS_URL")]
    pub redis_url: String,

    #[cfg(feature = "kv_redis")]
    /// Redis connection establishment timeout
    #[arg(long, default_value = "10s", env = "REDIS_CONNECTION_TIMEOUT")]
    pub redis_connection_timeout: String,

    #[cfg(feature = "kv_redis")]
    /// Maximum number of Redis connections
    #[arg(long, default_value = "100", env = "REDIS_MAX_SIZE")]
    pub redis_max_size: u32,

    #[cfg(feature = "kv_redis")]
    /// Minimum number of idle Redis connections
    #[arg(long, default_value = "1", env = "REDIS_MIN_IDLE")]
    pub redis_min_idle: u32,

    #[cfg(feature = "kv_redis")]
    /// Maximum lifetime of a Redis connection
    #[arg(long, default_value = "15min", env = "REDIS_MAX_LIFETIME")]
    pub redis_max_lifetime: String,

    #[cfg(feature = "kv_redis")]
    /// Maximum idle timeout of a Redis connection
    #[arg(long, default_value = "5min", env = "REDIS_IDLE_TIMEOUT")]
    pub redis_idle_timeout: String,

    #[cfg(feature = "bg_faktory")]
    /// Faktory server URL
    #[arg(long, env = "FAKTORY_ENDPOINT")]
    pub faktory_endpoint: String,

    #[cfg(feature = "bg_faktory")]
    /// Faktory queue name
    #[arg(long, default_value = "oxide-admin", env = "FAKTORY_QUEUE")]
    pub faktory_queue: String,

    /// Server bind address
    #[arg(long, default_value = "127.0.0.1", env = "SERVER_BIND")]
    pub server_bind: String,

    /// Server bind port
    #[arg(long, default_value = "8080", env = "SERVER_PORT")]
    pub server_port: u16,

    /// JWT access token secret
    #[arg(long, env = "JWT_ACCESS_TOKEN_SECRET")]
    pub jwt_access_token_secret: String,

    /// JWT access token validity period
    #[arg(long, default_value = "1h", env = "JWT_ACCESS_TOKEN_PERIOD")]
    pub jwt_access_token_period: String,

    /// JWT refresh token validity period
    #[arg(long, default_value = "7d", env = "JWT_REFRESH_TOKEN_PERIOD")]
    pub jwt_refresh_token_period: String,

    #[cfg(feature = "object_storage_fs")]
    /// File storage link signing secret
    #[arg(long, env = "FS_HMAC_SECRET")]
    pub fs_hmac_secret: String,

    #[cfg(feature = "object_storage_fs")]
    /// File storage link validity period
    #[arg(long, default_value = "1min", env = "FS_LINK_PERIOD")]
    pub fs_link_period: String,

    #[cfg(feature = "object_storage_s3")]
    /// S3 endpoint
    #[arg(long, env = "S3_ENDPOINT")]
    pub s3_endpoint: String,

    #[cfg(feature = "object_storage_s3")]
    /// S3 bucket name
    #[arg(long, env = "S3_BUCKET")]
    pub s3_bucket: String,

    #[cfg(feature = "object_storage_s3")]
    /// S3 access key ID
    #[arg(long, env = "S3_ACCESS_KEY_ID")]
    pub s3_access_key_id: String,

    #[cfg(feature = "object_storage_s3")]
    /// S3 secret access key
    #[arg(long, env = "S3_SECRET_ACCESS_KEY")]
    pub s3_secret_access_key: String,

    #[cfg(feature = "object_storage_s3")]
    /// S3 region
    #[arg(long, env = "S3_REGION")]
    pub s3_region: String,

    #[cfg(feature = "flag_flipt")]
    /// Flipt endpoint
    #[arg(long, env = "FLIPT_ENDPOINT")]
    pub flipt_endpoint: String,

    #[cfg(feature = "flag_flipt")]
    /// Flipt environment
    #[arg(long, env = "FLIPT_ENVIRONMENT")]
    pub flipt_environment: String,

    #[cfg(feature = "flag_flipt")]
    /// Flipt namespace
    #[arg(long, env = "FLIPT_NAMESPACE")]
    pub flipt_namespace: String,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Start web server
    Serve,
    #[cfg(not(feature = "serve_with_sched"))]
    /// Start scheduled task server
    Sched,
}

impl TryFrom<Cli> for ConfigRef {
    type Error = anyhow::Error;

    fn try_from(value: Cli) -> Result<Self, Self::Error> {
        let builder = Config::builder()
            .timezone(value.timezone)
            .openapi(Openapi::builder().enabled(value.openapi_enabled).build())
            .database(
                Database::builder()
                    .url(value.database_url)
                    .max_connections(value.database_max_connections)
                    .min_connections(value.database_min_connections)
                    .max_lifetime(parse_duration(&value.database_max_lifetime)?)
                    .idle_timeout(parse_duration(&value.database_idle_timeout)?)
                    .acquire_timeout(parse_duration(&value.database_acquire_timeout)?)
                    .build(),
            )
            .server(
                Server::builder()
                    .bind(value.server_bind)
                    .port(value.server_port)
                    .build(),
            )
            .jwt(
                Jwt::builder()
                    .access_token_secret(
                        Box::leak(Box::new(value.jwt_access_token_secret)).as_bytes(),
                    )
                    .access_token_period(parse_duration(&value.jwt_access_token_period)?)
                    .refresh_token_period(parse_duration(&value.jwt_refresh_token_period)?)
                    .build(),
            );
        #[cfg(feature = "object_storage_fs")]
        let builder = builder.fs(infrastructure::shared::config::StorageFs::builder()
            .hmac_secret(Box::leak(Box::new(value.fs_hmac_secret)).as_bytes())
            .link_period(parse_duration(&value.fs_link_period)?)
            .build());

        #[cfg(feature = "object_storage_s3")]
        let builder = builder.s3(infrastructure::shared::config::StorageS3::builder()
            .endpoint(value.s3_endpoint)
            .bucket(value.s3_bucket)
            .access_key_id(value.s3_access_key_id)
            .secret_access_key(value.s3_secret_access_key)
            .region(value.s3_region)
            .build());

        #[cfg(feature = "kv_redis")]
        let builder = builder.redis(
            infrastructure::shared::config::Redis::builder()
                .url(value.redis_url)
                .connection_timeout(parse_duration(&value.redis_connection_timeout)?)
                .idle_timeout(parse_duration(&value.redis_idle_timeout)?)
                .max_lifetime(parse_duration(&value.redis_max_lifetime)?)
                .max_size(value.redis_max_size)
                .min_idle(value.redis_min_idle)
                .build(),
        );
        #[cfg(feature = "bg_faktory")]
        let builder = builder.faktory(
            infrastructure::shared::config::Faktory::builder()
                .endpoint(value.faktory_endpoint)
                .queue(value.faktory_queue)
                .build(),
        );
        let builder = {
            let log_builder = Log::builder().level(value.log_level);
            #[cfg(feature = "trace_rolling")]
            let log_builder = log_builder.rolling_kind(value.log_rolling_kind);
            #[cfg(feature = "trace_otlp")]
            let log_builder = log_builder
                .otlp_service_name(value.otlp_service_name)
                .otlp_endpoint(value.otlp_endpoint)
                .otlp_metadata(value.otlp_metadata);
            builder.log(log_builder.build())
        };

        #[cfg(feature = "flag_flipt")]
        let builder = builder.flipt(
            infrastructure::shared::config::Flipt::builder()
                .endpoint(value.flipt_endpoint)
                .environment(value.flipt_environment)
                .namespace(value.flipt_namespace)
                .build(),
        );
        Ok(Arc::new(builder.build()))
    }
}
