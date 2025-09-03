use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use cruet::Inflector as _;
use minijinja::Environment;
use serde::{Deserialize, Serialize};

fn main() -> Result<()> {
    generate_subscribers()?;
    generate_jobs()?;
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=shared/event_subscriber");
    println!("cargo:rerun-if-changed=shared/background_job");
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
    let out_path = Path::new(&out_dir).join("subscribers.rs");

    fs::write(out_path, code)?;
    Ok(())
}

fn generate_jobs() -> Result<()> {
    let entries = read_rs("shared/background_job")?;
    let mut jobs = Vec::new();
    let mut stepped_jobs = Vec::new();
    let mut cron_jobs = Vec::new();
    for entry in entries {
        let stem = entry.file_stem().unwrap().to_string_lossy();
        let struct_name = stem.to_pascal_case();
        let file_content = fs::read_to_string(&entry)?;
        if !file_content.contains(&format!("struct {}", struct_name)) {
            continue;
        }
        let stem = stem.to_string();
        if stem.ends_with("stepped_job") {
            stepped_jobs.push(stem);
        } else if stem.ends_with("cron_job") {
            cron_jobs.push(stem);
        } else {
            jobs.push(stem);
        }
    }

    let env = build_env();
    let code = env.render_str(
        JOB_TEMPLATE,
        JobContext {
            jobs,
            cron_jobs,
            stepped_jobs,
        },
    )?;
    let out_dir = std::env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir).join("jobs.rs");

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

pub fn register_subscribers(provider: &Provider) {
    {%- for event in events %}

    EVENT_BUS.subscribe(provider.provide::<{{event}}::{{event | pascal_case}}>());
    tracing::info!("Event subscriber [{{event | pascal_case}}] has been registered");
    {%- endfor %}
}
"#;

#[derive(Serialize, Deserialize)]
pub struct JobContext {
    jobs: Vec<String>,
    stepped_jobs: Vec<String>,
    cron_jobs: Vec<String>,
}

const JOB_TEMPLATE: &str = r#"#[allow(unused_imports)]
use background_job::{BackgroundJobManager, JobStorage, SteppableStorage, SteppedJobStorage};
#[allow(unused_imports)]
use infrastructure::shared::provider::Provider;
#[allow(unused_imports)]
use infrastructure::shared::sqlite_pool::SqlitePool;
#[allow(unused_imports)]
use nject::injectable;
#[allow(unused_imports)]
use background_job::Storage;

pub fn register_jobs(manager: BackgroundJobManager, provider: &Provider) -> BackgroundJobManager {
    {%- for job in jobs %}

    let storage = JobStorage::new(provider.provide());
    let manager = manager.register::<{{job}}::{{job | pascal_case}}>(provider.provide(), storage);
    tracing::info!("Job [{{job | pascal_case}}] has been registered");
    {%- endfor %}

    {%- for job in stepped_jobs %}

    let storage = SteppedJobStorage::new(provider.provide());
    let manager = manager.register_stepped::<{{job}}::{{job | pascal_case}}>(storage);
    tracing::info!("Stepped job [{{job | pascal_case}}] has been registered");
    {%- endfor %}

    {%- for job in cron_jobs %}

    let manager = manager.register_cron::<{{job}}::{{job | pascal_case}}>(provider.provide());
    tracing::info!("Cron job [{{job | pascal_case}}] has been registered");
    {%- endfor %}
    manager
}
{%- for item in jobs %}

#[injectable]
pub struct {{item | pascal_case}}Dispatcher {
    #[inject(|sqlite_pool: SqlitePool| JobStorage::new(sqlite_pool) )]
    pub storage: JobStorage<{{item}}::{{item | pascal_case}}Params>,
}

impl {{item | pascal_case}}Dispatcher {
    pub async fn dispatch(&mut self, params: {{item}}::{{item | pascal_case}}Params) {
        if let Err(err) = self.storage.push(params).await {
            tracing::error!(%err, "Failed to push [{{item | pascal_case}}]");
        }
    }
}
{%- endfor %}

{%- for item in stepped_jobs %}

#[injectable]
pub struct {{item | pascal_case}}Dispatcher {
    #[inject(|sqlite_pool: SqlitePool| SteppedJobStorage::new(sqlite_pool) )]
    pub storage: SteppedJobStorage,
}

impl {{item | pascal_case}}Dispatcher {
    pub async fn dispatch(&mut self, params: {{item}}::{{item | pascal_case}}Begin) {
        if let Err(err) = self.storage.start_stepped(params).await {
            tracing::error!(%err, "Failed to start_stepped [{{item | pascal_case}}]");
        }
    }
}
{%- endfor %}
"#;
