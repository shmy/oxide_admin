use anyhow::Result;
use cruet::Inflector as _;
use minijinja::Value;
use proc_macro2::Span;
use quote::quote;
use std::{fs, path::Path};
use syn::{Ident, Item, ItemEnum, parse_file, parse_str};

use crate::{APP_DIR, template::TemplateEngine};

pub async fn generate_domain(context: Value) -> Result<()> {
    let module = context
        .get_item(&Value::from("module"))
        .unwrap()
        .to_string();
    let entity = context
        .get_item(&Value::from("entity"))
        .unwrap()
        .to_string();
    let template = TemplateEngine::from("domain").with_context(context);
    template.render_to(APP_DIR.join("domain")).await?;
    append_error(
        APP_DIR
            .join("domain")
            .join(&module)
            .join("error.rs")
            .as_path(),
        &module,
        &entity,
    )
    .await?;
    append_event(
        APP_DIR
            .join("domain")
            .join(&module)
            .join("event.rs")
            .as_path(),
        &module,
        &entity,
    )
    .await?;
    Ok(())
}

async fn append_error(path: &Path, module: &str, entity: &str) -> Result<()> {
    if !fs::exists(&path).map(|d| d)? {
        let basic = format!(
            r#"
            #[derive(Debug, thiserror::Error)]
            pub enum {}Error {{
                #[error("数据库错误: {{0}}")]
                DatabaseError(#[from] sqlx::Error),
            }}
            "#,
            module.to_pascal_case()
        );
        fs::write(&path, basic)?;
    }
    let code = fs::read_to_string(path)?;
    let mut syntax = parse_file(&code)?;

    let enum_name = format!("{}Error", module.to_pascal_case());
    let enum_ident = Ident::new(&enum_name, Span::call_site());
    let variant_notfound = Ident::new(
        &format!("{}NotFound", entity.to_pascal_case()),
        Span::call_site(),
    );
    let notfound_error_message = format!("{} 不存在", entity.to_pascal_case());

    for item in &mut syntax.items {
        if let Item::Enum(ItemEnum {
            ident, variants, ..
        }) = item
            && ident == &enum_ident
        {
            let already_exists = variants.iter().any(|v| v.ident == variant_notfound);
            if !already_exists {
                let variant_tokens = quote! {
                    #[error(#notfound_error_message)]
                    #variant_notfound
                };
                let variant: syn::Variant = syn::parse2(variant_tokens)?;
                variants.push(variant);
            }
        }
    }
    let formatted = prettyplease::unparse(&syntax);
    fs::write(path, formatted)?;

    Ok(())
}

async fn append_event(path: &Path, module: &str, entity: &str) -> Result<()> {
    if !fs::exists(&path).map(|d| d)? {
        let basic = format!(
            r#"
            use crate::shared::event_util::UpdatedEvent;

            #[derive(Debug, Clone)]
            pub enum {}Event {{
            }}
            "#,
            module.to_pascal_case()
        );
        fs::write(&path, basic)?;
    }
    let code = fs::read_to_string(path)?;
    let mut syntax = parse_file(&code)?;

    let enum_name = format!("{}Event", module.to_pascal_case());
    let enum_ident = Ident::new(&enum_name, Span::call_site());
    let entity_ty: syn::Type = parse_str(&format!("{}", entity.to_pascal_case()))?;
    let variant_created = Ident::new(
        &format!("{}Created", entity.to_plural().to_pascal_case()),
        Span::call_site(),
    );

    let variant_updated = Ident::new(
        &format!("{}Updated", entity.to_plural().to_pascal_case()),
        Span::call_site(),
    );

    let variant_deleted = Ident::new(
        &format!("{}Deleted", entity.to_plural().to_pascal_case()),
        Span::call_site(),
    );

    let mut import_code = Vec::new();

    for item in &mut syntax.items {
        if let Item::Enum(ItemEnum {
            ident, variants, ..
        }) = item
            && ident == &enum_ident
        {
            let entity_use_statement = format!(
                "use crate::{}::entity::{}::{};",
                module,
                entity,
                entity.to_pascal_case()
            );
            if !code.contains(&entity_use_statement) {
                import_code.push(entity_use_statement);
            }
            let already_exists = variants.iter().any(|v| v.ident == variant_created);
            if !already_exists {
                let variant_tokens = quote! {
                    #variant_created { items: Vec<#entity_ty> }
                };
                let variant: syn::Variant = syn::parse2(variant_tokens)?;
                variants.push(variant);
            }

            let already_exists = variants.iter().any(|v| v.ident == variant_updated);
            if !already_exists {
                let variant_tokens = quote! {
                    #variant_updated { items: Vec<#entity_ty> }
                };
                let variant: syn::Variant = syn::parse2(variant_tokens)?;
                variants.push(variant);
            }
            let already_exists = variants.iter().any(|v| v.ident == variant_deleted);
            if !already_exists {
                let variant_tokens = quote! {
                    #variant_deleted { items: Vec<UpdatedEvent<#entity_ty>> }
                };
                let variant: syn::Variant = syn::parse2(variant_tokens)?;
                variants.push(variant);
            }
        }
    }
    let formatted = prettyplease::unparse(&syntax);
    fs::write(path, format!("{}\n{}", import_code.join("\n"), formatted))?;

    Ok(())
}
