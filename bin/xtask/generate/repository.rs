use anyhow::Result;
use minijinja::Value;

use crate::{APP_DIR, template::TemplateEngine};

pub async fn generate_repository(context: Value) -> Result<()> {
    let template = TemplateEngine::from("repository").with_context(context);
    template
        .render_to(APP_DIR.join("infrastructure").join("repository"))
        .await?;
    Ok(())
}
