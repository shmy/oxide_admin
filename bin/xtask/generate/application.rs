use std::{fs, path::Path};

use anyhow::Result;
use cruet::Inflector as _;
use minijinja::Value;
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, Item, ItemEnum, parse_file, parse2};

use crate::{APP_DIR, template::TemplateEngine};

pub async fn generate_application(context: Value) -> Result<()> {
    let module = context
        .get_item(&Value::from("module"))
        .unwrap()
        .to_string();
    let template = TemplateEngine::from("application").with_context(context);
    template.render_to(APP_DIR.join("application")).await?;
    append_domain_event(
        APP_DIR
            .join("application")
            .join("shared")
            .join("event.rs")
            .as_path(),
        &module,
    )?;
    Ok(())
}

fn append_domain_event(path: &Path, module: &str) -> Result<()> {
    let code = std::fs::read_to_string(path)?;
    let mut syntax = parse_file(&code)?;

    let enum_ident = Ident::new("Event", Span::call_site());
    let event_ident = Ident::new(&format!("{}", module.to_pascal_case()), Span::call_site());
    let domain_event_ident = Ident::new(
        &format!("{}Event", module.to_pascal_case()),
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
            let event_use_statement = format!(
                "use domain::{}::event::{}Event;",
                module,
                module.to_pascal_case()
            );
            already_exists = variants.iter().any(|v| v.ident == event_ident);
            if !already_exists {
                if !code.contains(&event_use_statement) {
                    import_code.push(event_use_statement);
                }
                let variant_tokens = quote! {
                     #event_ident(#domain_event_ident)
                };
                let variant: syn::Variant = syn::parse2(variant_tokens)?;
                variants.push(variant);
            }
        }
    }
    if !already_exists {
        let impl_tokens = quote! {
            impl From<#domain_event_ident> for Event {
                fn from(value: #domain_event_ident) -> Self {
                    Self :: #event_ident(value)
                }
            }
        };

        let impl_item: Item = parse2(impl_tokens)?;
        syntax.items.push(impl_item);
    }

    let formatted = prettyplease::unparse(&syntax);
    fs::write(path, format!("{}\n{}", import_code.join("\n"), formatted))?;
    Ok(())
}
