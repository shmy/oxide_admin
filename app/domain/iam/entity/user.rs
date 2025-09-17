use crate::iam::{
    error::IamError,
    value_object::{hashed_password::HashedPassword, role_id::RoleId, user_id::UserId},
};
use bon::Builder;
use sqlx::types::chrono::{NaiveDateTime, Utc};

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct User {
    pub id: UserId,
    pub account: String,
    pub portrait: Option<String>,
    pub name: String,
    pub privileged: bool,
    pub password: HashedPassword,
    pub role_ids: Vec<RoleId>,
    pub enabled: bool,
    pub refresh_token: Option<String>,
    pub refresh_token_expired_at: Option<NaiveDateTime>,
}

impl User {
    pub fn update_account(&mut self, account: String) {
        self.account = account;
    }

    pub fn update_portrait(&mut self, portrait: Option<String>) {
        self.portrait = portrait;
    }

    pub fn update_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn update_role_ids(&mut self, role_ids: Vec<RoleId>) {
        self.role_ids = role_ids;
    }

    pub fn update_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn update_password(&mut self, password: String) -> Result<(), IamError> {
        self.password = HashedPassword::try_new(password)?;
        Ok(())
    }

    pub fn update_refresh_token(
        &mut self,
        refresh_token: Option<String>,
        refresh_token_expired_at: Option<NaiveDateTime>,
    ) {
        self.refresh_token = refresh_token;
        self.refresh_token_expired_at = refresh_token_expired_at;
    }

    pub fn assert_activated(&self) -> Result<(), IamError> {
        if !self.enabled {
            return Err(IamError::UserDisabled);
        }
        Ok(())
    }
    pub fn assert_refresh_token_valid_period(&self) -> Result<(), IamError> {
        let Some(refresh_token_expired_at) = self.refresh_token_expired_at else {
            return Err(IamError::RefreshTokenExpired);
        };
        if refresh_token_expired_at.and_utc() < Utc::now() {
            return Err(IamError::RefreshTokenExpired);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::iam::value_object::{hashed_password::PasswordError, role_id::RoleId};

    #[test]
    fn test_update_account() {
        let mut user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(true)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(true)
            .build();
        user.update_account("test2".to_string());
        assert_eq!(user.account, "test2");
    }

    #[test]
    fn test_update_portrait() {
        let mut user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(true)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(true)
            .build();
        user.update_portrait(Some("test2".to_string()));
        assert_eq!(user.portrait, Some("test2".to_string()));
    }

    #[test]
    fn test_update_enabled() {
        let mut user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(true)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(true)
            .build();
        user.update_enabled(false);
        assert_eq!(user.enabled, false);
    }

    #[test]
    fn test_update_role_ids() {
        let mut user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(true)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(true)
            .build();
        user.update_role_ids(vec![RoleId::generate()]);
        assert_eq!(user.role_ids.len(), 1);
    }

    #[test]
    fn test_update_name() {
        let mut user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(true)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(true)
            .build();
        user.update_name("test2".to_string());
        assert_eq!(user.name, "test2");
    }

    #[test]
    fn test_update_password() {
        let mut user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(true)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(true)
            .build();
        assert!(user.update_password("123456".to_string()).is_ok());
    }

    #[test]
    fn should_update_password_return_err_given_invalid_password() {
        let mut user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(true)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(true)
            .build();
        assert_eq!(
            user.update_password("1234".to_string()),
            Err(IamError::Password(PasswordError::TooShort))
        );
    }

    #[test]
    fn test_update_refresh_token() {
        let mut user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(true)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(true)
            .build();
        user.update_refresh_token(Some("test2".to_string()), Some(Utc::now().naive_local()));
        assert_eq!(user.refresh_token, Some("test2".to_string()));
    }

    #[test]
    fn should_assert_activated_return_ok_given_user_enabled() {
        let user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(true)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(true)
            .build();
        assert!(user.assert_activated().is_ok());
    }

    #[test]
    fn should_assert_activated_return_err_given_user_disabled() {
        let user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(true)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(false)
            .build();
        assert_eq!(user.assert_activated(), Err(IamError::UserDisabled));
    }

    #[test]
    fn should_assert_refresh_token_valid_period_return_err_given_no_set_expired_at() {
        let user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(true)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(true)
            .build();
        let result = user.assert_refresh_token_valid_period();
        assert_eq!(result, Err(IamError::RefreshTokenExpired));
    }

    #[test]
    fn should_assert_refresh_token_valid_period_is_expired_at_return_err_given_expired_at() {
        let user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(true)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(true)
            .refresh_token_expired_at({
                let datetiem = Utc::now().naive_local() - Duration::from_secs(10);
                datetiem
            })
            .build();
        let result = user.assert_refresh_token_valid_period();
        assert_eq!(result, Err(IamError::RefreshTokenExpired));
    }

    #[test]
    fn should_assert_refresh_token_valid_period_return_ok_given_unexpired_at() {
        let user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .privileged(true)
            .password(HashedPassword::try_new("123456".to_string()).unwrap())
            .role_ids(vec![])
            .enabled(true)
            .refresh_token_expired_at({
                let datetiem = Utc::now().naive_local() + Duration::from_secs(10);
                datetiem
            })
            .build();
        let result = user.assert_refresh_token_valid_period();
        assert!(result.is_ok());
    }
}
