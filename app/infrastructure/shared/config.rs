use bon::Builder;
use std::fmt::Debug;
use std::time::Duration;

#[derive(Clone, Builder)]
#[readonly::make]
pub struct Config {
    pub log: Log,
    pub timezone: chrono_tz::Tz,

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
    #[cfg(feature = "flag_flipt")]
    pub flip: Flip,
}

impl Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config").finish()
    }
}

#[derive(Clone, Builder)]
#[readonly::make]
pub struct Log {
    pub level: String,
    #[cfg(feature = "trace_rolling")]
    pub rolling_kind: String,
    #[cfg(feature = "trace_otlp")]
    pub otlp_service_name: String,
    #[cfg(feature = "trace_otlp")]
    pub otlp_endpoint: String,
    #[cfg(feature = "trace_otlp")]
    pub otlp_metadata: String,
}

#[derive(Clone, Builder)]
#[readonly::make]
pub struct Database {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub max_lifetime: Duration,
    pub idle_timeout: Duration,
    pub acquire_timeout: Duration,
}

#[cfg(feature = "kv_redis")]
#[derive(Clone, Builder)]
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
#[derive(Clone, Builder)]
#[readonly::make]
pub struct Faktory {
    pub url: String,
    pub queue: String,
}

#[derive(Clone, Builder)]
#[readonly::make]
pub struct Server {
    pub bind: String,
    pub port: u16,
}

#[derive(Clone, Builder)]
#[readonly::make]
pub struct Jwt {
    pub access_token_secret: &'static [u8],
    pub access_token_period: Duration,
    pub refresh_token_period: Duration,
}

#[cfg(feature = "object_storage_fs")]
#[derive(Clone, Builder)]
#[readonly::make]
pub struct StorageFs {
    pub hmac_secret: &'static [u8],
    pub link_period: Duration,
}

#[cfg(feature = "object_storage_s3")]
#[derive(Clone, Builder)]
#[readonly::make]
pub struct StorageS3 {
    pub endpoint: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
}

#[cfg(feature = "flag_flipt")]
#[derive(Clone, Builder)]
#[readonly::make]
pub struct Flip {
    pub endpoint: String,
    pub environment: String,
    pub namespace: String,
}
