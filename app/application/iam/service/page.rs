use std::sync::LazyLock;

use bon::Builder;
use domain::iam::value_object::permission_code::{
    NONE, PermissionCode, SYSTEM_INFO, SYSTEM_UPGRADE,
};
use serde::Serialize;

use domain::iam::value_object::permission_code::{SYSTEM, SYSTEM_ROLE, SYSTEM_USER};

#[derive(Debug, Clone, Serialize, Builder)]
pub struct Page {
    pub key: PermissionCode,
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
    pub children: Option<Vec<Page>>,
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

pub static PAGES: LazyLock<[Page; 1]> = LazyLock::new(|| {
    [Page::builder()
        .key(SYSTEM)
        .label("系统管理")
        .icon("fas fa-screwdriver-wrench")
        .children(vec![
            Page::builder()
                .key(SYSTEM_USER)
                .label("用户管理")
                .url("/system/user")
                .icon("fas fa-user")
                .schema_api(build_schema_url!("/system/user"))
                .build(),
            Page::builder()
                .key(SYSTEM_ROLE)
                .label("角色管理")
                .url("/system/role")
                .icon("fas fa-people-group")
                .schema_api(build_schema_url!("/system/role"))
                .build(),
            Page::builder()
                .key(SYSTEM_INFO)
                .label("系统信息")
                .url("/system/info")
                .icon("fas fa-info")
                .schema_api(build_schema_url!("/system/info"))
                .build(),
            Page::builder()
                .key(SYSTEM_UPGRADE)
                .label("系统更新")
                .url("/system/upgrade")
                .icon("fas fa-rocket")
                .schema_api(build_schema_url!("/system/upgrade"))
                .build(),
        ])
        .build()]
});

pub static SHARED_PAGES: LazyLock<[Page; 1]> = LazyLock::new(|| {
    [Page::builder()
        .key(NONE)
        .label("修改密码")
        .url("/profile/update_password")
        .schema_api(build_schema_url!("/profile/update_password"))
        .visible(false)
        .build()]
});
