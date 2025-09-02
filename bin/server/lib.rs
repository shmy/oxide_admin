use adapter::WebState;
use anyhow::Result;
use application::shared::{background_job::register_jobs, event_subscriber::register_subscribers};
use axum::Router;
use background_job::BackgroundJobManager;
use infrastructure::shared::{
    config::Config,
    path::{DATA_DIR, LOG_DIR},
    pg_pool, sqlite_pool,
};
use infrastructure::{
    migration, shared::kv::Kv, shared::pg_pool::PgPool, shared::provider::Provider,
};
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    str::FromStr as _,
    sync::Arc,
};
use tokio::{net::TcpListener, signal, sync::Notify};
use tracing::{info, warn};
use tracing_appender::{non_blocking::WorkerGuard, rolling::Rotation};
use tracing_subscriber::{
    EnvFilter, Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

pub mod cli;

pub async fn bootstrap(config: Config) -> Result<()> {
    let _guard = init_tracing(&config.log.level, config.log.rolling_kind.clone());
    let ip = Ipv4Addr::from_str(&config.server.bind)?;
    let addr = SocketAddrV4::new(ip, config.server.port);
    let listener = TcpListener::bind(addr).await?;
    let (pg_pool, sqlite_pool, kv) = tokio::try_join!(
        pg_pool::try_new(&config.database),
        sqlite_pool::try_new(DATA_DIR.join("data.sqlite")),
        async { Kv::try_new(DATA_DIR.join("data.redb")) }
    )?;

    let provider = Provider::builder()
        .pg_pool(pg_pool.clone())
        .sqlite_pool(sqlite_pool.clone())
        .kv(kv)
        .config(config)
        .build();
    let (_, _, manager) = tokio::try_join!(
        migration::migrate(&provider),
        async {
            register_subscribers(&provider);
            anyhow::Ok(())
        },
        async {
            let manager = BackgroundJobManager::new(sqlite_pool);
            let manager = manager.migrate().await?;
            let manager = register_jobs(manager, &provider);
            anyhow::Ok(manager)
        }
    )?;

    let notify_shutdown = Arc::new(Notify::new());
    let background_job_handle =
        tokio::spawn(start_background_job(manager, notify_shutdown.clone()));
    let app = adapter::routing(WebState::new(provider.clone()));
    let server_handle = tokio::spawn(start_http_server(
        listener,
        app,
        provider.clone(),
        notify_shutdown.clone(),
    ));
    shutdown_signal().await;
    notify_shutdown.notify_waiters();
    let _ = tokio::join!(background_job_handle, server_handle);
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

async fn start_http_server(
    listener: TcpListener,
    app: Router,
    provider: Provider,
    notify: Arc<Notify>,
) -> Result<()> {
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
    provider.provide::<PgPool>().close().await;
    Ok(())
}

async fn start_background_job(manager: BackgroundJobManager, notify: Arc<Notify>) -> Result<()> {
    let shutdown = async move {
        notify.notified().await;
        info!("Received shutdown signal, shutting down background job...");
        Ok(())
    };
    manager.run_with_signal(shutdown).await?;
    info!("Background job shutdown complete");
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
