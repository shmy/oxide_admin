use adapter::WebState;
use anyhow::Result;
use application::shared::{
    background_worker::register_workers, event_subscriber::register_subscribers,
};
use axum::Router;
use bg_worker::queuer::Queuer;
use bg_worker::worker_manager::WorkerManager;
use infrastructure::shared::{
    config::{Config, Server},
    path::LOG_DIR,
    pg_pool,
};
use infrastructure::{
    migration, shared::kv::Kv, shared::pg_pool::PgPool, shared::provider::Provider,
};
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    str::FromStr as _,
    sync::Arc,
};
use tokio::{net::TcpListener, signal, sync::Notify, try_join};
use tracing::{info, warn};
use tracing_appender::{non_blocking::WorkerGuard, rolling::Rotation};
use tracing_subscriber::{
    EnvFilter, Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

pub mod cli;

pub async fn bootstrap(config: Config) -> Result<()> {
    let _guard = init_tracing(&config.log.level, config.log.rolling_kind.clone());
    let listener = build_listener(&config.server).await?;
    let provider = build_provider(config).await?;
    let (_, worker) = try_join!(initilize(&provider), build_worker_manager(&provider))?;
    let app = adapter::routing(WebState::new(provider.clone()));
    let notify_shutdown = Arc::new(Notify::new());
    let background_job_handle =
        tokio::spawn(start_background_worker(worker, notify_shutdown.clone()));
    let server_handle = tokio::spawn(start_http_server(listener, app, notify_shutdown.clone()));
    shutdown_signal().await;
    notify_shutdown.notify_waiters();
    let _ = tokio::join!(background_job_handle, server_handle);
    provider.provide::<PgPool>().close().await;
    info!("👋 Goodbye!");
    Ok(())
}

fn init_tracing(level: &str, rotation: Rotation) -> WorkerGuard {
    let directive = level;
    let file_appender =
        tracing_appender::rolling::RollingFileAppender::new(rotation, LOG_DIR.as_path(), "log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let terminal_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_filter(EnvFilter::new(directive));
    let rolling_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_line_number(true)
        .with_file(true)
        .with_ansi(false)
        .with_filter(EnvFilter::new(directive));
    tracing_subscriber::registry()
        .with(terminal_layer)
        .with(rolling_layer)
        .init();
    guard
}

async fn build_listener(server: &Server) -> Result<TcpListener> {
    let ip = Ipv4Addr::from_str(&server.bind)?;
    let addr = SocketAddrV4::new(ip, server.port);
    let listener = TcpListener::bind(addr).await?;
    Ok(listener)
}

async fn build_provider(config: Config) -> Result<Provider> {
    let pg_fut = pg_pool::try_new(&config.database);

    #[cfg(feature = "kv_redb")]
    let kv_fut = Kv::try_new(infrastructure::shared::path::DATA_DIR.join("data.redb"));
    #[cfg(feature = "kv_redis")]
    let kv_fut = Kv::try_new(&config.redis);

    #[cfg(feature = "kv_redb")]
    let (pg_pool, kv) = tokio::try_join!(pg_fut, kv_fut)?;

    #[cfg(feature = "kv_redis")]
    let (pg_pool, kv) = tokio::try_join!(pg_fut, kv_fut)?;

    let provider = {
        #[cfg(feature = "bg_faktory")]
        let queuer = Queuer::try_new(&config.faktory.url, &config.faktory.queue).await?;
        #[cfg(not(feature = "bg_faktory"))]
        let queuer = Queuer::new();
        Provider::builder()
            .pg_pool(pg_pool.clone())
            .kv(kv)
            .config(config)
            .queuer(queuer)
            .build()
    };
    Ok(provider)
}

async fn initilize(provider: &Provider) -> Result<()> {
    tokio::try_join!(migration::migrate(provider), async {
        register_subscribers(provider);
        anyhow::Ok(())
    })?;
    Ok(())
}

async fn build_worker_manager(provider: &Provider) -> Result<WorkerManager> {
    #[cfg(feature = "bg_faktory")]
    let mut worker_manager = {
        let config = &provider.provide::<Config>();
        let worker_manager = WorkerManager::new(&config.faktory.url, &config.faktory.queue);
        worker_manager
    };
    #[cfg(not(feature = "bg_faktory"))]
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
