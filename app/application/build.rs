use std::{fs, path::Path};

use anyhow::Result;
use cruet::Inflector as _;

fn main() -> Result<()> {
    registry_events()?;
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=shared/event_subscriber");
    Ok(())
}

fn registry_events() -> Result<()> {
    let mut entries = fs::read_dir("shared/event_subscriber")?;
    let mut contents = Vec::new();
    while let Some(Ok(entry)) = entries.next() {
        if entry.metadata()?.is_file() {
            let filename = entry.file_name();
            if filename == "mod.rs" {
                continue;
            }
            let stem = Path::new(&filename).file_stem().unwrap().to_string_lossy();
            let struct_name = stem.to_pascal_case();
            let file_content = fs::read_to_string(entry.path())?;
            if !file_content.contains(&format!("struct {}", struct_name)) {
                continue;
            }
            contents.push(format!(
                "\tEVENT_BUS.subscribe(provider.provide::<{}::{}>());",
                stem, struct_name
            ));
        }
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
    let out_path = Path::new(&out_dir).join("register_subscribers.rs");

    fs::write(out_path, code)?;
    Ok(())
}
