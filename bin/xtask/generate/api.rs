use std::{fs, path::Path};

use anyhow::Result;
use cruet::Inflector as _;
use minijinja::Value;
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, Item, ItemEnum, parse_file};

use crate::{APP_DIR, template::TemplateEngine};

pub async fn generate_api(context: Value) -> Result<()> {
    let module = context
        .get_item(&Value::from("module"))
        .unwrap()
        .to_string();
    let template = TemplateEngine::from("api").with_context(context);
    template
        .render_to(APP_DIR.join("adapter").join("api"))
        .await?;
    append_domain_error(
        APP_DIR
            .join("adapter")
            .join("shared")
            .join("error.rs")
            .as_path(),
        &module,
    )?;
    Ok(())
}

fn append_domain_error(path: &Path, module: &str) -> Result<()> {
    let code = std::fs::read_to_string(path)?;
    let mut syntax = parse_file(&code)?;

    let enum_ident = Ident::new("WebError", Span::call_site());
    let error_ident = Ident::new(&module.to_pascal_case().to_string(), Span::call_site());
    let domain_error_ident = Ident::new(
        &format!("{}Error", module.to_pascal_case()),
        Span::call_site(),
    );
    let mut import_code = Vec::new();
    let mut already_exists = false;

    for item in &mut syntax.items {
        if let Item::Enum(ItemEnum {
            ident, variants, ..
        }) = item
            && ident == &enum_ident
        {
            let error_use_statement = format!(
                "use domain::{}::error::{}Error;",
                module,
                module.to_pascal_case()
            );
            already_exists = variants.iter().any(|v| v.ident == error_ident);
            if !already_exists {
                if !code.contains(&error_use_statement) {
                    import_code.push(error_use_statement);
                }
                let variant_tokens = quote! {
                     #[error("{0}")]
                     #error_ident(#[from] #domain_error_ident)
                };
                let variant: syn::Variant = syn::parse2(variant_tokens)?;
                variants.push(variant);
            }
        }
    }
    if !already_exists {
        // let impl_tokens = quote! {
        //     impl From<#domain_event_ident> for Event {
        //         fn from(value: #domain_event_ident) -> Self {
        //             Self :: #event_ident(value)
        //         }
        //     }
        // };

        // let impl_item: Item = parse2(impl_tokens)?;
        // syntax.items.push(impl_item);
    }

    let formatted = prettyplease::unparse(&syntax);
    fs::write(path, format!("{}\n{}", import_code.join("\n"), formatted))?;
    Ok(())
}
