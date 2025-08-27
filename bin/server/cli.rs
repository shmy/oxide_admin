use chrono_tz::Tz;
use clap::Parser;
use humantime::parse_duration;
use infrastructure::shared::config::{Config, Database, Jwt, Log, Server, Upload};
use tracing_appender::rolling::Rotation;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
/// 启动 Web 服务器
pub struct Cli {
    /// 日志级别: trace debug info warn error
    #[arg(long, default_value = "info", env = "LOG_LEVEL")]
    pub log_level: String,

    /// 日志文件有效期
    #[arg(long, default_value = "30d", env = "LOG_ROLLING_PERIOD")]
    pub log_rolling_period: String,

    /// 日志文件滚动周期: minutely hourly daily never
    #[arg(long, default_value = "daily", env = "LOG_ROLLING_KIND")]
    pub log_rolling_kind: String,

    /// 数据库连接地址
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,

    /// 数据库时区
    #[arg(long, default_value = "Asia/Shanghai", env = "DATABASE_TIMEZONE")]
    pub database_timezone: Tz,

    /// 数据库最大连接数
    #[arg(long, default_value = "100", env = "DATABASE_MAX_CONNECTIONS")]
    pub database_max_connections: u32,

    /// 数据库最小连接数
    #[arg(long, default_value = "1", env = "DATABASE_MIN_CONNECTIONS")]
    pub database_min_connections: u32,

    /// 数据库最大连接时间
    #[arg(long, default_value = "15min", env = "DATABASE_MAX_LIFETIME")]
    pub database_max_lifetime: String,

    /// 数据库最大空闲时间
    #[arg(long, default_value = "5min", env = "DATABASE_IDLE_TIMEOUT")]
    pub database_idle_timeout: String,

    /// 数据库最大等待时间
    #[arg(long, default_value = "3s", env = "DATABASE_ACQUIRE_TIMEOUT")]
    pub database_acquire_timeout: String,

    /// 绑定的主机地址
    #[arg(long, default_value = "127.0.0.1", env = "SERVER_BIND")]
    pub server_bind: String,

    /// 绑定的端口号
    #[arg(long, default_value = "8080", env = "SERVER_PORT")]
    pub server_port: u16,

    /// JWT 访问令牌密钥
    #[arg(long, env = "JWT_ACCESS_TOKEN_SECRET")]
    pub jwt_access_token_secret: String,

    /// JWT 访问令牌有效期
    #[arg(long, default_value = "1h", env = "JWT_ACCESS_TOKEN_PERIOD")]
    pub jwt_access_token_period: String,

    /// JWT 刷新令牌有效期
    #[arg(long, default_value = "7d", env = "JWT_REFRESH_TOKEN_PERIOD")]
    pub jwt_refresh_token_period: String,

    /// 上传文件链接签名的密钥
    #[arg(long, env = "UPLOAD_HMAC_SECRET")]
    pub upload_hmac_secret: String,

    /// 上传文件链接访问有效期
    #[arg(long, default_value = "1min", env = "UPLOAD_LINK_PERIOD")]
    pub upload_link_period: String,
}

impl TryFrom<Cli> for Config {
    type Error = anyhow::Error;

    fn try_from(value: Cli) -> Result<Self, Self::Error> {
        Ok(Self::builder()
            .log(
                Log::builder()
                    .level(value.log_level)
                    .rolling_period(parse_duration(&value.log_rolling_period)?)
                    .rolling_kind({
                        match value.log_rolling_kind.as_str() {
                            "minutely" => Rotation::MINUTELY,
                            "hourly" => Rotation::HOURLY,
                            "daily" => Rotation::DAILY,
                            _ => Rotation::NEVER,
                        }
                    })
                    .build(),
            )
            .database(
                Database::builder()
                    .url(value.database_url)
                    .timezone(value.database_timezone)
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
            )
            .upload(
                Upload::builder()
                    .hmac_secret(Box::leak(Box::new(value.upload_hmac_secret)).as_bytes())
                    .link_period(parse_duration(&value.upload_link_period)?)
                    .build(),
            )
            .build())
    }
}
