use std::{
    fs,
    path::{Path, PathBuf},
};

use cruet::Inflector as _;
use minijinja::Environment;
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    generate_subscribers()?;
    generate_bgworkers()?;
    generate_scheduler_jobs()?;
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=shared/event_subscriber");
    println!("cargo:rerun-if-changed=shared/bgworker");
    println!("cargo:rerun-if-changed=shared/scheduler_job");
    Ok(())
}

fn generate_subscribers() -> Result<()> {
    let entries = read_rs("shared/event_subscriber")?;
    let mut events = Vec::new();
    for entry in entries {
        let stem = entry.file_stem().unwrap().to_string_lossy();
        let struct_name = stem.to_pascal_case();
        let file_content = fs::read_to_string(&entry)?;
        if !file_content.contains(&format!("struct {}", struct_name)) {
            continue;
        }
        events.push(stem.to_string());
    }
    let env = build_env();
    let code = env.render_str(EVENT_TEMPLATE, EventContext { events })?;
    let out_dir = std::env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir).join("event_subscriber.rs");

    fs::write(out_path, code)?;
    Ok(())
}

fn generate_bgworkers() -> Result<()> {
    let entries = read_rs("shared/bgworker")?;
    let mut workers = Vec::new();
    for entry in entries {
        let stem = entry.file_stem().unwrap().to_string_lossy();
        let struct_name = stem.to_pascal_case();
        let file_content = fs::read_to_string(&entry)?;
        if !file_content.contains(&format!("struct {}", struct_name)) {
            continue;
        }
        let stem = stem.to_string();
        workers.push(stem);
    }

    let env = build_env();
    let code = env.render_str(BGWORKER_TEMPLATE, JobContext { jobs: workers })?;
    let out_dir = std::env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir).join("bgworker.rs");

    fs::write(out_path, code)?;
    Ok(())
}

fn generate_scheduler_jobs() -> Result<()> {
    let entries = read_rs("shared/scheduler_job")?;
    let mut jobs = Vec::new();
    for entry in entries {
        let stem = entry.file_stem().unwrap().to_string_lossy();
        let struct_name = stem.to_pascal_case();
        let file_content = fs::read_to_string(&entry)?;
        if !file_content.contains(&format!("struct {}", struct_name)) {
            continue;
        }
        let stem = stem.to_string();
        jobs.push(stem);
    }

    let env = build_env();
    let code = env.render_str(JOB_TEMPLATE, JobContext { jobs })?;
    let out_dir = std::env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir).join("scheduler_job.rs");

    fs::write(out_path, code)?;
    Ok(())
}

fn read_rs(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let entries = fs::read_dir(path)?;
    let items = entries
        .filter_map(|entry| {
            let Ok(f) = entry else {
                return None;
            };
            let Ok(meta) = f.metadata() else {
                return None;
            };
            if !meta.is_file() {
                return None;
            }
            Some(f.path())
        })
        .collect();
    Ok(items)
}

fn build_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_filter("pascal_case", |s: String| s.to_pascal_case());
    env.add_filter("uppercase", |s: String| s.to_uppercase());
    env.add_filter("screaming_snake_case", |s: String| {
        s.to_screaming_snake_case()
    });
    env
}

#[derive(Serialize, Deserialize)]
pub struct EventContext {
    events: Vec<String>,
}

const EVENT_TEMPLATE: &str = r#"#[allow(unused_imports)]
use crate::shared::event::EVENT_BUS;
#[allow(unused_imports)]
use infrastructure::shared::provider::Provider;

pub fn register_event_subscribers(provider: &Provider) {
    {%- for event in events %}

    EVENT_BUS.subscribe(provider.provide::<crate::shared::event_subscriber::{{event}}::{{event | pascal_case}}>());
    tracing::info!("Event subscriber [{{event | pascal_case}}] has been registered");
    {%- endfor %}
}
"#;

#[derive(Serialize, Deserialize)]
pub struct JobContext {
    jobs: Vec<String>,
}

const BGWORKER_TEMPLATE: &str = r#"use bg_worker_kit::WorkerManager;
use bg_worker_kit::{Storage, StorageBackend};
use infrastructure::shared::provider::Provider;
use bg_worker_kit::error::WorkerError;
use std::sync::OnceLock;

{%- for job in jobs %}
use crate::shared::bgworker::{{job}}::{{job | pascal_case}};
{%- endfor %}

{%- for job in jobs %}
static {{job | screaming_snake_case}}: OnceLock<StorageBackend<{{job | pascal_case}}>> = OnceLock::new();
{%- endfor %}

{%- for job in jobs %}
pub struct {{job | pascal_case}}Storage;

impl {{job | pascal_case}}Storage {
  
    pub async fn push(job: {{job | pascal_case}}) -> Result<(), WorkerError> {
        if let Some(backend) = {{job | screaming_snake_case}}.get() {
            backend.clone().push(job).await?;
        }
        Ok(())
    }
}

{%- endfor %}
pub fn register_bgworkers(
    manager: WorkerManager,
    provider: Provider) -> WorkerManager {
    {%- for job in jobs %}
    let (manager, backend) = manager.register::<{{job | pascal_case}}>(
        provider.clone(),
    );
    {{job | screaming_snake_case}}.set(backend).expect("Failed to set backend");
    tracing::info!("Worker [{{job | pascal_case}}] has been registered");
    {%- endfor %}
   
    manager

}
"#;

const JOB_TEMPLATE: &str = r#"#[allow(unused_imports)]
use crate::error::ApplicationResult;
#[allow(unused_imports)]
use infrastructure::shared::provider::Provider;
#[allow(unused_imports)]
use sched_kit::cron_tab::CronTab;
#[allow(unused_imports)]
use sched_kit::ScheduledJob;
#[allow(unused_imports)]
use infrastructure::port::sched_receiver_impl::SchedReceiverImpl;
#[allow(unused_imports)]
use infrastructure::shared::config::ConfigRef;

pub const SCHEDULER_JOBS: &[SchedulerJob] = &[
{%- for job in jobs %}
    SchedulerJob {
        key: "{{job}}",
        name: crate::shared::scheduler_job::{{job}}::{{job | pascal_case}}::NAME,
        expr: crate::shared::scheduler_job::{{job}}::{{job | pascal_case}}::EXPR,
    },
{%- endfor %}
];

pub async fn register_scheduled_jobs(
    #[allow(unused)]
    cron_tab: &mut CronTab<SchedReceiverImpl>,
    #[allow(unused)]
    provider: &Provider,
) -> ApplicationResult<()> {

    {%- for job in jobs %}

    let job = provider.provide::<crate::shared::scheduler_job::{{job}}::{{job | pascal_case}}>();
    cron_tab.add("{{job}}", job).await?;
    {%- endfor %}
    Ok(())
}
"#;
