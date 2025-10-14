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
    pub label: Option<&'static str>,
    pub icon: Option<&'static str>,
    pub url: Option<&'static str>,
    pub link: Option<&'static str>,
    pub redirect: Option<&'static str>,
    pub schema_api: Option<&'static str>,
    #[schema(no_recursion)]
    pub children: Option<Vec<MenuTree>>,
    #[builder(default = true)]
    pub visible: bool,
}

include!(concat!(env!("OUT_DIR"), "/menus.rs"));
