use anyhow::Result;
use minijinja::Value;

use crate::{ROOT_DIR, template::TemplateEngine};

pub async fn generate_frontend(context: Value) -> Result<()> {
    let template = TemplateEngine::from("frontend").with_context(context);
    let pages_dir = ROOT_DIR.join("frontend").join("src").join("pages");
    template.render_to(pages_dir).await?;
    Ok(())
}
