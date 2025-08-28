use std::path::{Path, PathBuf};
use tokio::fs;

use anyhow::Result;
use cruet::Inflector as _;
use minijinja::{Environment, Value, context, path_loader};
use walkdir::WalkDir;

use crate::util::append_to_mod_file;

/// 模板引擎
pub struct TemplateEngine {
    template_dir: PathBuf,
    context: Value,
}

impl TemplateEngine {
    /// 创建模板引擎，模板路径为 xtask/templates/{subdir}
    pub fn from(subdir: &str) -> Self {
        let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("templates")
            .join(subdir);

        Self {
            template_dir,
            context: context! {}, // 默认空上下文
        }
    }

    /// 设置渲染上下文
    pub fn with_context(mut self, context: Value) -> Self {
        self.context = context;
        self
    }

    /// 渲染所有 `.j2` 模板并输出到指定目录（保持目录结构）
    pub async fn render_to(&self, output_dir: impl AsRef<Path>) -> Result<()> {
        let mut env = Self::build_env();
        env.set_loader(path_loader(&self.template_dir));

        for entry in WalkDir::new(&self.template_dir) {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("j2") {
                continue;
            }

            let rel_path = path.strip_prefix(&self.template_dir)?; // 去掉 `.j2`
            let output_path = output_dir.as_ref().join(rel_path.with_extension(""));

            let output_path =
                env.render_str(&output_path.to_string_lossy(), self.context.clone())?;
            let output_path = PathBuf::from(&output_path);
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            let template_name = rel_path.to_string_lossy().replace("\\", "/"); // 兼容 Windows 路径

            let rendered = env
                .get_template(&template_name)?
                .render(self.context.clone())?;

            let is_mod_rs = if let Some(filename) = output_path.file_name() {
                filename.to_string_lossy() == "mod.rs" || filename.to_string_lossy() == "lib.rs"
            } else {
                false
            };

            if is_mod_rs {
                if fs::try_exists(&output_path).await? {
                    append_to_mod_file(&output_path, &rendered).await?;
                } else {
                    fs::write(&output_path, rendered).await?;
                }
            } else {
                fs::write(&output_path, rendered).await?;
            }
            println!("✅Generated: {}", output_path.display());
        }

        Ok(())
    }
    fn build_env() -> Environment<'static> {
        let mut env = Environment::new();
        env.add_filter("pascal_case", |s: String| s.to_pascal_case());
        env.add_filter("uppercase", |s: String| s.to_uppercase());
        env.add_filter("pluralize", |s: String| s.to_plural());
        env.add_filter("is_copy_type", |ty: String| -> bool { is_copy_type(&ty) });
        env.add_filter("strip_raw_ident", |s: String| strip_raw_ident(&s));

        env
    }
}

fn is_copy_type(ty: &str) -> bool {
    matches!(
        ty.trim(),
        "i8" | "i16" | "i32" | "i64" | "i128"
        | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" | "isize"
        | "bool" | "char"
        // 可选加浮点
        | "f32" | "f64"
    )
}

fn strip_raw_ident(s: &str) -> String {
    s.strip_prefix("r#").unwrap_or(s).to_string()
}
