use anyhow::Result;
use minijinja::Value;

use crate::{APP_DIR, template::TemplateEngine};

pub async fn generate_api(context: Value) -> Result<()> {
    let template = TemplateEngine::from("api").with_context(context);
    template
        .render_to(APP_DIR.join("adapter").join("api"))
        .await?;
    Ok(())
}
