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
    generate_background_workers()?;
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

fn generate_background_workers() -> Result<()> {
    let entries = read_rs("shared/background_worker")?;
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
    let out_path = Path::new(&out_dir).join("workers.rs");

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
}

const JOB_TEMPLATE: &str = r#"#[allow(unused_imports)]
use bg_worker_kit::worker_manager::WorkerManager;
#[allow(unused_imports)]
use bg_worker_kit::queuer::Queuer;
#[allow(unused_imports)]
use bg_worker_kit::JobRunner;
#[allow(unused_imports)]
use infrastructure::shared::provider::Provider;
#[allow(unused_imports)]
use anyhow::Result;

#[allow(unused_imports)]
use nject::injectable;

pub fn register_workers(worker_manager: &mut WorkerManager, provider: &Provider) {
    {%- for job in jobs %}

    worker_manager.register("{{job}}", provider.provide::<{{job}}::{{job | pascal_case}}>());
    tracing::info!("Job [{{job | pascal_case}}] has been registered");
    {%- endfor %}

}
{%- for item in jobs %}

#[derive(Debug, Clone)]
#[injectable]
pub struct {{item | pascal_case}}Queuer {
     queuer: Queuer,
}

impl {{item | pascal_case}}Queuer {
    pub async fn enqueue(&self, params: <{{item}}::{{item | pascal_case}} as JobRunner>::Params) -> Result<()> {
        self.queuer.enqueue("{{item}}", params).await
    }
}
{%- endfor %}
"#;
