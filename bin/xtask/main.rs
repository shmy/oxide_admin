use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::Result;
use clap::Parser;
use cruet::Inflector;
use inquire::{Select, Text};
use minijinja::context;
use tokio::{fs, process::Command};

use crate::{
    cli::{Cli, SubCommandArgs, SubCommands},
    database::{TableInfoTrait, postgres::Postgres},
    generate::{
        api::generate_api, application::generate_application, domain::generate_domain,
        repository::generate_repository,
    },
    template::TemplateEngine,
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
    match &cli.command {
        SubCommands::Scaffold(sub_command_args) => {
            generate_module(sub_command_args, GenerateModuleType::Scaffold, db).await
        }
        SubCommands::Api(sub_command_args) => {
            generate_module(sub_command_args, GenerateModuleType::Api, db).await
        }
        SubCommands::Application(sub_command_args) => {
            generate_module(sub_command_args, GenerateModuleType::Application, db).await
        }
        SubCommands::Domain(sub_command_args) => {
            generate_module(sub_command_args, GenerateModuleType::Domain, db).await
        }
        SubCommands::Repository(sub_command_args) => {
            generate_module(sub_command_args, GenerateModuleType::Repository, db).await
        }
        SubCommands::Ch => {
            generate_application_partials(APP_DIR.join("application"), "command").await
        }
        SubCommands::Qh => {
            generate_application_partials(APP_DIR.join("application"), "query").await
        }
        SubCommands::Job => {
            todo!()
        }
        SubCommands::Event => {
            let name = Text::new("What's name?").prompt()?.to_snake_case();
            let context = context! {
                name => name,
            };
            let template = TemplateEngine::from("partials/event").with_context(context);
            template
                .render_to(
                    APP_DIR
                        .join("application")
                        .join("shared")
                        .join("event_subscriber"),
                )
                .await?;
            Ok(())
        }
    }?;

    Command::new("cargo")
        .arg("fmt")
        .arg("--all")
        .current_dir(ROOT_DIR.as_path())
        .status()
        .await?;
    Ok(())
}

enum GenerateModuleType {
    Scaffold,
    Domain,
    Repository,
    Api,
    Application,
}

async fn generate_module(
    args: &SubCommandArgs,
    r#type: GenerateModuleType,
    db: Postgres,
) -> Result<()> {
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
    match r#type {
        GenerateModuleType::Api => generate_api(context).await?,
        GenerateModuleType::Application => generate_application(context).await?,
        GenerateModuleType::Domain => generate_domain(context).await?,
        GenerateModuleType::Repository => generate_repository(context).await?,
        GenerateModuleType::Scaffold => {
            tokio::try_join!(
                generate_api(context.clone()),
                generate_application(context.clone()),
                generate_domain(context.clone()),
                generate_repository(context.clone())
            )?;
        }
    }
    Ok(())
}

async fn list_modules(base: impl AsRef<Path>) -> Result<Vec<String>> {
    let mut entries = fs::read_dir(base).await?;
    let mut modules = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name() {
                let name = name.to_string_lossy().to_string();
                if name != "shared" {
                    modules.push(name);
                }
            }
        }
    }

    Ok(modules)
}

async fn generate_application_partials(base: PathBuf, dir_name: &str) -> Result<()> {
    let name = Text::new("What's name?").prompt()?.to_snake_case();
    let modules: Vec<String> = list_modules(&base).await?;
    let module = Select::new("What's your module choice?", modules).prompt()?;
    let sub_dir = base.join(&module).join(dir_name);
    fs::create_dir_all(&sub_dir).await?;
    let context = context! {
        name => name,
        module => module,
    };
    let template = TemplateEngine::from(&format!("partials/{dir_name}")).with_context(context);
    template.render_to(&sub_dir).await?;
    Ok(())
}
