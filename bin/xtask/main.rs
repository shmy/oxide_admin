use std::{path::PathBuf, sync::LazyLock};

use anyhow::Result;
use clap::Parser;
use cruet::Inflector;
use minijinja::{Value, context};
use tokio::process::Command;

use crate::{
    cli::{Cli, SubCommands},
    database::TableInfoTrait,
    template::TemplateEngine,
};

mod cli;
mod database;
mod template;
mod util;

static ROOT_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../")
        .join("../")
        .canonicalize()
        .expect("root dir")
});

static APP_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| ROOT_DIR.join("app").canonicalize().expect("app dir"));

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL provided");
    let db = database::sqlite::Sqlite::new(&database_url).await?;
    let module = cli.module;
    let entity = cli.entity;
    let table = cli.table;

    let table_name = table.unwrap_or_else(|| entity.to_plural());
    let fields = db.table_info(&table_name).await?;

    let domain_fields = fields
        .iter()
        .filter_map(|c| {
            let name = &c.name;
            if name == "id" || name == "updated_at" || name == "created_at" {
                return None;
            }
            let r#type = &c.r#type;
            Some(serde_json::json!({
                "name": name,
                "type": r#type,
            }))
        })
        .collect::<Vec<_>>();

    let context = context! {
        module => module,
        entity => entity,
        table_name => table_name,
        fields => fields,
        domain_fields => domain_fields,
    };
    match cli.command {
        SubCommands::Api => generate_api(context).await?,
        SubCommands::Application => generate_application(context).await?,
        SubCommands::Domain => generate_domain(context).await?,
        SubCommands::Repository => generate_repository(context).await?,
        SubCommands::Scaffold => {
            tokio::try_join!(
                generate_api(context.clone()),
                generate_application(context.clone()),
                generate_domain(context.clone()),
                generate_repository(context.clone())
            )?;
        }
    }
    Command::new("cargo")
        .arg("fmt")
        .current_dir(ROOT_DIR.as_path())
        .status()
        .await?;
    Ok(())
}

async fn generate_api(context: Value) -> Result<()> {
    let template = TemplateEngine::from("api").with_context(context);
    template
        .render_to(APP_DIR.join("adapter").join("api"))
        .await?;
    Ok(())
}

async fn generate_application(context: Value) -> Result<()> {
    let template = TemplateEngine::from("application").with_context(context);
    template.render_to(APP_DIR.join("application")).await?;
    Ok(())
}

async fn generate_domain(context: Value) -> Result<()> {
    let template = TemplateEngine::from("domain").with_context(context);
    template.render_to(APP_DIR.join("domain")).await?;
    Ok(())
}

async fn generate_repository(context: Value) -> Result<()> {
    let template = TemplateEngine::from("repository").with_context(context);
    template
        .render_to(APP_DIR.join("infrastructure").join("repository"))
        .await?;
    Ok(())
}
