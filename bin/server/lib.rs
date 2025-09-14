use adapter::WebState;
use anyhow::Result;
use application::shared::{
    background_worker::register_workers, event_subscriber::register_subscribers,
};
use axum::Router;
use bg_worker::queuer::Queuer;
use bg_worker::worker_manager::WorkerManager;
use infrastructure::shared::{
    config::{Config, Log, Server},
    pg_pool,
};
use infrastructure::{migration, shared::pg_pool::PgPool, shared::provider::Provider};
use kvdb::{Kvdb, KvdbTrait as _};
use object_storage::ObjectStorage;
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    str::FromStr as _,
    sync::Arc,
};
use tokio::{net::TcpListener, signal, sync::Notify, try_join};
use trace_kit::TracingGuard;
use tracing::{info, warn};

pub mod cli;

pub async fn bootstrap(config: Config) -> Result<()> {
    let _guard = init_tracing(&config.log);
    let listener = build_listener(&config.server).await?;
    let provider = build_provider(config).await?;
    let ((), worker) = try_join!(initilize(&provider), build_worker_manager(&provider))?;
    let pg_pool = provider.provide::<PgPool>();
    let kvdb = provider.provide::<Kvdb>();
    let app = adapter::routing(WebState::new(provider));
    let notify_shutdown = Arc::new(Notify::new());
    let background_job_handle =
        tokio::spawn(start_background_worker(worker, notify_shutdown.clone()));
    let server_handle = tokio::spawn(start_http_server(listener, app, notify_shutdown.clone()));
    shutdown_signal().await;
    notify_shutdown.notify_waiters();
    let _ = tokio::join!(background_job_handle, server_handle);
    tokio::join!(pg_pool.close(), kvdb.close());
    info!("👋 Goodbye!");
    Ok(())
}

fn init_tracing(config: &Log) -> TracingGuard {
    let config_builder = trace_kit::TraceConfig::builder().level(&config.level);
    #[cfg(feature = "trace_rolling")]
    let config_builder = config_builder
        .rolling_kind(&config.rolling_kind)
        .rolling_dir(infrastructure::shared::path::LOG_DIR.as_path());
    #[cfg(feature = "trace_otlp")]
    let config_builder = config_builder
        .otlp_service_name(&config.otlp_service_name)
        .otlp_endpoint(&config.otlp_endpoint)
        .otlp_metadata(&config.otlp_metadata);
    trace_kit::init_tracing(config_builder.build())
}

async fn build_listener(server: &Server) -> Result<TcpListener> {
    let ip = Ipv4Addr::from_str(&server.bind)?;
    let addr = SocketAddrV4::new(ip, server.port);
    let listener = TcpListener::bind(addr).await?;
    Ok(listener)
}

async fn build_provider(config: Config) -> Result<Provider> {
    let (pg_pool, kvdb, queuer, object_storage) = tokio::try_join!(
        pg_pool::try_new(&config.database),
        build_kvdb(&config),
        build_queuer(&config),
        build_object_storage(&config),
    )?;
    let provider = Provider::builder()
        .pg_pool(pg_pool)
        .kvdb(kvdb)
        .config(config)
        .queuer(queuer)
        .object_storage(object_storage)
        .build();
    Ok(provider)
}

async fn initilize(provider: &Provider) -> Result<()> {
    migration::migrate(provider).await?;
    register_subscribers(provider);
    Ok(())
}

#[allow(unused_variables)]
async fn build_queuer(config: &Config) -> Result<Queuer> {
    #[cfg(feature = "bg_faktory")]
    return Queuer::try_new(&config.faktory.url, &config.faktory.queue).await;
    #[cfg(feature = "bg_dummy")]
    return Queuer::try_new().await;
}

async fn build_object_storage(config: &Config) -> Result<ObjectStorage> {
    #[cfg(feature = "object_storage_fs")]
    return {
        let config = object_storage::FsConfig::builder()
            .root(
                infrastructure::shared::path::UPLOAD_DIR
                    .to_string_lossy()
                    .to_string(),
            )
            .basepath(adapter::UPLOAD_PATH.to_string())
            .hmac_secret(config.fs.hmac_secret)
            .link_period(config.fs.link_period)
            .build();
        ObjectStorage::try_new(config)
    };
    #[cfg(feature = "object_storage_s3")]
    return {
        let config = object_storage::S3Config::builder()
            .endpoint(config.s3.endpoint.to_string())
            .bucket(config.s3.bucket.to_string())
            .access_key_id(config.s3.access_key_id.to_string())
            .secret_access_key(config.s3.secret_access_key.to_string())
            .region(config.s3.region.to_string())
            .build();
        ObjectStorage::try_new(config).await
    };
}

#[allow(unused_variables)]
async fn build_kvdb(config: &Config) -> Result<Kvdb> {
    #[cfg(feature = "kv_redb")]
    return Kvdb::try_new(infrastructure::shared::path::DATA_DIR.join("data.redb")).await;
    #[cfg(feature = "kv_redis")]
    return {
        let config = kvdb::RedisKvdbConfig::builder()
            .url(config.redis.url.clone())
            .connection_timeout(config.redis.connection_timeout)
            .max_size(config.redis.max_size)
            .min_idle(config.redis.min_idle)
            .max_lifetime(config.redis.max_lifetime)
            .idle_timeout(config.redis.idle_timeout)
            .build();
        Kvdb::try_new(config).await
    };
}

async fn build_worker_manager(provider: &Provider) -> Result<WorkerManager> {
    #[cfg(feature = "bg_faktory")]
    let mut worker_manager = {
        let config = &provider.provide::<Config>();
        WorkerManager::new(&config.faktory.url, &config.faktory.queue)
    };
    #[cfg(feature = "bg_dummy")]
    let mut worker_manager = WorkerManager::new();
    register_workers(&mut worker_manager, provider);
    Ok(worker_manager)
}

async fn start_http_server(listener: TcpListener, app: Router, notify: Arc<Notify>) -> Result<()> {
    let shutdown = async move {
        notify.notified().await;
        info!("Received shutdown signal, shutting down server...");
    };
    info!("🚀 Server is running on http://{}", listener.local_addr()?);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await?;
    info!("Server shutdown complete");
    Ok(())
}

async fn start_background_worker(
    mut worker_manager: WorkerManager,
    notify: Arc<Notify>,
) -> Result<()> {
    let shutdown = async move {
        notify.notified().await;
        info!("Received shutdown signal, shutting down background worker...");
    };
    worker_manager.run_with_signal(shutdown).await?;
    info!("Background worker shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        warn!("Received Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
        warn!("Received SIGTERM");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
