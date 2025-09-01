use adapter::WebState;
use anyhow::Result;
use application::shared::{
    background_job::{
        BackgroundJobs, delete_expired_kv_job::DeleteExpiredKvJob,
        delete_outdate_log_dir_job::DeleteOutdateLogDirJob,
        delete_outdate_temp_dir_job::DeleteOutdateTempDirJob,
        delete_unused_file_job::DeleteUnusedFileJob, register_jobs,
    },
    event_subscriber::register_subscribers,
};
use axum::Router;
use background_job::BackgroundJobManager;
use background_job::Storage;
use infrastructure::shared::{
    config::Config,
    path::{DATA_DIR, LOG_DIR},
    pool,
};
use infrastructure::{migration, shared::kv::Kv, shared::pool::Pool, shared::provider::Provider};
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    str::FromStr as _,
    sync::Arc,
};
use tokio::{net::TcpListener, signal, sync::Notify};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};
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
    let (pool, kv) = tokio::try_join!(pool::try_new(&config.database), async {
        Kv::try_new(DATA_DIR.join("data.redb"))
    })?;

    let provider = Provider::builder()
        .pool(pool.clone())
        .kv(kv)
        .config(config)
        .build();
    migration::migrate(&provider).await?;
    register_subscribers(&provider);
    let app = adapter::routing(WebState::new(provider.clone()));
    let notify_shutdown = Arc::new(Notify::new());
    let server_handle = tokio::spawn(start_http_server(
        listener,
        app,
        provider.clone(),
        notify_shutdown.clone(),
    ));
    let (manager, jobs) = build_background_job(&provider).await?;
    let scheduler_handle = tokio::spawn(start_scheduler(jobs, notify_shutdown.clone()));
    let background_job_handle =
        tokio::spawn(start_background_job(manager, notify_shutdown.clone()));
    shutdown_signal().await;
    notify_shutdown.notify_waiters();
    let _ = tokio::join!(server_handle, scheduler_handle, background_job_handle);
    info!("👋 Goodbye!");
    Ok(())
}

fn init_tracing(level: &str, rotation: Rotation) -> WorkerGuard {
    let directive = level;
    let file_appender =
        tracing_appender::rolling::RollingFileAppender::new(rotation, LOG_DIR.as_path(), "log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let terminal_layer = tracing_subscriber::fmt::layer().with_filter(EnvFilter::new(directive));
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
    provider.provide::<Pool>().close().await;
    Ok(())
}

async fn start_scheduler(jobs: BackgroundJobs, notify: Arc<Notify>) -> Result<()> {
    let mut sched = JobScheduler::new().await?;
    let cloned_jobs = jobs.clone();
    sched
        .add(Job::new_async("every 1 hour", move |_uuid, _l| {
            let mut cloned_jobs = cloned_jobs.clone();
            Box::pin(async move {
                if let Err(e) = cloned_jobs
                    .delete_expired_kv_job
                    .push(DeleteExpiredKvJob)
                    .await
                {
                    error!("Failed to enqueue DeleteExpiredKvJob: {:?}", e);
                }
            })
        })?)
        .await?;

    let cloned_jobs = jobs.clone();
    sched
        .add(Job::new_async("at 00:01 am", move |_uuid, _l| {
            let mut cloned_jobs = cloned_jobs.clone();
            Box::pin(async move {
                if let Err(e) = cloned_jobs
                    .delete_unused_file_job
                    .push(DeleteUnusedFileJob)
                    .await
                {
                    error!("Failed to enqueue DeleteUnusedFileJob: {:?}", e);
                }
            })
        })?)
        .await?;

    let cloned_jobs = jobs.clone();
    sched
        .add(Job::new_async("at 01:01 am", move |_uuid, _l| {
            let mut cloned_jobs = cloned_jobs.clone();
            Box::pin(async move {
                if let Err(e) = cloned_jobs
                    .delete_outdate_temp_dir_job
                    .push(DeleteOutdateTempDirJob)
                    .await
                {
                    error!("Failed to enqueue DeleteOutdateTempDirJob: {:?}", e);
                }
            })
        })?)
        .await?;

    let cloned_jobs = jobs.clone();
    sched
        .add(Job::new_async("at 02:01 am", move |_uuid, _l| {
            let mut cloned_jobs = cloned_jobs.clone();
            Box::pin(async move {
                if let Err(e) = cloned_jobs
                    .delete_outdate_log_dir_job
                    .push(DeleteOutdateLogDirJob)
                    .await
                {
                    error!("Failed to enqueue DeleteOutdateLogDirJob: {:?}", e);
                }
            })
        })?)
        .await?;
    sched.start().await?;
    notify.notified().await;
    info!("Received shutdown signal, shutting down scheduler...");
    sched.shutdown().await?;
    info!("Scheduler shutdown complete");
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

async fn build_background_job(
    provider: &Provider,
) -> Result<(BackgroundJobManager, BackgroundJobs)> {
    let manager = BackgroundJobManager::try_new(DATA_DIR.join("data.sqlite")).await?;
    let manager = manager.migrate().await?;
    register_jobs(manager, provider)
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
