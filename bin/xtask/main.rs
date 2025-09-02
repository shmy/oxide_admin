use std::{path::PathBuf, sync::LazyLock};

use anyhow::Result;
use clap::Parser;
use cruet::Inflector;
use minijinja::context;
use tokio::process::Command;

use crate::{
    cli::{Cli, SubCommands},
    database::TableInfoTrait,
    generate::{
        api::generate_api, application::generate_application, domain::generate_domain,
        repository::generate_repository,
    },
};

mod cli;
mod database;
mod generate;
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
    let db = database::postgres::Postgres::new(&database_url).await?;
    let args = match &cli.command {
        SubCommands::Scaffold(sub_command_args) => sub_command_args,
        SubCommands::Api(sub_command_args) => sub_command_args,
        SubCommands::Application(sub_command_args) => sub_command_args,
        SubCommands::Domain(sub_command_args) => sub_command_args,
        SubCommands::Repository(sub_command_args) => sub_command_args,
    };
    let module = args.module.to_string();
    let entity = args.entity.to_string();
    let table = args.table.clone();

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
        SubCommands::Api(_) => generate_api(context).await?,
        SubCommands::Application(_) => generate_application(context).await?,
        SubCommands::Domain(_) => generate_domain(context).await?,
        SubCommands::Repository(_) => generate_repository(context).await?,
        SubCommands::Scaffold(_) => {
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
