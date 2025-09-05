use argon2::password_hash::SaltString;
use argon2::{Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, password_hash};
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::LazyLock;
use thiserror::Error;

static ARGON2: LazyLock<Argon2> = LazyLock::new(|| {
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::default(),
    )
});

#[derive(Debug, Clone, Error)]
pub enum PasswordError {
    #[error("密码太短")]
    TooShort,
    #[error("密码太长")]
    TooLong,
    #[error("密码加密失败")]
    EncodeFailed,
    #[error("密码解密失败")]
    DecodeFailed,
    #[error("密码错误")]
    Incorrect,
}
#[derive(Clone, sqlx::Type)]
#[sqlx(transparent)]
pub struct HashedPassword(String);

impl HashedPassword {
    pub fn try_new(password: String) -> Result<Self, PasswordError> {
        let password = password.trim();
        let len = password.chars().count();
        if len < 6 {
            return Err(PasswordError::TooShort);
        }
        if len > 64 {
            return Err(PasswordError::TooLong);
        }
        let password_hash = Self::argon2_hash_password(password)?;
        Ok(Self(password_hash))
    }

    pub fn new_unchecked(hash: String) -> Self {
        Self(hash)
    }

    fn argon2_hash_password(password: &str) -> Result<String, PasswordError> {
        let password = password.to_owned();

        let salt = SaltString::generate(&mut password_hash::rand_core::OsRng);
        let hash = ARGON2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| PasswordError::EncodeFailed)?; // 处理 JoinError
        Ok(hash.to_string())
    }

    pub fn verify(&self, password: &str) -> Result<(), PasswordError> {
        let parsed_hash = PasswordHash::new(&self.0).map_err(|_| PasswordError::DecodeFailed)?;
        if ARGON2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_err()
        {
            return Err(PasswordError::Incorrect);
        }
        Ok(())
    }
}

impl Debug for HashedPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("HashedPassword")
            .field(&"<RESERVED>")
            .finish()
    }
}

impl Deref for HashedPassword {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for HashedPassword {
    fn from(value: String) -> Self {
        Self::new_unchecked(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_new_valid() {
        let password = "validpassword123".to_string();
        let hashed = HashedPassword::try_new(password).unwrap();
        assert!(!hashed.0.is_empty());
    }

    #[test]
    fn test_try_new_too_short() {
        let password = "short".to_string();
        let result = HashedPassword::try_new(password);
        assert!(matches!(result, Err(PasswordError::TooShort)));
    }

    #[test]
    fn test_try_new_too_long() {
        let password = "thispasswordiswaytoolongandshouldfailvalidationthispasswordiswaytoolongandshouldfailvalidation".to_string();
        let result = HashedPassword::try_new(password);
        assert!(matches!(result, Err(PasswordError::TooLong)));
    }

    #[test]
    fn test_verify_correct() {
        let password = "testpassword".to_string();
        let hashed = HashedPassword::try_new(password.clone()).unwrap();
        assert!(hashed.verify(&password).is_ok());
    }

    #[test]
    fn test_verify_incorrect() {
        let password = "testpassword".to_string();
        let hashed = HashedPassword::try_new(password).unwrap();
        assert!(matches!(
            hashed.verify("wrongpassword"),
            Err(PasswordError::Incorrect)
        ));
    }
}
