use crate::organization::value_object::user_id::UserId;

#[derive(Debug, Clone)]
pub enum AuthEvent {
    UserRefreshTokenSucceeded { id: UserId },
    UserLoginSucceeded { id: UserId },
    UserLogoutSucceeded { id: UserId },
}
