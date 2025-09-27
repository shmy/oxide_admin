use std::sync::LazyLock;

use bon::Builder;
use domain::iam::value_object::permission::{
    Permission, SYSTEM_ROLE_CREATE, SYSTEM_ROLE_DELETE, SYSTEM_ROLE_DISABLE, SYSTEM_ROLE_ENABLE,
    SYSTEM_ROLE_READ, SYSTEM_ROLE_UPDATE, SYSTEM_UPLOAD_FILE, SYSTEM_USER_CREATE,
    SYSTEM_USER_DELETE, SYSTEM_USER_DISABLE, SYSTEM_USER_ENABLE, SYSTEM_USER_READ,
    SYSTEM_USER_UPDATE, SYSTEM_USER_UPDATE_PASSWORD,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Builder, ToSchema)]
pub struct PermissionTree {
    label: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<Permission>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(no_recursion)]
    children: Option<Vec<PermissionTree>>,
}

pub static PERMISSIONS: LazyLock<[PermissionTree; 3]> = LazyLock::new(|| {
    [
        PermissionTree::builder()
            .label("用户")
            .children(vec![
                PermissionTree::builder()
                    .label("读取")
                    .value(SYSTEM_USER_READ)
                    .build(),
                PermissionTree::builder()
                    .label("创建")
                    .value(SYSTEM_USER_CREATE)
                    .build(),
                PermissionTree::builder()
                    .label("更新")
                    .value(SYSTEM_USER_UPDATE)
                    .build(),
                PermissionTree::builder()
                    .label("删除")
                    .value(SYSTEM_USER_DELETE)
                    .build(),
                PermissionTree::builder()
                    .label("启用")
                    .value(SYSTEM_USER_ENABLE)
                    .build(),
                PermissionTree::builder()
                    .label("禁用")
                    .value(SYSTEM_USER_DISABLE)
                    .build(),
                PermissionTree::builder()
                    .label("修改密码")
                    .value(SYSTEM_USER_UPDATE_PASSWORD)
                    .build(),
            ])
            .build(),
        PermissionTree::builder()
            .label("角色")
            .children(vec![
                PermissionTree::builder()
                    .label("读取")
                    .value(SYSTEM_ROLE_READ)
                    .build(),
                PermissionTree::builder()
                    .label("创建")
                    .value(SYSTEM_ROLE_CREATE)
                    .build(),
                PermissionTree::builder()
                    .label("更新")
                    .value(SYSTEM_ROLE_UPDATE)
                    .build(),
                PermissionTree::builder()
                    .label("删除")
                    .value(SYSTEM_ROLE_DELETE)
                    .build(),
                PermissionTree::builder()
                    .label("启用")
                    .value(SYSTEM_ROLE_ENABLE)
                    .build(),
                PermissionTree::builder()
                    .label("禁用")
                    .value(SYSTEM_ROLE_DISABLE)
                    .build(),
            ])
            .build(),
        PermissionTree::builder()
            .label("其他")
            .children(vec![
                PermissionTree::builder()
                    .label("上传文件")
                    .value(SYSTEM_UPLOAD_FILE)
                    .build(),
            ])
            .build(),
    ]
});
