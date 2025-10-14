use crate::organization::{error::OrganizationError, value_object::hashed_password::PasswordError};
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AuthError {
    #[error("failed_to_generate_captcha")]
    CaptchaGenerationFailed,
    #[error("invalid_captcha")]
    CaptchaInvalid,
    #[error("incorrect_captcha")]
    CaptchaIncorrect,
    #[error("failed_to_generate_access_token")]
    AccessTokenSignFailed,
    #[error("failed_to_verify_access_token")]
    AccessTokenVerifyFailed,
    #[error("failed_to_save_access_token")]
    AccessTokenSaveFailed,
    #[error(transparent)]
    Password(#[from] PasswordError),
    #[error(transparent)]
    Organization(#[from] OrganizationError),
    #[error("database_error")]
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
