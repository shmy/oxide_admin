use crate::iam::value_object::hashed_password::PasswordError;

#[derive(Debug, thiserror::Error)]
pub enum IamError {
    #[error("用户不存在")]
    UserNotFound,
    #[error("用户已禁用")]
    UserDisabled,
    #[error("用户已存在")]
    UserDuplicated,
    #[error("两次密码不一致")]
    TwoPasswordsInconsistent,
    #[error("头像保存失败")]
    PortraitSaveFailed,

    #[error("新密码不能与原密码相同")]
    CannotSameOriginalPassword,

    #[error("无法修改特权用户")]
    CannotPrivilegedUserPassword,

    #[error("角色不存在")]
    RoleNotFound,
    #[error("角色已禁用")]
    RoleDisabled,
    #[error("角色已存在")]
    RoleDuplicated,

    #[error("验证码生成失败")]
    CaptchaFailedGenerate,

    #[error("Token 签名错误")]
    AccessTokenFailedGenerate,
    #[error("Token 解析错误")]
    AccessTokenFailedVerify,
    #[error("Token 保存错误")]
    AccessTokenFailedSave,
    #[error("Refresh token 已过期")]
    RefreshTokenExpired,

    #[error("{0}")]
    Password(#[from] PasswordError),

    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),
}
