use bon::Builder;
use std::time::Duration;
use tracing_appender::rolling::Rotation;

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct Config {
    pub log: Log,
    pub database: Database,
    pub server: Server,
    pub jwt: Jwt,
    pub upload: Upload,
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

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct Upload {
    pub hmac_secret: &'static [u8],
    pub link_period: Duration,
}
