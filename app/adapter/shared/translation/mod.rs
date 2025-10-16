use domain::auth::value_object::{
    menu::{Menu, MenuTree},
    permission::{Permission, PermissionTree},
};
use i18n::LanguageIdentifier;
use serde::Serialize;
use utoipa::ToSchema;

use crate::i18n::LOCALES;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TranslatedMenuTree {
    pub key: Menu,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,
    #[serde(rename = "schemaApi", skip_serializing_if = "Option::is_none")]
    pub schema_api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(no_recursion)]
    pub children: Option<Vec<TranslatedMenuTree>>,
    #[serde(skip_serializing_if = "is_true")]
    pub visible: bool,
}

fn is_true(b: &bool) -> bool {
    *b
}

impl From<MenuTree> for TranslatedMenuTree {
    fn from(value: MenuTree) -> Self {
        Self {
            key: value.key,
            label: value.label.map(ToString::to_string),
            icon: value.icon.map(ToString::to_string),
            url: value.url.map(ToString::to_string),
            link: value.link.map(ToString::to_string),
            redirect: value.redirect.map(ToString::to_string),
            schema_api: value.schema_api.map(ToString::to_string),
            children: value
                .children
                .map(|d| d.into_iter().map(Into::into).collect()),
            visible: value.visible,
        }
    }
}

pub fn tranlate_menus(
    menus: Vec<MenuTree>,
    lang_id: &LanguageIdentifier,
) -> Vec<TranslatedMenuTree> {
    let items = menus
        .into_iter()
        .map(Into::into)
        .collect::<Vec<TranslatedMenuTree>>();
    tranlate_menus_inner(&items, lang_id)
}

fn tranlate_menus_inner(
    menus: &[TranslatedMenuTree],
    lang_id: &LanguageIdentifier,
) -> Vec<TranslatedMenuTree> {
    let mut menus = menus.to_vec();
    for menu in menus.iter_mut() {
        if let Some(label) = &menu.label {
            let query = i18n::Query::new(label);
            menu.label = LOCALES
                .query(lang_id, &query)
                .map(|message| message.value)
                .ok();
        }
        if let Some(children) = &menu.children {
            menu.children = Some(tranlate_menus_inner(children, lang_id));
        }
    }
    menus
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TranslatedPermissionTree {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Permission>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(no_recursion)]
    pub children: Option<Vec<TranslatedPermissionTree>>,
}

impl From<PermissionTree> for TranslatedPermissionTree {
    fn from(value: PermissionTree) -> Self {
        Self {
            label: value.label.to_string(),
            value: value.value,
            children: value
                .children
                .map(|d| d.to_vec().iter().cloned().map(Into::into).collect()),
        }
    }
}

pub fn tranlate_permissions(
    permissions: Vec<PermissionTree>,
    lang_id: &LanguageIdentifier,
) -> Vec<TranslatedPermissionTree> {
    let items = permissions
        .into_iter()
        .map(Into::into)
        .collect::<Vec<TranslatedPermissionTree>>();
    tranlate_permissions_inner(&items, lang_id)
}

fn tranlate_permissions_inner(
    permissions: &[TranslatedPermissionTree],
    lang_id: &LanguageIdentifier,
) -> Vec<TranslatedPermissionTree> {
    let mut permissions = permissions.to_vec();
    for permission in permissions.iter_mut() {
        let query = i18n::Query::new(&permission.label);
        permission.label = LOCALES
            .query(lang_id, &query)
            .map(|message| message.value)
            .ok()
            .unwrap_or(permission.label.to_string());
        if let Some(children) = &permission.children {
            permission.children = Some(tranlate_permissions_inner(children, lang_id));
        }
    }
    permissions
}
