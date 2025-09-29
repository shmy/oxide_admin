use crate::{
    system::{
        entity::{role::Role, user::User},
        value_object::user_id::UserId,
    },
    shared::event_util::UpdatedEvent,
};

#[derive(Debug, Clone)]
pub enum IamEvent {
    UsersCreated { items: Vec<User> },
    UsersUpdated { items: Vec<UpdatedEvent<User>> },
    UsersDeleted { items: Vec<User> },

    UserLoginSucceeded { id: UserId },
    UserLogoutSucceeded { id: UserId },
    UserRefreshTokenSucceeded { id: UserId },

    RolesCreated { items: Vec<Role> },
    RolesUpdated { items: Vec<UpdatedEvent<Role>> },
    RolesDeleted { items: Vec<Role> },
}
