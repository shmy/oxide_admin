use crate::organization::value_object::hashed_password::PasswordError;
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OrganizationError {
    #[error("department_not_found")]
    DepartmentNotFound,
    #[error("user_not_found")]
    UserNotFound,
    #[error("user_disabled")]
    UserDisabled,
    #[error("user_duplicated")]
    UserDuplicated,
    #[error("password_mismatch")]
    PasswordMismatch,
    #[error("password_unchanged")]
    PasswordUnchanged,
    #[error("privileged_user_immutable")]
    UserPrivilegedImmutable,
    #[error("role_not_found")]
    RoleNotFound,
    #[error("role_disabled")]
    RoleDisabled,
    #[error("role_duplicated")]
    RoleDuplicated,
    #[error("privileged_role_immutable")]
    RolePrivilegedImmutable,
    #[error("refresh_token_expired")]
    RefreshTokenExpired,
    #[error(transparent)]
    Password(#[from] PasswordError),
    #[error("database_error")]
    Sqlx(String),
}
impl From<sqlx::Error> for OrganizationError {
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
        let err = OrganizationError::from(sqlx::Error::RowNotFound);
        assert_eq!(
            err,
            OrganizationError::Sqlx(sqlx::Error::RowNotFound.to_string())
        );
    }
}
