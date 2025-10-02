use crate::organization::value_object::hashed_password::PasswordError;
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AuthError {
    #[error("Password mismatch")]
    PasswordMismatch,
    #[error("Password unchanged")]
    PasswordUnchanged,
    #[error("Privileged user immutable")]
    UserPrivilegedImmutable,
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
}
impl From<sqlx::Error> for AuthError {
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
    fn test_organization_error() {
        let err = AuthError::from(sqlx::Error::RowNotFound);
        assert_eq!(err, AuthError::Sqlx(sqlx::Error::RowNotFound.to_string()));
    }
}
