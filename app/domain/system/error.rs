use crate::system::value_object::hashed_password::PasswordError;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SystemError {
    #[error("用户不存在")]
    UserNotFound,
    #[error("用户已禁用")]
    UserDisabled,
    #[error("用户已存在")]
    UserDuplicated,
    #[error("两次密码不一致")]
    PasswordMismatch,
    #[error("新密码不能与原密码相同")]
    PasswordUnchanged,
    #[error("无法修改特权用户")]
    UserPrivilegedImmutable,

    #[error("角色不存在")]
    RoleNotFound,
    #[error("角色已禁用")]
    RoleDisabled,
    #[error("角色已存在")]
    RoleDuplicated,
    #[error("无法修改特权角色")]
    RolePrivilegedImmutable,

    #[error("验证码生成失败")]
    CaptchaGenerationFailed,
    #[error("验证码无效")]
    CaptchaInvalid,
    #[error("验证码错误")]
    CaptchaIncorrect,

    #[error("访问 Token 签名错误")]
    AccessTokenSignFailed,
    #[error("访问 Token 解析错误")]
    AccessTokenVerifyFailed,
    #[error("访问 Token 保存错误")]
    AccessTokenSaveFailed,
    #[error("Refresh Token 已过期")]
    RefreshTokenExpired,

    #[error("{0}")]
    Password(#[from] PasswordError),

    #[error("{0}")]
    Sqlx(String),
}

impl From<sqlx::Error> for SystemError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(%err, "sqlx error");
        let message = err.to_string();
        Self::Sqlx(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_error() {
        let err = SystemError::from(sqlx::Error::RowNotFound);
        assert_eq!(err, SystemError::Sqlx(sqlx::Error::RowNotFound.to_string()));
    }
}
