use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use cruet::Inflector as _;

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
    let mut contents = Vec::new();
    for entry in entries {
        let stem = entry.file_stem().unwrap().to_string_lossy();
        let struct_name = stem.to_pascal_case();
        let file_content = fs::read_to_string(&entry)?;
        if !file_content.contains(&format!("struct {}", struct_name)) {
            continue;
        }
        contents.push(format!(
            r#"    EVENT_BUS.subscribe(provider.provide::<{}::{}>());
    tracing::info!("Event subscriber [{}] has been registered");"#,
            stem, struct_name, struct_name
        ));
    }
    let code = format!(
        r#"use crate::shared::event::EVENT_BUS;
use infrastructure::shared::provider::Provider;
pub fn register_subscribers(provider: &Provider) {{
{}
}}"#,
        contents.join("\n")
    );
    let out_dir = std::env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir).join("subscribers.rs");

    fs::write(out_path, code)?;
    Ok(())
}

fn generate_jobs() -> Result<()> {
    let entries = read_rs("shared/background_job")?;
    let mut struct_stems = Vec::new();
    let mut field_stems = Vec::new();
    let mut register_stems = Vec::new();
    for entry in entries {
        let stem = entry.file_stem().unwrap().to_string_lossy();
        let struct_name = stem.to_pascal_case();
        let file_content = fs::read_to_string(&entry)?;
        if !file_content.contains(&format!("struct {}", struct_name)) {
            continue;
        }
        struct_stems.push(format!(
            "\t pub {}: JobStorage<{}::{}>,",
            stem, stem, struct_name
        ));
        field_stems.push(format!("\t\t{}", stem));
        register_stems.push(format!(
            r#"    let (manager, {}) = manager.register::<{}::{}>(provider.clone());
    tracing::info!("Background job [{}] has been registered");"#,
            stem, stem, struct_name, struct_name
        ));
    }
    let jobs = format!(
        r#"use anyhow::Result;
use background_job::{{BackgroundJobManager, JobStorage}};
use infrastructure::shared::provider::Provider;
#[derive(Clone, Debug)]
pub struct Jobs {{
{}
}}"#,
        struct_stems.join("\n")
    );

    let func = format!(
        r#"pub fn register_jobs(manager: BackgroundJobManager, provider: &Provider) -> Result<(BackgroundJobManager, Jobs)> {{
{}
    Ok((
        manager,
        Jobs {{
    {}
        }},
    ))
}}"#,
        register_stems.join("\n"),
        field_stems.join(",\n")
    );
    let out_dir = std::env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir).join("jobs.rs");

    fs::write(out_path, format!("{}\n{}", jobs, func))?;
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
