use bon::Builder;
use std::time::Duration;
use tracing_appender::rolling::Rotation;

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct Config {
    pub log: Log,
    pub database: Database,
    #[cfg(feature = "kv_redis")]
    pub redis: Redis,
    #[cfg(feature = "bg_faktory")]
    pub faktory: Faktory,
    pub server: Server,
    pub jwt: Jwt,
    #[cfg(feature = "object_storage_fs")]
    pub fs: StorageFs,
    #[cfg(feature = "object_storage_s3")]
    pub s3: StorageS3,
}

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct Log {
    pub level: String,
    pub rolling_period: Duration,
    pub rolling_kind: Rotation,
}

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct Database {
    pub url: String,
    pub timezone: chrono_tz::Tz,
    pub max_connections: u32,
    pub min_connections: u32,
    pub max_lifetime: Duration,
    pub idle_timeout: Duration,
    pub acquire_timeout: Duration,
}

#[cfg(feature = "kv_redis")]
#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct Redis {
    pub url: String,
    pub connection_timeout: Duration,
    pub max_size: u32,
    pub min_idle: u32,
    pub max_lifetime: Duration,
    pub idle_timeout: Duration,
}

#[cfg(feature = "bg_faktory")]
#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct Faktory {
    pub url: String,
    pub queue: String,
}

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct Server {
    pub bind: String,
    pub port: u16,
}

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct Jwt {
    pub access_token_secret: &'static [u8],
    pub access_token_period: Duration,
    pub refresh_token_period: Duration,
}

#[cfg(feature = "object_storage_fs")]
#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct StorageFs {
    pub hmac_secret: &'static [u8],
    pub link_period: Duration,
}

#[cfg(feature = "object_storage_s3")]
#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct StorageS3 {
    pub endpoint: String,
    pub bucket: String,
    pub client_id: String,
    pub client_secret: String,
    pub region: String,
}
