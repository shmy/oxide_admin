use std::sync::LazyLock;

use bon::Builder;
use domain::iam::value_object::menu::{
    Menu, NONE, SYSTEM, SYSTEM_EXAMPLE, SYSTEM_ROLE, SYSTEM_STAT, SYSTEM_USER,
};
use serde::Serialize;

use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Builder, ToSchema)]
pub struct MenuTree {
    pub key: Menu,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<&'static str>,
    #[serde(rename = "schemaApi", skip_serializing_if = "Option::is_none")]
    pub schema_api: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(no_recursion)]
    pub children: Option<Vec<MenuTree>>,
    #[builder(default = true)]
    #[serde(skip_serializing_if = "is_true")]
    pub visible: bool,
}

fn is_true(b: &bool) -> bool {
    *b
}

macro_rules! build_schema_url {
    ($path:expr) => {
        concat!("jsonp:", "/_", "/static/pages", $path, ".js?callback=_j")
    };
}

pub static MENUS: LazyLock<[MenuTree; 1]> = LazyLock::new(|| {
    [MenuTree::builder()
        .key(SYSTEM)
        .label("系统管理")
        .icon("fas fa-screwdriver-wrench")
        .children(vec![
            MenuTree::builder()
                .key(SYSTEM_USER)
                .label("用户管理")
                .url("/system/user")
                .icon("fas fa-user")
                .schema_api(build_schema_url!("/system/user"))
                .build(),
            MenuTree::builder()
                .key(SYSTEM_ROLE)
                .label("角色管理")
                .url("/system/role")
                .icon("fas fa-people-group")
                .schema_api(build_schema_url!("/system/role"))
                .build(),
            MenuTree::builder()
                .key(SYSTEM_STAT)
                .label("系统信息")
                .url("/system/info")
                .icon("fas fa-info")
                .schema_api(build_schema_url!("/system/info"))
                .build(),
            MenuTree::builder()
                .key(SYSTEM_EXAMPLE)
                .label("示例页面")
                .url("/system/example")
                .icon("fas fa-hexagon-nodes")
                .schema_api(build_schema_url!("/system/example"))
                .build(),
        ])
        .build()]
});

pub static SHARED_MENUS: LazyLock<[MenuTree; 1]> = LazyLock::new(|| {
    [MenuTree::builder()
        .key(NONE)
        .label("修改密码")
        .url("/profile/update_password")
        .schema_api(build_schema_url!("/profile/update_password"))
        .visible(false)
        .build()]
});
