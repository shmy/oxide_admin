use std::ops::Deref;

use bon::Builder;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(transparent)]
pub struct Menu(i32);

impl Menu {
    pub const fn new(code: i32) -> Self {
        Self(code)
    }
}

impl Deref for Menu {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Builder, ToSchema)]
pub struct MenuTree {
    pub key: Menu,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<&'static str>,
    #[serde(rename = "schemaApi", skip_serializing_if = "Option::is_none")]
    pub schema_api: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(no_recursion)]
    pub children: Option<Vec<MenuTree>>,
    #[serde(skip_serializing_if = "is_true")]
    #[builder(default = true)]
    pub visible: bool,
}

fn is_true(b: &bool) -> bool {
    *b
}

include!(concat!(env!("OUT_DIR"), "/menus.rs"));
