use anyhow::Result;
use cruet::Inflector as _;
use minijinja::Value;
use proc_macro2::Span;
use quote::quote;
use serde::{Deserialize, Serialize};
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
    )?;
    append_event(
        APP_DIR
            .join("domain")
            .join(&module)
            .join("event.rs")
            .as_path(),
        &module,
        &entity,
    )?;
    append_permissions(
        APP_DIR
            .join("domain")
            .join("system")
            .join("permissions.yaml")
            .as_path(),
        &module,
        &entity,
    )?;
    Ok(())
}

fn append_error(path: &Path, module: &str, entity: &str) -> Result<()> {
    if !fs::exists(path)? {
        let basic = format!(
            r#"
            #[derive(Debug, Clone, thiserror::Error)]
            pub enum {}Error {{
                #[error("{{0}}")]
                Sqlx(String),
            }}

            impl From<sqlx::Error> for {}Error {{
                fn from(err: sqlx::Error) -> Self {{
                    tracing::error!(%err, "sqlx error");
                    let message = err.to_string();
                    Self::Sqlx(message)
                }}
            }}
            "#,
            module.to_pascal_case(),
            module.to_pascal_case(),
        );
        fs::write(path, basic)?;
    }
    let code = fs::read_to_string(path)?;
    let mut syntax = parse_file(&code)?;

    let enum_name = format!("{}Error", module.to_pascal_case());
    let enum_ident = Ident::new(&enum_name, Span::call_site());
    let variant_notfound = Ident::new(
        &format!("{}NotFound", entity.to_pascal_case()),
        Span::call_site(),
    );
    let notfound_error_message = format!("{} not exists", entity.to_pascal_case());

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

fn append_event(path: &Path, module: &str, entity: &str) -> Result<()> {
    if !fs::exists(path)? {
        let basic = format!(
            r#"
            use crate::shared::event_util::UpdatedEvent;

            #[derive(Debug, Clone)]
            pub enum {}Event {{
            }}
            "#,
            module.to_pascal_case()
        );
        fs::write(path, basic)?;
    }
    let code = fs::read_to_string(path)?;
    let mut syntax = parse_file(&code)?;

    let enum_name = format!("{}Event", module.to_pascal_case());
    let enum_ident = Ident::new(&enum_name, Span::call_site());
    let entity_ty: syn::Type = parse_str(&entity.to_pascal_case().to_string())?;
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

            let already_exists = variants.iter().any(|v| v.ident == variant_created);
            if !already_exists {
                if !code.contains(&entity_use_statement) {
                    import_code.push(entity_use_statement);
                }
                let variant_tokens = quote! {
                    #variant_created { items: Vec<#entity_ty> }
                };
                let variant: syn::Variant = syn::parse2(variant_tokens)?;
                variants.push(variant);
            }

            let already_exists = variants.iter().any(|v| v.ident == variant_updated);
            if !already_exists {
                let variant_tokens = quote! {
                    #variant_updated { items: Vec<UpdatedEvent<#entity_ty>> }
                };
                let variant: syn::Variant = syn::parse2(variant_tokens)?;
                variants.push(variant);
            }
            let already_exists = variants.iter().any(|v| v.ident == variant_deleted);
            if !already_exists {
                let variant_tokens = quote! {
                    #variant_deleted { items: Vec<#entity_ty> }
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

fn append_permissions(path: &Path, module: &str, entity: &str) -> Result<()> {
    let content = fs::read(path)?;
    fn insert_entity(item: &mut PermissionItem, entity: &str) {
        let value = 1000;
        item.children.extend_from_slice(&[PermissionItem {
            label: entity.to_pascal_case(),
            key: entity.to_string(),
            value: None,
            children: vec![
                PermissionItem {
                    label: "Read".to_string(),
                    key: "read".to_string(),
                    value: Some(value),
                    children: vec![],
                },
                PermissionItem {
                    label: "Create".to_string(),
                    key: "create".to_string(),
                    value: Some(value + 1),
                    children: vec![],
                },
                PermissionItem {
                    label: "Update".to_string(),
                    key: "update".to_string(),
                    value: Some(value + 2),
                    children: vec![],
                },
                PermissionItem {
                    label: "Delete".to_string(),
                    key: "delete".to_string(),
                    value: Some(value + 3),
                    children: vec![],
                },
            ],
        }]);
    }

    fn insert_module(items: &mut Vec<PermissionItem>, module: &str) {
        items.push(PermissionItem {
            label: module.to_pascal_case(),
            key: module.to_string(),
            value: None,
            children: vec![],
        });
    }
    let mut tree: Vec<PermissionItem> = serde_yaml::from_slice(&content)?;
    let mut module_exisit: bool = false;
    let mut entity_exisit: bool = false;
    for ele in tree.iter_mut() {
        if ele.key == module {
            module_exisit = true;
            for ele in ele.children.iter_mut() {
                if ele.key == entity {
                    insert_entity(ele, entity);
                    entity_exisit = true;
                    break;
                }
            }
            if !entity_exisit {
                insert_entity(ele, entity);
                entity_exisit = true;
            }
        }
    }
    if !module_exisit {
        insert_module(&mut tree, module);
    }
    if !entity_exisit {
        insert_entity(tree.last_mut().unwrap(), entity);
    }
    let content = serde_yaml::to_string(&tree)?;
    fs::write(path, content)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionItem {
    label: String,
    key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    value: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    children: Vec<PermissionItem>,
}
