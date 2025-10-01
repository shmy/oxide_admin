use crate::system::entity::file::File;
use crate::{
    shared::event_util::UpdatedEvent,
    system::{
        entity::{role::Role, user::User},
        value_object::user_id::UserId,
    },
};
#[derive(Debug, Clone)]
pub enum SystemEvent {
    UsersCreated { items: Vec<User> },
    UsersUpdated { items: Vec<UpdatedEvent<User>> },
    UsersDeleted { items: Vec<User> },
    UserLoginSucceeded { id: UserId },
    UserLogoutSucceeded { id: UserId },
    UserRefreshTokenSucceeded { id: UserId },
    RolesCreated { items: Vec<Role> },
    RolesUpdated { items: Vec<UpdatedEvent<Role>> },
    RolesDeleted { items: Vec<Role> },
    FilesCreated { items: Vec<File> },
    FilesUpdated { items: Vec<UpdatedEvent<File>> },
    FilesDeleted { items: Vec<File> },
}
