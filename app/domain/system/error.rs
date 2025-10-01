use crate::system::value_object::hashed_password::PasswordError;
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SystemError {
    #[error("User not found")]
    UserNotFound,
    #[error("User disabled")]
    UserDisabled,
    #[error("User duplicated")]
    UserDuplicated,
    #[error("Password mismatch")]
    PasswordMismatch,
    #[error("Password unchanged")]
    PasswordUnchanged,
    #[error("Privileged user immutable")]
    UserPrivilegedImmutable,
    #[error("Role not found")]
    RoleNotFound,
    #[error("Role disabled")]
    RoleDisabled,
    #[error("Role duplicated")]
    RoleDuplicated,
    #[error("Privileged role immutable")]
    RolePrivilegedImmutable,
    #[error("Captcha generation failed")]
    CaptchaGenerationFailed,
    #[error("Invalid captcha")]
    CaptchaInvalid,
    #[error("Incorrect captcha")]
    CaptchaIncorrect,
    #[error("Failed to generate access token")]
    AccessTokenSignFailed,
    #[error("Failed to verify access token")]
    AccessTokenVerifyFailed,
    #[error("Failed to save access token")]
    AccessTokenSaveFailed,
    #[error("Refresh token expired")]
    RefreshTokenExpired,
    #[error("{0}")]
    Password(#[from] PasswordError),
    #[error("{0}")]
    Sqlx(String),
    #[error("File not exists")]
    FileNotFound,
}
impl From<sqlx::Error> for SystemError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(% err, "sqlx error");
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
