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
